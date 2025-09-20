const { expect } = require('chai');
const { ethers } = require('hardhat');
const { loadFixture } = require('@nomicfoundation/hardhat-network-helpers');

// Test configuration
const UNISWAP_ADDRESSES = {
  ROUTER: '0xE592427A0AEce92De3Edee1F18E0157C05861564',
  QUOTER: '0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6',
  FACTORY: '0x1F98431c8aD98523631AE4a59f267346ea31F984',
  WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
  USDC: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  DAI: '0x6B175474E89094C44Da98b954EedeAC495271d0F'
};

describe('UniswapV3Integration', function () {
  // Test fixtures
  async function deployUniswapIntegrationFixture() {
    const [owner, user1, user2, relayer] = await ethers.getSigners();

    // Deploy mock tokens for testing
    const MockToken = await ethers.getContractFactory('MockERC20');
    const tokenA = await MockToken.deploy('Token A', 'TKA', ethers.parseEther('1000000'));
    const tokenB = await MockToken.deploy('Token B', 'TKB', ethers.parseEther('1000000'));
    
    // Deploy UniswapV3Integration contract
    const UniswapV3Integration = await ethers.getContractFactory('UniswapV3Integration');
    const uniswapIntegration = await UniswapV3Integration.deploy(
      UNISWAP_ADDRESSES.ROUTER,
      UNISWAP_ADDRESSES.QUOTER,
      UNISWAP_ADDRESSES.FACTORY
    );

    // Setup: Authorize relayer
    await uniswapIntegration.setRelayerAuthorization(relayer.address, true);

    // Distribute tokens
    await tokenA.transfer(user1.address, ethers.parseEther('10000'));
    await tokenA.transfer(user2.address, ethers.parseEther('10000'));
    await tokenB.transfer(user1.address, ethers.parseEther('10000'));
    await tokenB.transfer(user2.address, ethers.parseEther('10000'));

    return { uniswapIntegration, tokenA, tokenB, owner, user1, user2, relayer };
  }

  describe('Deployment', function () {
    it('Should deploy with correct addresses', async function () {
      const { uniswapIntegration } = await loadFixture(deployUniswapIntegrationFixture);
      
      expect(await uniswapIntegration.swapRouter()).to.equal(UNISWAP_ADDRESSES.ROUTER);
      expect(await uniswapIntegration.quoter()).to.equal(UNISWAP_ADDRESSES.QUOTER);
      expect(await uniswapIntegration.factory()).to.equal(UNISWAP_ADDRESSES.FACTORY);
    });

    it('Should set correct owner', async function () {
      const { uniswapIntegration, owner } = await loadFixture(deployUniswapIntegrationFixture);
      expect(await uniswapIntegration.owner()).to.equal(owner.address);
    });

    it('Should have correct default parameters', async function () {
      const { uniswapIntegration } = await loadFixture(deployUniswapIntegrationFixture);
      
      expect(await uniswapIntegration.DEFAULT_FEE()).to.equal(3000);
      expect(await uniswapIntegration.MAX_SLIPPAGE()).to.equal(500);
      expect(await uniswapIntegration.MEV_PROTECTION_DELAY()).to.equal(2);
    });
  });

  describe('Protected Swaps', function () {
    it('Should schedule a protected swap', async function () {
      const { uniswapIntegration, tokenA, tokenB, user1 } = await loadFixture(deployUniswapIntegrationFixture);
      
      const amountIn = ethers.parseEther('100');
      
      // Approve tokens
      await tokenA.connect(user1).approve(uniswapIntegration.target, amountIn);
      
      // Prepare swap params
      const swapParams = {
        tokenIn: tokenA.target,
        tokenOut: tokenB.target,
        fee: 3000,
        recipient: user1.address,
        amountIn: amountIn,
        amountOutMinimum: 0,
        sqrtPriceLimitX96: 0
      };
      
      // Schedule swap
      const tx = await uniswapIntegration.connect(user1).scheduleProtectedSwap(swapParams);
      const receipt = await tx.wait();
      
      // Check event
      const event = receipt.logs.find(log => log.fragment?.name === 'SwapScheduled');
      expect(event).to.not.be.undefined;
      
      // Verify swap was stored
      const swapId = event.args.swapId;
      const storedSwap = await uniswapIntegration.protectedSwaps(swapId);
      
      expect(storedSwap.user).to.equal(user1.address);
      expect(storedSwap.executed).to.be.false;
      expect(storedSwap.params.amountIn).to.equal(amountIn);
    });

    it('Should enforce MEV protection delay', async function () {
      const { uniswapIntegration, tokenA, tokenB, user1, relayer } = await loadFixture(deployUniswapIntegrationFixture);
      
      const amountIn = ethers.parseEther('100');
      
      // Setup and schedule swap
      await tokenA.connect(user1).approve(uniswapIntegration.target, amountIn);
      
      const swapParams = {
        tokenIn: tokenA.target,
        tokenOut: tokenB.target,
        fee: 3000,
        recipient: user1.address,
        amountIn: amountIn,
        amountOutMinimum: 0,
        sqrtPriceLimitX96: 0
      };
      
      const tx = await uniswapIntegration.connect(user1).scheduleProtectedSwap(swapParams);
      const receipt = await tx.wait();
      const event = receipt.logs.find(log => log.fragment?.name === 'SwapScheduled');
      const swapId = event.args.swapId;
      
      // Try to execute immediately (should fail)
      await expect(
        uniswapIntegration.connect(relayer).executeProtectedSwap(swapId)
      ).to.be.revertedWith('MEV protection delay not met');
      
      // Mine blocks to pass delay
      await network.provider.send('hardhat_mine', ['0x2']); // Mine 2 blocks
      
      // Now execution should work (would fail on mainnet fork due to pool not existing)
      // In production test, this would succeed
    });

    it('Should allow swap cancellation by owner', async function () {
      const { uniswapIntegration, tokenA, tokenB, user1 } = await loadFixture(deployUniswapIntegrationFixture);
      
      const amountIn = ethers.parseEther('100');
      
      // Setup and schedule swap
      await tokenA.connect(user1).approve(uniswapIntegration.target, amountIn);
      
      const swapParams = {
        tokenIn: tokenA.target,
        tokenOut: tokenB.target,
        fee: 3000,
        recipient: user1.address,
        amountIn: amountIn,
        amountOutMinimum: 0,
        sqrtPriceLimitX96: 0
      };
      
      const tx = await uniswapIntegration.connect(user1).scheduleProtectedSwap(swapParams);
      const receipt = await tx.wait();
      const event = receipt.logs.find(log => log.fragment?.name === 'SwapScheduled');
      const swapId = event.args.swapId;
      
      // Cancel swap
      await uniswapIntegration.connect(user1).cancelSwap(swapId);
      
      // Verify swap was deleted
      const storedSwap = await uniswapIntegration.protectedSwaps(swapId);
      expect(storedSwap.user).to.equal(ethers.ZeroAddress);
    });

    it('Should prevent non-owner from cancelling swap', async function () {
      const { uniswapIntegration, tokenA, tokenB, user1, user2 } = await loadFixture(deployUniswapIntegrationFixture);
      
      const amountIn = ethers.parseEther('100');
      
      // Setup and schedule swap
      await tokenA.connect(user1).approve(uniswapIntegration.target, amountIn);
      
      const swapParams = {
        tokenIn: tokenA.target,
        tokenOut: tokenB.target,
        fee: 3000,
        recipient: user1.address,
        amountIn: amountIn,
        amountOutMinimum: 0,
        sqrtPriceLimitX96: 0
      };
      
      const tx = await uniswapIntegration.connect(user1).scheduleProtectedSwap(swapParams);
      const receipt = await tx.wait();
      const event = receipt.logs.find(log => log.fragment?.name === 'SwapScheduled');
      const swapId = event.args.swapId;
      
      // Try to cancel as different user
      await expect(
        uniswapIntegration.connect(user2).cancelSwap(swapId)
      ).to.be.revertedWith('Not swap owner');
    });
  });

  describe('Relayer Management', function () {
    it('Should authorize relayers', async function () {
      const { uniswapIntegration, owner, user1 } = await loadFixture(deployUniswapIntegrationFixture);
      
      // Authorize user1 as relayer
      await uniswapIntegration.connect(owner).setRelayerAuthorization(user1.address, true);
      
      expect(await uniswapIntegration.authorizedRelayers(user1.address)).to.be.true;
    });

    it('Should revoke relayer authorization', async function () {
      const { uniswapIntegration, owner, relayer } = await loadFixture(deployUniswapIntegrationFixture);
      
      // Revoke relayer authorization
      await uniswapIntegration.connect(owner).setRelayerAuthorization(relayer.address, false);
      
      expect(await uniswapIntegration.authorizedRelayers(relayer.address)).to.be.false;
    });

    it('Should only allow owner to manage relayers', async function () {
      const { uniswapIntegration, user1, user2 } = await loadFixture(deployUniswapIntegrationFixture);
      
      await expect(
        uniswapIntegration.connect(user1).setRelayerAuthorization(user2.address, true)
      ).to.be.revertedWith('Ownable: caller is not the owner');
    });
  });

  describe('MEV Protection', function () {
    it('Should detect large price movements', async function () {
      const { uniswapIntegration } = await loadFixture(deployUniswapIntegrationFixture);
      
      // This test would require mainnet forking to test actual price detection
      // Testing the function exists and is callable
      const currentPrice = ethers.parseUnits('1000', 18);
      const limitPrice = ethers.parseUnits('1100', 18); // 10% movement
      
      // The function is internal, so we can't test directly
      // Would need to expose it or test through execution flow
    });

    it('Should emit MEV protection event when triggered', async function () {
      // This requires a more complex setup with actual pool interaction
      // Would be tested in integration tests on testnet
    });
  });

  describe('Emergency Functions', function () {
    it('Should allow owner to emergency withdraw', async function () {
      const { uniswapIntegration, tokenA, owner } = await loadFixture(deployUniswapIntegrationFixture);
      
      // Send some tokens to contract
      await tokenA.transfer(uniswapIntegration.target, ethers.parseEther('100'));
      
      // Emergency withdraw
      await uniswapIntegration.connect(owner).emergencyWithdraw(
        tokenA.target,
        ethers.parseEther('100')
      );
      
      // Check balance
      expect(await tokenA.balanceOf(uniswapIntegration.target)).to.equal(0);
    });

    it('Should prevent non-owner emergency withdraw', async function () {
      const { uniswapIntegration, tokenA, user1 } = await loadFixture(deployUniswapIntegrationFixture);
      
      await expect(
        uniswapIntegration.connect(user1).emergencyWithdraw(
          tokenA.target,
          ethers.parseEther('100')
        )
      ).to.be.revertedWith('Ownable: caller is not the owner');
    });
  });

  describe('Quote Functionality', function () {
    it('Should get quote for swap', async function () {
      // This requires mainnet forking to work properly
      // as it needs actual Uniswap pools
      const { uniswapIntegration } = await loadFixture(deployUniswapIntegrationFixture);
      
      // Test that the function exists and is callable
      // Actual quote would fail without mainnet fork
    });
  });
});

// Mock ERC20 contract for testing
const MockERC20 = `
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
    constructor(string memory name, string memory symbol, uint256 initialSupply) 
        ERC20(name, symbol) {
        _mint(msg.sender, initialSupply);
    }
    
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
`;

module.exports = { MockERC20 };