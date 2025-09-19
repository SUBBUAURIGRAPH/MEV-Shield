import React, { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Button,
  Chip,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Alert,
  Stepper,
  Step,
  StepLabel,
  StepContent,
  Paper,
  Divider,
  Tab,
  Tabs,
  LinearProgress,
  Badge,
} from '@mui/material';
import {
  Business,
  CheckCircle,
  Warning,
  Error,
  Settings,
  Code,
  Api,
  Security,
  Speed,
  AttachMoney,
  TrendingUp,
  Group,
  Link,
  ContentCopy,
  Visibility,
  Download,
  Upload,
  Refresh,
  Add,
} from '@mui/icons-material';

interface Exchange {
  id: string;
  name: string;
  logo: string;
  status: 'active' | 'pending' | 'inactive';
  volume24h: number;
  transactionsProtected: number;
  mevSaved: number;
  apiVersion: string;
  lastSync: string;
}

const mockExchanges: Exchange[] = [
  {
    id: '1',
    name: 'Binance',
    logo: '🟨',
    status: 'active',
    volume24h: 4500000,
    transactionsProtected: 125420,
    mevSaved: 892000,
    apiVersion: 'v2.1.0',
    lastSync: '2 min ago',
  },
  {
    id: '2',
    name: 'Coinbase',
    logo: '🔵',
    status: 'active',
    volume24h: 3200000,
    transactionsProtected: 98300,
    mevSaved: 654000,
    apiVersion: 'v2.1.0',
    lastSync: '5 min ago',
  },
  {
    id: '3',
    name: 'Kraken',
    logo: '🟣',
    status: 'pending',
    volume24h: 2100000,
    transactionsProtected: 0,
    mevSaved: 0,
    apiVersion: 'v2.0.0',
    lastSync: 'Never',
  },
];

