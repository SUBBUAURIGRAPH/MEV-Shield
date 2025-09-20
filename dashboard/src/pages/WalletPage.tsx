import React, { useState } from 'react';
import {
  Box,
  Container,
  Typography,
  Grid,
  Card,
  CardContent,
  Button,
  TextField,
  Paper,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  Avatar,
  Chip,
  IconButton,
  Divider,
  Alert,
  Tab,
  Tabs,
} from '@mui/material';
import {
  AccountBalanceWallet,
  Send,
  CallReceived,
  SwapHoriz,
  ContentCopy,
  QrCode,
  Add,
  Remove,
  TrendingUp,
  TrendingDown,
  AttachMoney,
} from '@mui/icons-material';

const WalletPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [sendAmount, setSendAmount] = useState('');
  const [recipient, setRecipient] = useState('');

  const walletBalance = {
    total: '$124,580.00',
    eth: '42.5 ETH',
    usdc: '15,000 USDC',
    usdt: '8,500 USDT',
  };

  const transactions = [
    {
      id: 1,
      type: 'receive',
      amount: '2.5 ETH',
      from: '0x742d...bEb3',
      time: '2 hours ago',
      status: 'completed',
    },
    {
      id: 2,
      type: 'send',
      amount: '1,000 USDC',
      to: '0x9f8c...4d2a',
      time: '5 hours ago',
      status: 'completed',
    },
    {
      id: 3,
      type: 'swap',
      amount: '0.5 ETH → 1,100 USDC',
      time: '1 day ago',
      status: 'completed',
    },
  ];

  const assets = [
    { symbol: 'ETH', name: 'Ethereum', balance: '42.5', value: '$91,375', change: '+5.2%' },
    { symbol: 'USDC', name: 'USD Coin', balance: '15,000', value: '$15,000', change: '0.0%' },
    { symbol: 'USDT', name: 'Tether', balance: '8,500', value: '$8,500', change: '0.0%' },
    { symbol: 'WBTC', name: 'Wrapped Bitcoin', balance: '0.25', value: '$9,705', change: '+3.8%' },
  ];

  return (
    <Container maxWidth="xl" sx={{ py: 3 }}>
      <Box sx={{ mb: 3 }}>
        <Typography variant="h4" gutterBottom sx={{ display: 'flex', alignItems: 'center' }}>
          <AccountBalanceWallet sx={{ mr: 2, fontSize: 40 }} />
          Wallet
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage your crypto assets and transactions
        </Typography>
      </Box>

      <Grid container spacing={3}>
        {/* Balance Card */}
        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>Total Balance</Typography>
              <Typography variant="h3" sx={{ mb: 2 }}>
                {walletBalance.total}
              </Typography>
              <Box sx={{ display: 'flex', gap: 1 }}>
                <Chip label={walletBalance.eth} size="small" />
                <Chip label={walletBalance.usdc} size="small" />
              </Box>
              <Box sx={{ mt: 3, display: 'flex', gap: 1 }}>
                <Button variant="contained" startIcon={<Send />} size="small">
                  Send
                </Button>
                <Button variant="outlined" startIcon={<CallReceived />} size="small">
                  Receive
                </Button>
                <Button variant="outlined" startIcon={<SwapHoriz />} size="small">
                  Swap
                </Button>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Quick Actions */}
        <Grid item xs={12} md={8}>
          <Paper sx={{ p: 3 }}>
            <Tabs value={activeTab} onChange={(e, v) => setActiveTab(v)} sx={{ mb: 2 }}>
              <Tab label="Send" icon={<Send />} iconPosition="start" />
              <Tab label="Receive" icon={<CallReceived />} iconPosition="start" />
              <Tab label="Swap" icon={<SwapHoriz />} iconPosition="start" />
            </Tabs>

            {activeTab === 0 && (
              <Box>
                <Grid container spacing={2}>
                  <Grid item xs={12}>
                    <TextField
                      fullWidth
                      label="Recipient Address"
                      value={recipient}
                      onChange={(e) => setRecipient(e.target.value)}
                      placeholder="0x..."
                    />
                  </Grid>
                  <Grid item xs={12} md={6}>
                    <TextField
                      fullWidth
                      label="Amount"
                      value={sendAmount}
                      onChange={(e) => setSendAmount(e.target.value)}
                      type="number"
                    />
                  </Grid>
                  <Grid item xs={12} md={6}>
                    <TextField
                      fullWidth
                      label="Asset"
                      select
                      defaultValue="ETH"
                      SelectProps={{ native: true }}
                    >
                      <option value="ETH">ETH</option>
                      <option value="USDC">USDC</option>
                      <option value="USDT">USDT</option>
                    </TextField>
                  </Grid>
                  <Grid item xs={12}>
                    <Button variant="contained" fullWidth>
                      Send Transaction
                    </Button>
                  </Grid>
                </Grid>
              </Box>
            )}

            {activeTab === 1 && (
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h6" gutterBottom>Your Wallet Address</Typography>
                <Paper sx={{ p: 2, bgcolor: 'background.default' }}>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                    0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb3
                  </Typography>
                  <IconButton size="small">
                    <ContentCopy />
                  </IconButton>
                </Paper>
                <Box sx={{ mt: 2 }}>
                  <QrCode sx={{ fontSize: 120 }} />
                </Box>
              </Box>
            )}

            {activeTab === 2 && (
              <Alert severity="info">
                Swap functionality coming soon. Use the Trading dashboard for DEX swaps.
              </Alert>
            )}
          </Paper>
        </Grid>

        {/* Assets List */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>Assets</Typography>
            <List>
              {assets.map((asset, index) => (
                <React.Fragment key={asset.symbol}>
                  <ListItem>
                    <ListItemAvatar>
                      <Avatar sx={{ bgcolor: 'primary.main' }}>
                        {asset.symbol[0]}
                      </Avatar>
                    </ListItemAvatar>
                    <ListItemText
                      primary={asset.name}
                      secondary={`${asset.balance} ${asset.symbol}`}
                    />
                    <Box sx={{ textAlign: 'right' }}>
                      <Typography variant="body2">{asset.value}</Typography>
                      <Chip
                        label={asset.change}
                        size="small"
                        color={asset.change.startsWith('+') ? 'success' : 'error'}
                        icon={asset.change.startsWith('+') ? <TrendingUp /> : <TrendingDown />}
                      />
                    </Box>
                  </ListItem>
                  {index < assets.length - 1 && <Divider />}
                </React.Fragment>
              ))}
            </List>
          </Paper>
        </Grid>

        {/* Recent Transactions */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>Recent Transactions</Typography>
            <List>
              {transactions.map((tx) => (
                <ListItem key={tx.id}>
                  <ListItemAvatar>
                    <Avatar sx={{ bgcolor: tx.type === 'send' ? 'error.light' : 'success.light' }}>
                      {tx.type === 'send' ? <Send /> : tx.type === 'receive' ? <CallReceived /> : <SwapHoriz />}
                    </Avatar>
                  </ListItemAvatar>
                  <ListItemText
                    primary={tx.amount}
                    secondary={tx.type === 'send' ? `To: ${tx.to}` : tx.type === 'receive' ? `From: ${tx.from}` : tx.time}
                  />
                  <Chip
                    label={tx.status}
                    size="small"
                    color="success"
                    variant="outlined"
                  />
                </ListItem>
              ))}
            </List>
          </Paper>
        </Grid>
      </Grid>
    </Container>
  );
};

export default WalletPage;