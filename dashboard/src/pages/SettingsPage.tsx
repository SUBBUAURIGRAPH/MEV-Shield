import React, { useState } from 'react';
import {
  Box,
  Container,
  Typography,
  Paper,
  Grid,
  TextField,
  Button,
  Switch,
  FormControlLabel,
  Divider,
  Tab,
  Tabs,
  Alert,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Slider,
} from '@mui/material';
import {
  Settings,
  Person,
  Security,
  Notifications,
  Palette,
  Language,
  Save,
} from '@mui/icons-material';

const SettingsPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [notifications, setNotifications] = useState({
    email: true,
    push: false,
    trading: true,
    security: true,
  });
  const [security, setSecurity] = useState({
    twoFactor: true,
    sessionTimeout: 30,
    ipWhitelist: false,
  });

  return (
    <Container maxWidth="lg" sx={{ py: 3 }}>
      <Box sx={{ mb: 3 }}>
        <Typography variant="h4" gutterBottom sx={{ display: 'flex', alignItems: 'center' }}>
          <Settings sx={{ mr: 2, fontSize: 40 }} />
          Settings
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage your account preferences and platform settings
        </Typography>
      </Box>

      <Paper sx={{ mt: 3 }}>
        <Tabs
          value={activeTab}
          onChange={(e, v) => setActiveTab(v)}
          sx={{ borderBottom: 1, borderColor: 'divider', px: 2 }}
        >
          <Tab label="Profile" icon={<Person />} iconPosition="start" />
          <Tab label="Security" icon={<Security />} iconPosition="start" />
          <Tab label="Notifications" icon={<Notifications />} iconPosition="start" />
          <Tab label="Appearance" icon={<Palette />} iconPosition="start" />
        </Tabs>

        <Box sx={{ p: 3 }}>
          {/* Profile Tab */}
          {activeTab === 0 && (
            <Grid container spacing={3}>
              <Grid item xs={12} md={6}>
                <TextField fullWidth label="Display Name" defaultValue="John Doe" />
              </Grid>
              <Grid item xs={12} md={6}>
                <TextField fullWidth label="Email" defaultValue="user@mevshield.ai" disabled />
              </Grid>
              <Grid item xs={12} md={6}>
                <TextField fullWidth label="Phone" defaultValue="+1 234 567 8900" />
              </Grid>
              <Grid item xs={12} md={6}>
                <FormControl fullWidth>
                  <InputLabel>Time Zone</InputLabel>
                  <Select defaultValue="UTC">
                    <MenuItem value="UTC">UTC</MenuItem>
                    <MenuItem value="EST">Eastern Time</MenuItem>
                    <MenuItem value="PST">Pacific Time</MenuItem>
                    <MenuItem value="CET">Central European Time</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
              <Grid item xs={12}>
                <TextField
                  fullWidth
                  multiline
                  rows={3}
                  label="Bio"
                  defaultValue="Professional trader focused on DeFi and MEV protection strategies."
                />
              </Grid>
              <Grid item xs={12}>
                <Button variant="contained" startIcon={<Save />}>
                  Save Profile
                </Button>
              </Grid>
            </Grid>
          )}

          {/* Security Tab */}
          {activeTab === 1 && (
            <Grid container spacing={3}>
              <Grid item xs={12}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={security.twoFactor}
                      onChange={(e) => setSecurity({ ...security, twoFactor: e.target.checked })}
                    />
                  }
                  label="Two-Factor Authentication"
                />
                <Typography variant="caption" display="block" color="text.secondary" sx={{ ml: 4 }}>
                  Add an extra layer of security to your account
                </Typography>
              </Grid>
              <Grid item xs={12}>
                <Typography gutterBottom>Session Timeout (minutes)</Typography>
                <Slider
                  value={security.sessionTimeout}
                  onChange={(e, v) => setSecurity({ ...security, sessionTimeout: v as number })}
                  min={5}
                  max={120}
                  valueLabelDisplay="auto"
                  marks={[
                    { value: 5, label: '5' },
                    { value: 30, label: '30' },
                    { value: 60, label: '60' },
                    { value: 120, label: '120' },
                  ]}
                />
              </Grid>
              <Grid item xs={12}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={security.ipWhitelist}
                      onChange={(e) => setSecurity({ ...security, ipWhitelist: e.target.checked })}
                    />
                  }
                  label="IP Whitelist"
                />
                <Typography variant="caption" display="block" color="text.secondary" sx={{ ml: 4 }}>
                  Only allow access from specific IP addresses
                </Typography>
              </Grid>
              <Grid item xs={12}>
                <Divider sx={{ my: 2 }} />
                <Typography variant="h6" gutterBottom>Change Password</Typography>
                <Grid container spacing={2}>
                  <Grid item xs={12}>
                    <TextField fullWidth type="password" label="Current Password" />
                  </Grid>
                  <Grid item xs={12} md={6}>
                    <TextField fullWidth type="password" label="New Password" />
                  </Grid>
                  <Grid item xs={12} md={6}>
                    <TextField fullWidth type="password" label="Confirm New Password" />
                  </Grid>
                  <Grid item xs={12}>
                    <Button variant="outlined">Change Password</Button>
                  </Grid>
                </Grid>
              </Grid>
            </Grid>
          )}

          {/* Notifications Tab */}
          {activeTab === 2 && (
            <Grid container spacing={3}>
              <Grid item xs={12}>
                <Typography variant="h6" gutterBottom>Email Notifications</Typography>
                <FormControlLabel
                  control={
                    <Switch
                      checked={notifications.email}
                      onChange={(e) => setNotifications({ ...notifications, email: e.target.checked })}
                    />
                  }
                  label="Enable Email Notifications"
                />
              </Grid>
              <Grid item xs={12}>
                <Typography variant="h6" gutterBottom>Push Notifications</Typography>
                <FormControlLabel
                  control={
                    <Switch
                      checked={notifications.push}
                      onChange={(e) => setNotifications({ ...notifications, push: e.target.checked })}
                    />
                  }
                  label="Enable Push Notifications"
                />
              </Grid>
              <Grid item xs={12}>
                <Divider sx={{ my: 2 }} />
                <Typography variant="h6" gutterBottom>Notification Types</Typography>
                <FormControlLabel
                  control={
                    <Switch
                      checked={notifications.trading}
                      onChange={(e) => setNotifications({ ...notifications, trading: e.target.checked })}
                    />
                  }
                  label="Trading Alerts"
                />
                <Typography variant="caption" display="block" color="text.secondary" sx={{ ml: 4 }}>
                  Price alerts, order fills, and trade executions
                </Typography>
                <FormControlLabel
                  control={
                    <Switch
                      checked={notifications.security}
                      onChange={(e) => setNotifications({ ...notifications, security: e.target.checked })}
                    />
                  }
                  label="Security Alerts"
                  sx={{ mt: 2 }}
                />
                <Typography variant="caption" display="block" color="text.secondary" sx={{ ml: 4 }}>
                  Login attempts, password changes, and suspicious activity
                </Typography>
              </Grid>
              <Grid item xs={12}>
                <Button variant="contained" startIcon={<Save />}>
                  Save Notification Settings
                </Button>
              </Grid>
            </Grid>
          )}

          {/* Appearance Tab */}
          {activeTab === 3 && (
            <Grid container spacing={3}>
              <Grid item xs={12}>
                <FormControl fullWidth>
                  <InputLabel>Theme</InputLabel>
                  <Select defaultValue="dark">
                    <MenuItem value="light">Light</MenuItem>
                    <MenuItem value="dark">Dark</MenuItem>
                    <MenuItem value="auto">System Default</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
              <Grid item xs={12}>
                <FormControl fullWidth>
                  <InputLabel>Language</InputLabel>
                  <Select defaultValue="en">
                    <MenuItem value="en">English</MenuItem>
                    <MenuItem value="es">Español</MenuItem>
                    <MenuItem value="fr">Français</MenuItem>
                    <MenuItem value="de">Deutsch</MenuItem>
                    <MenuItem value="ja">日本語</MenuItem>
                    <MenuItem value="zh">中文</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
              <Grid item xs={12}>
                <FormControl fullWidth>
                  <InputLabel>Chart Style</InputLabel>
                  <Select defaultValue="candlestick">
                    <MenuItem value="candlestick">Candlestick</MenuItem>
                    <MenuItem value="line">Line</MenuItem>
                    <MenuItem value="bar">Bar</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
              <Grid item xs={12}>
                <Button variant="contained" startIcon={<Save />}>
                  Save Appearance Settings
                </Button>
              </Grid>
            </Grid>
          )}
        </Box>
      </Paper>

      <Alert severity="info" sx={{ mt: 3 }}>
        <Typography variant="body2">
          <strong>Tip:</strong> Keep your security settings up to date and enable two-factor authentication for maximum protection.
        </Typography>
      </Alert>
    </Container>
  );
};

export default SettingsPage;