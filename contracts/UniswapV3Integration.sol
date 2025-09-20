// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@uniswap/v3-periphery/contracts/interfaces/ISwapRouter.sol";
import "@uniswap/v3-periphery/contracts/interfaces/IQuoter.sol";
import "@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol";
import "@uniswap/v3-core/contracts/interfaces/IUniswapV3Factory.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title UniswapV3Integration
 * @dev MEV Shield integration with Uniswap V3 for protected trading
 */
contract UniswapV3Integration is Ownable, ReentrancyGuard {
    ISwapRouter public immutable swapRouter;
    IQuoter public immutable quoter;
    IUniswapV3Factory public immutable factory;
    
    uint24 public constant DEFAULT_FEE = 3000; // 0.3%
    uint256 public constant MAX_SLIPPAGE = 500; // 5%
    uint256 public constant MEV_PROTECTION_DELAY = 2; // blocks
    
    struct SwapParams {
        address tokenIn;
        address tokenOut;
        uint24 fee;
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
        uint160 sqrtPriceLimitX96;
    }
    
    struct ProtectedSwap {
        SwapParams params;
        uint256 blockNumber;
        address user;
        bool executed;
    }
    
    mapping(bytes32 => ProtectedSwap) public protectedSwaps;
    mapping(address => bool) public authorizedRelayers;
    
    event SwapScheduled(bytes32 indexed swapId, address indexed user, address tokenIn, address tokenOut, uint256 amountIn);
    event SwapExecuted(bytes32 indexed swapId, address indexed user, uint256 amountOut);
    event RelayerAuthorized(address indexed relayer, bool authorized);
    event MEVProtectionTriggered(bytes32 indexed swapId, string reason);
    
    modifier onlyAuthorizedRelayer() {
        require(authorizedRelayers[msg.sender] || msg.sender == owner(), "Not authorized relayer");
        _;
    }
    
    constructor(
        address _swapRouter,
        address _quoter,
        address _factory
    ) {
        swapRouter = ISwapRouter(_swapRouter);
        quoter = IQuoter(_quoter);
        factory = IUniswapV3Factory(_factory);
    }
    
    /**
     * @dev Schedule a protected swap with MEV protection
     */
    function scheduleProtectedSwap(SwapParams calldata params) external nonReentrant returns (bytes32) {
        require(params.amountIn > 0, "Amount must be greater than 0");
        require(params.tokenIn != params.tokenOut, "Tokens must be different");
        
        // Transfer tokens to this contract
        IERC20(params.tokenIn).transferFrom(msg.sender, address(this), params.amountIn);
        
        // Generate unique swap ID
        bytes32 swapId = keccak256(abi.encodePacked(
            msg.sender,
            params.tokenIn,
            params.tokenOut,
            params.amountIn,
            block.timestamp
        ));
        
        // Store protected swap
        protectedSwaps[swapId] = ProtectedSwap({
            params: params,
            blockNumber: block.number,
            user: msg.sender,
            executed: false
        });
        
        emit SwapScheduled(swapId, msg.sender, params.tokenIn, params.tokenOut, params.amountIn);
        
        return swapId;
    }
    
    /**
     * @dev Execute a protected swap after MEV protection delay
     */
    function executeProtectedSwap(bytes32 swapId) external onlyAuthorizedRelayer nonReentrant {
        ProtectedSwap storage swap = protectedSwaps[swapId];
        
        require(!swap.executed, "Swap already executed");
        require(swap.user != address(0), "Swap does not exist");
        require(block.number >= swap.blockNumber + MEV_PROTECTION_DELAY, "MEV protection delay not met");
        
        // Check for sandwich attack patterns
        if (detectSandwichAttack(swap.params)) {
            emit MEVProtectionTriggered(swapId, "Sandwich attack detected");
            revert("MEV protection: Sandwich attack detected");
        }
        
        // Approve router to spend tokens
        IERC20(swap.params.tokenIn).approve(address(swapRouter), swap.params.amountIn);
        
        // Execute swap
        ISwapRouter.ExactInputSingleParams memory swapParams = ISwapRouter.ExactInputSingleParams({
            tokenIn: swap.params.tokenIn,
            tokenOut: swap.params.tokenOut,
            fee: swap.params.fee,
            recipient: swap.params.recipient,
            deadline: block.timestamp,
            amountIn: swap.params.amountIn,
            amountOutMinimum: swap.params.amountOutMinimum,
            sqrtPriceLimitX96: swap.params.sqrtPriceLimitX96
        });
        
        uint256 amountOut = swapRouter.exactInputSingle(swapParams);
        
        swap.executed = true;
        
        emit SwapExecuted(swapId, swap.user, amountOut);
    }
    
    /**
     * @dev Get quote for swap
     */
    function getQuote(
        address tokenIn,
        address tokenOut,
        uint24 fee,
        uint256 amountIn
    ) external returns (uint256 amountOut) {
        return quoter.quoteExactInputSingle(
            tokenIn,
            tokenOut,
            fee,
            amountIn,
            0
        );
    }
    
    /**
     * @dev Detect potential sandwich attacks
     */
    function detectSandwichAttack(SwapParams memory params) internal view returns (bool) {
        // Get pool address
        address pool = factory.getPool(params.tokenIn, params.tokenOut, params.fee);
        if (pool == address(0)) return false;
        
        IUniswapV3Pool poolContract = IUniswapV3Pool(pool);
        
        // Check recent pool activity
        (uint160 sqrtPriceX96, , , , , , ) = poolContract.slot0();
        
        // Simple heuristic: Check if price moved significantly in recent blocks
        // In production, implement more sophisticated detection
        uint256 priceMovement = calculatePriceMovement(sqrtPriceX96, params.sqrtPriceLimitX96);
        
        return priceMovement > MAX_SLIPPAGE;
    }
    
    /**
     * @dev Calculate price movement percentage
     */
    function calculatePriceMovement(uint160 currentPrice, uint160 limitPrice) internal pure returns (uint256) {
        if (limitPrice == 0) return 0;
        
        uint256 diff = currentPrice > limitPrice ? 
            currentPrice - limitPrice : 
            limitPrice - currentPrice;
            
        return (diff * 10000) / currentPrice;
    }
    
    /**
     * @dev Authorize or revoke relayer
     */
    function setRelayerAuthorization(address relayer, bool authorized) external onlyOwner {
        authorizedRelayers[relayer] = authorized;
        emit RelayerAuthorized(relayer, authorized);
    }
    
    /**
     * @dev Cancel a scheduled swap (only by user who created it)
     */
    function cancelSwap(bytes32 swapId) external nonReentrant {
        ProtectedSwap storage swap = protectedSwaps[swapId];
        
        require(swap.user == msg.sender, "Not swap owner");
        require(!swap.executed, "Swap already executed");
        
        // Refund tokens
        IERC20(swap.params.tokenIn).transfer(msg.sender, swap.params.amountIn);
        
        delete protectedSwaps[swapId];
    }
    
    /**
     * @dev Emergency withdraw (only owner)
     */
    function emergencyWithdraw(address token, uint256 amount) external onlyOwner {
        IERC20(token).transfer(owner(), amount);
    }
}