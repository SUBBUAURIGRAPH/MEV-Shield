import React, { useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  AppBar,
  Toolbar,
  Typography,
  Button,
  IconButton,
  Menu,
  MenuItem,
  Box,
  Avatar,
  Chip,
  Tooltip,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Divider,
  useTheme,
  alpha,
  Badge,
  useMediaQuery,
} from '@mui/material';
import {
  Shield,
  Dashboard,
  AccountBalanceWallet,
  Build,
  TrendingUp,
  Settings,
  Notifications,
  ExitToApp,
  Menu as MenuIcon,
  Close,
  Security,
  Analytics,
  SwapHoriz,
  Person,
  DarkMode,
  LightMode,
  Speed,
  Warning,
} from '@mui/icons-material';
import { useAuth } from '../auth/AuthContext';

interface ModernNavigationProps {
  onThemeToggle?: () => void;
  isDarkMode?: boolean;
}

const ModernNavigation: React.FC<ModernNavigationProps> = ({ onThemeToggle, isDarkMode = true }) => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [notifications] = useState(3); // Mock notification count

  const handleProfileMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleProfileMenuClose = () => {
    setAnchorEl(null);
  };

  const handleLogout = () => {
    handleProfileMenuClose();
    logout();
    navigate('/');
  };

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  // Navigation items based on user role
  const getNavigationItems = () => {
    const baseItems = [
      { icon: Dashboard, label: 'Dashboard', path: '/dashboard' },
    ];

    if (user?.role === 'Admin') {
      return [
        ...baseItems,
        { icon: Security, label: 'Security', path: '/security' },
        { icon: Analytics, label: 'Analytics', path: '/analytics' },
        { icon: Person, label: 'Users', path: '/users' },
        { icon: Settings, label: 'Settings', path: '/settings' },
      ];
    } else if (user?.role === 'Trader') {
      return [
        ...baseItems,
        { icon: SwapHoriz, label: 'Trading', path: '/trading' },
        { icon: TrendingUp, label: 'Portfolio', path: '/portfolio' },
        { icon: AccountBalanceWallet, label: 'Wallet', path: '/wallet' },
      ];
    } else if (user?.role === 'Builder') {
      return [
        ...baseItems,
        { icon: Build, label: 'Builds', path: '/builds' },
        { icon: Speed, label: 'Performance', path: '/performance' },
        { icon: Analytics, label: 'Metrics', path: '/metrics' },
      ];
    } else {
      return [
        ...baseItems,
        { icon: AccountBalanceWallet, label: 'Wallet', path: '/wallet' },
        { icon: Shield, label: 'Protection', path: '/protection' },
      ];
    }
  };

  const navigationItems = getNavigationItems();

  const getRoleColor = () => {
    switch (user?.role) {
      case 'Admin': return theme.palette.error.main;
      case 'Trader': return theme.palette.primary.main;
      case 'Builder': return theme.palette.warning.main;
      default: return theme.palette.success.main;
    }
  };

  const drawer = (
    <Box sx={{ width: 280, height: '100%', bgcolor: theme.palette.background.paper }}>
      <Box sx={{ p: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Typography variant="h6" sx={{ fontWeight: 700 }}>
          MEV Shield
        </Typography>
        <IconButton onClick={handleDrawerToggle}>
          <Close />
        </IconButton>
      </Box>
      <Divider />
      <List>
        {navigationItems.map((item) => (
          <ListItem key={item.path} disablePadding>
            <ListItemButton
              selected={location.pathname === item.path}
              onClick={() => {
                navigate(item.path);
                setMobileOpen(false);
              }}
              sx={{
                '&.Mui-selected': {
                  bgcolor: alpha(theme.palette.primary.main, 0.1),
                  borderLeft: `3px solid ${theme.palette.primary.main}`,
                  '& .MuiListItemIcon-root': {
                    color: theme.palette.primary.main,
                  },
                },
              }}
            >
              <ListItemIcon>
                <item.icon />
              </ListItemIcon>
              <ListItemText primary={item.label} />
            </ListItemButton>
          </ListItem>
        ))}
      </List>
    </Box>
  );

  return (
    <>
      <AppBar 
        position="sticky" 
        elevation={0}
        sx={{
          background: alpha(theme.palette.background.paper, 0.8),
          backdropFilter: 'blur(20px)',
          borderBottom: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
        }}
      >
        <Toolbar sx={{ justifyContent: 'space-between' }}>
          {/* Left section */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            {isMobile && (
              <IconButton
                color="inherit"
                aria-label="open drawer"
                edge="start"
                onClick={handleDrawerToggle}
              >
                <MenuIcon />
              </IconButton>
            )}
            
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Shield 
                sx={{ 
                  fontSize: 32,
                  color: theme.palette.primary.main,
                }} 
              />
              <Typography 
                variant="h6" 
                sx={{ 
                  fontWeight: 700,
                  display: { xs: 'none', sm: 'block' },
                  background: `linear-gradient(135deg, ${theme.palette.primary.main} 0%, ${theme.palette.secondary.main} 100%)`,
                  backgroundClip: 'text',
                  WebkitBackgroundClip: 'text',
                  color: 'transparent',
                }}
              >
                MEV Shield
              </Typography>
            </Box>

            {/* Desktop navigation */}
            {!isMobile && (
              <Box sx={{ display: 'flex', gap: 1, ml: 4 }}>
                {navigationItems.map((item) => (
                  <Button
                    key={item.path}
                    startIcon={<item.icon />}
                    onClick={() => navigate(item.path)}
                    sx={{
                      color: location.pathname === item.path 
                        ? theme.palette.primary.main 
                        : theme.palette.text.secondary,
                      backgroundColor: location.pathname === item.path
                        ? alpha(theme.palette.primary.main, 0.1)
                        : 'transparent',
                      '&:hover': {
                        backgroundColor: alpha(theme.palette.primary.main, 0.1),
                      },
                    }}
                  >
                    {item.label}
                  </Button>
                ))}
              </Box>
            )}
          </Box>

          {/* Right section */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            {/* Status indicator */}
            <Chip
              icon={<Speed />}
              label="Protected"
              size="small"
              sx={{
                bgcolor: alpha(theme.palette.success.main, 0.1),
                color: theme.palette.success.main,
                border: `1px solid ${alpha(theme.palette.success.main, 0.3)}`,
                display: { xs: 'none', sm: 'flex' },
              }}
            />

            {/* Theme toggle */}
            <Tooltip title="Toggle theme">
              <IconButton onClick={onThemeToggle} size="small">
                {isDarkMode ? <LightMode /> : <DarkMode />}
              </IconButton>
            </Tooltip>

            {/* Notifications */}
            <Tooltip title="Notifications">
              <IconButton size="small">
                <Badge badgeContent={notifications} color="error">
                  <Notifications />
                </Badge>
              </IconButton>
            </Tooltip>

            {/* User menu */}
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Chip
                label={user?.role}
                size="small"
                sx={{
                  bgcolor: alpha(getRoleColor(), 0.1),
                  color: getRoleColor(),
                  border: `1px solid ${alpha(getRoleColor(), 0.3)}`,
                  fontWeight: 600,
                  display: { xs: 'none', sm: 'flex' },
                }}
              />
              <Tooltip title="Account">
                <IconButton onClick={handleProfileMenuOpen}>
                  <Avatar 
                    sx={{ 
                      width: 36, 
                      height: 36,
                      bgcolor: getRoleColor(),
                      fontSize: '0.875rem',
                      fontWeight: 600,
                    }}
                  >
                    {user?.email?.charAt(0).toUpperCase()}
                  </Avatar>
                </IconButton>
              </Tooltip>
            </Box>
          </Box>
        </Toolbar>
      </AppBar>

      {/* User menu dropdown */}
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={handleProfileMenuClose}
        PaperProps={{
          sx: {
            mt: 1,
            minWidth: 200,
            background: alpha(theme.palette.background.paper, 0.95),
            backdropFilter: 'blur(10px)',
            border: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
          },
        }}
      >
        <Box sx={{ px: 2, py: 1 }}>
          <Typography variant="subtitle2">{user?.email}</Typography>
          <Typography variant="caption" color="text.secondary">
            {user?.role} Account
          </Typography>
        </Box>
        <Divider />
        <MenuItem onClick={() => { handleProfileMenuClose(); navigate('/profile'); }}>
          <ListItemIcon>
            <Person fontSize="small" />
          </ListItemIcon>
          Profile
        </MenuItem>
        <MenuItem onClick={() => { handleProfileMenuClose(); navigate('/settings'); }}>
          <ListItemIcon>
            <Settings fontSize="small" />
          </ListItemIcon>
          Settings
        </MenuItem>
        <Divider />
        <MenuItem onClick={handleLogout}>
          <ListItemIcon>
            <ExitToApp fontSize="small" />
          </ListItemIcon>
          Logout
        </MenuItem>
      </Menu>

      {/* Mobile drawer */}
      <Drawer
        variant="temporary"
        open={mobileOpen}
        onClose={handleDrawerToggle}
        ModalProps={{
          keepMounted: true, // Better open performance on mobile
        }}
      >
        {drawer}
      </Drawer>
    </>
  );
};

export default ModernNavigation;