export const ExchangeIntegration: React.FC = () => {
  const [selectedTab, setSelectedTab] = useState(0);
  const [openDialog, setOpenDialog] = useState(false);
  const [activeStep, setActiveStep] = useState(0);
  const [apiKey, setApiKey] = useState('');
  const [selectedExchange, setSelectedExchange] = useState('');

  const integrationSteps = [
    {
      label: 'Generate API Credentials',
      description: 'Create unique API keys for the exchange integration.',
    },
    {
      label: 'Configure Webhook Endpoints',
      description: 'Set up webhook URLs for real-time transaction monitoring.',
    },
    {
      label: 'Implement SDK',
      description: 'Integrate MEV Shield SDK into your exchange infrastructure.',
    },
    {
      label: 'Test Integration',
      description: 'Run test transactions to verify protection is working.',
    },
    {
      label: 'Go Live',
      description: 'Enable MEV protection for all exchange transactions.',
    },
  ];

  const handleNext = () => {
    setActiveStep((prevStep) => prevStep + 1);
  };

  const handleBack = () => {
    setActiveStep((prevStep) => prevStep - 1);
  };

  const generateApiKey = () => {
    const key = 'mev_shield_' + Math.random().toString(36).substr(2, 9);
    setApiKey(key);
  };

  return (
    <Box>
      <Typography variant="h5" gutterBottom sx={{ mb: 3, fontWeight: 600 }}>
        Exchange & Platform Integration
      </Typography>

      <Tabs value={selectedTab} onChange={(_, v) => setSelectedTab(v)} sx={{ mb: 3 }}>
        <Tab label="Connected Exchanges" />
        <Tab label="Integration Guide" />
        <Tab label="API Management" />
        <Tab label="Performance Metrics" />
      </Tabs>

      {selectedTab === 0 && (
        <Grid container spacing={3}>
          <Grid item xs={12}>
            <Button
              variant="contained"
              startIcon={<Add />}
              onClick={() => setOpenDialog(true)}
              sx={{ mb: 2 }}
            >
              Add New Exchange
            </Button>
          </Grid>

          {mockExchanges.map((exchange) => (
            <Grid item xs={12} md={6} lg={4} key={exchange.id}>
              <Card sx={{ height: '100%' }}>
                <CardContent>
                  <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                    <Avatar sx={{ bgcolor: 'primary.main', mr: 2 }}>
                      {exchange.logo}
                    </Avatar>
                    <Box sx={{ flexGrow: 1 }}>
                      <Typography variant="h6">{exchange.name}</Typography>
                      <Chip
                        label={exchange.status}
                        color={
                          exchange.status === 'active'
                            ? 'success'
                            : exchange.status === 'pending'
                            ? 'warning'
                            : 'default'
                        }
                        size="small"
                        icon={
                          exchange.status === 'active' ? (
                            <CheckCircle />
                          ) : exchange.status === 'pending' ? (
                            <Warning />
                          ) : (
                            <Error />
                          )
                        }
                      />
                    </Box>
                    <IconButton>
                      <Settings />
                    </IconButton>
                  </Box>

                  <Grid container spacing={2}>
                    <Grid item xs={6}>
                      <Typography variant="body2" color="text.secondary">
                        24h Volume
                      </Typography>
                      <Typography variant="h6">
                        ${(exchange.volume24h / 1000000).toFixed(1)}M
                      </Typography>
                    </Grid>
                    <Grid item xs={6}>
                      <Typography variant="body2" color="text.secondary">
                        Protected
                      </Typography>
                      <Typography variant="h6">
                        {exchange.transactionsProtected.toLocaleString()}
                      </Typography>
                    </Grid>
                    <Grid item xs={6}>
                      <Typography variant="body2" color="text.secondary">
                        MEV Saved
                      </Typography>
                      <Typography variant="h6">
                        ${(exchange.mevSaved / 1000).toFixed(0)}K
                      </Typography>
                    </Grid>
                    <Grid item xs={6}>
                      <Typography variant="body2" color="text.secondary">
                        Last Sync
                      </Typography>
                      <Typography variant="h6">{exchange.lastSync}</Typography>
                    </Grid>
                  </Grid>

                  <Box sx={{ mt: 2, pt: 2, borderTop: 1, borderColor: 'divider' }}>
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      API Version: {exchange.apiVersion}
                    </Typography>
                    <LinearProgress
                      variant="determinate"
                      value={exchange.status === 'active' ? 100 : exchange.status === 'pending' ? 50 : 0}
                      sx={{ mt: 1 }}
                    />
                  </Box>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
      )}

      {selectedTab === 1 && (
        <Paper sx={{ p: 3 }}>
          <Typography variant="h6" gutterBottom>
            Quick Integration Guide
          </Typography>
          <Stepper activeStep={activeStep} orientation="vertical">
            {integrationSteps.map((step, index) => (
              <Step key={step.label}>
                <StepLabel>{step.label}</StepLabel>
                <StepContent>
                  <Typography>{step.description}</Typography>
                  
                  {index === 0 && (
                    <Box sx={{ mt: 2 }}>
                      <TextField
                        fullWidth
                        label="API Key"
                        value={apiKey}
                        InputProps={{
                          readOnly: true,
                          endAdornment: (
                            <IconButton onClick={generateApiKey}>
                              <Refresh />
                            </IconButton>
                          ),
                        }}
                        sx={{ mb: 2 }}
                      />
                      <Button variant="outlined" onClick={generateApiKey}>
                        Generate New Key
                      </Button>
                    </Box>
                  )}

                  {index === 1 && (
                    <Box sx={{ mt: 2 }}>
                      <TextField
                        fullWidth
                        label="Webhook URL"
                        defaultValue="https://api.your-exchange.com/mev-shield/webhook"
                        sx={{ mb: 2 }}
                      />
                      <Alert severity="info">
                        Webhooks will receive real-time notifications about MEV attacks and protection status.
                      </Alert>
                    </Box>
                  )}

                  {index === 2 && (
                    <Box sx={{ mt: 2 }}>
                      <Typography variant="body2" gutterBottom>
                        Install MEV Shield SDK:
                      </Typography>
                      <Paper sx={{ p: 2, bgcolor: 'grey.900', color: 'common.white' }}>
                        <code>npm install @mev-shield/sdk</code>
                      </Paper>
                      <Box sx={{ mt: 2 }}>
                        <Button variant="outlined" startIcon={<Download />}>
                          Download SDK
                        </Button>
                        <Button variant="outlined" startIcon={<Code />} sx={{ ml: 1 }}>
                          View Documentation
                        </Button>
                      </Box>
                    </Box>
                  )}

                  <Box sx={{ mb: 2, mt: 2 }}>
                    <Button
                      variant="contained"
                      onClick={handleNext}
                      sx={{ mt: 1, mr: 1 }}
                    >
                      {index === integrationSteps.length - 1 ? 'Finish' : 'Continue'}
                    </Button>
                    <Button
                      disabled={index === 0}
                      onClick={handleBack}
                      sx={{ mt: 1, mr: 1 }}
                    >
                      Back
                    </Button>
                  </Box>
                </StepContent>
              </Step>
            ))}
          </Stepper>
        </Paper>
      )}

      {selectedTab === 2 && (
        <Grid container spacing={3}>
          <Grid item xs={12}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                API Configuration
              </Typography>
              
              <List>
                <ListItem>
                  <ListItemAvatar>
                    <Avatar sx={{ bgcolor: 'success.main' }}>
                      <Api />
                    </Avatar>
                  </ListItemAvatar>
                  <ListItemText
                    primary="REST API Endpoint"
                    secondary="https://api.mevshield.com/v2"
                  />
                  <ListItemSecondaryAction>
                    <IconButton>
                      <ContentCopy />
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
                
                <ListItem>
                  <ListItemAvatar>
                    <Avatar sx={{ bgcolor: 'info.main' }}>
                      <Link />
                    </Avatar>
                  </ListItemAvatar>
                  <ListItemText
                    primary="WebSocket Endpoint"
                    secondary="wss://stream.mevshield.com"
                  />
                  <ListItemSecondaryAction>
                    <IconButton>
                      <ContentCopy />
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
                
                <ListItem>
                  <ListItemAvatar>
                    <Avatar sx={{ bgcolor: 'warning.main' }}>
                      <Speed />
                    </Avatar>
                  </ListItemAvatar>
                  <ListItemText
                    primary="Rate Limits"
                    secondary="10,000 requests/minute"
                  />
                </ListItem>
              </List>

              <Divider sx={{ my: 2 }} />

              <Typography variant="subtitle1" gutterBottom>
                SDK Libraries
              </Typography>
              <Grid container spacing={2}>
                <Grid item xs={6} md={3}>
                  <Button fullWidth variant="outlined" startIcon={<Code />}>
                    JavaScript
                  </Button>
                </Grid>
                <Grid item xs={6} md={3}>
                  <Button fullWidth variant="outlined" startIcon={<Code />}>
                    Python
                  </Button>
                </Grid>
                <Grid item xs={6} md={3}>
                  <Button fullWidth variant="outlined" startIcon={<Code />}>
                    Go
                  </Button>
                </Grid>
                <Grid item xs={6} md={3}>
                  <Button fullWidth variant="outlined" startIcon={<Code />}>
                    Rust
                  </Button>
                </Grid>
              </Grid>
            </Paper>
          </Grid>
        </Grid>
      )}

      {selectedTab === 3 && (
        <Grid container spacing={3}>
          <Grid item xs={12} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Security color="primary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary">Protection Rate</Typography>
                </Box>
                <Typography variant="h4">99.8%</Typography>
                <Typography variant="body2" color="success.main">
                  +2.3% from last week
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Speed color="primary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary">Avg Latency</Typography>
                </Box>
                <Typography variant="h4">42ms</Typography>
                <Typography variant="body2" color="success.main">
                  -5ms improvement
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <AttachMoney color="primary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary">Total Saved</Typography>
                </Box>
                <Typography variant="h4">$2.4M</Typography>
                <Typography variant="body2" color="text.secondary">
                  This month
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Group color="primary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary">Active Platforms</Typography>
                </Box>
                <Typography variant="h4">12</Typography>
                <Typography variant="body2" color="info.main">
                  3 pending approval
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {/* Integration Dialog */}
      <Dialog open={openDialog} onClose={() => setOpenDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Add New Exchange Integration</DialogTitle>
        <DialogContent>
          <FormControl fullWidth sx={{ mt: 2, mb: 2 }}>
            <InputLabel>Select Exchange</InputLabel>
            <Select
              value={selectedExchange}
              onChange={(e) => setSelectedExchange(e.target.value)}
              label="Select Exchange"
            >
              <MenuItem value="okx">OKX</MenuItem>
              <MenuItem value="bybit">Bybit</MenuItem>
              <MenuItem value="gateio">Gate.io</MenuItem>
              <MenuItem value="kucoin">KuCoin</MenuItem>
              <MenuItem value="custom">Custom Platform</MenuItem>
            </Select>
          </FormControl>
          
          <TextField
            fullWidth
            label="Exchange API Endpoint"
            placeholder="https://api.exchange.com"
            sx={{ mb: 2 }}
          />
          
          <TextField
            fullWidth
            label="Contact Email"
            type="email"
            placeholder="tech@exchange.com"
            sx={{ mb: 2 }}
          />
          
          <Alert severity="info">
            Our integration team will contact you within 24 hours to complete the setup process.
          </Alert>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpenDialog(false)}>Cancel</Button>
          <Button variant="contained" onClick={() => setOpenDialog(false)}>
            Submit Request
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};