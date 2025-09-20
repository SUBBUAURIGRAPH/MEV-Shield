import React, { useState } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ThemeProvider, CssBaseline, createTheme } from '@mui/material';
import { AuthProvider, useAuth } from './auth/AuthContext';
import { ProtectedRoute, AdminRoute, UserRoute, BuilderRoute, TraderRoute } from './auth/ProtectedRoute';
import ModernLoginPage from './auth/ModernLoginPage';
import AdminDashboard from './dashboards/AdminDashboard';
import ImprovedAdminDashboard from './dashboards/ImprovedAdminDashboard';
import UserDashboard from './dashboards/UserDashboard';
import BuilderDashboard from './dashboards/BuilderDashboard';
import TraderDashboard from './dashboards/TraderDashboard';
import RoleSelector from './components/RoleSelector';
import ModernNavigation from './components/ModernNavigation';
import darkTheme from './theme/theme';
import SecurityPage from './pages/SecurityPage';
import WalletPage from './pages/WalletPage';
import SettingsPage from './pages/SettingsPage';
import DEXProtection from './components/DEXProtection';

// Light theme variant
const lightTheme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#00A3CC',
      light: '#00D4FF',
      dark: '#007399',
    },
    secondary: {
      main: '#5841CC',
      light: '#7B61FF',
      dark: '#4030A0',
    },
    success: {
      main: '#00CC6A',
    },
    error: {
      main: '#CC3946',
    },
    warning: {
      main: '#CC9300',
    },
    background: {
      default: '#F5F7FA',
      paper: '#FFFFFF',
    },
  },
  typography: {
    fontFamily: '"Inter", "SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    h1: {
      fontWeight: 800,
    },
    h6: {
      fontWeight: 600,
    },
  },
  shape: {
    borderRadius: 12,
  },
  components: {
    MuiCard: {
      styleOverrides: {
        root: {
          boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
          transition: 'all 0.3s ease',
          '&:hover': {
            boxShadow: '0 8px 12px rgba(0, 0, 0, 0.15)',
          },
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: 'none',
          fontWeight: 500,
          borderRadius: 8,
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          fontWeight: 500,
        },
      },
    },
  },
});

// Component to handle role-based routing
function RoleBasedRoute() {
  const { user } = useAuth();
  
  // Redirect based on user role
  if (!user) return <Navigate to="/login" replace />;
  
  switch(user.role) {
    case 'Admin':
      return <Navigate to="/admin" replace />;
    case 'Builder':
      return <Navigate to="/builder" replace />;
    case 'Trader':
      return <Navigate to="/trader" replace />;
    case 'User':
    default:
      return <Navigate to="/dashboard" replace />;
  }
}

function App() {
  const [isDarkMode, setIsDarkMode] = useState(true);
  const currentTheme = isDarkMode ? darkTheme : lightTheme;
  
  const handleThemeToggle = () => {
    setIsDarkMode(!isDarkMode);
  };

  return (
    <ThemeProvider theme={currentTheme}>
      <CssBaseline />
      <AuthProvider>
        <Router>
          <Routes>
            {/* Public routes */}
            <Route path="/login" element={<ModernLoginPage />} />
            
            {/* Protected routes */}
            <Route 
              path="/" 
              element={
                <ProtectedRoute>
                  <RoleBasedRoute />
                </ProtectedRoute>
              } 
            />
            
            {/* Admin dashboard - requires admin role */}
            <Route 
              path="/admin" 
              element={
                <AdminRoute>
                  <ImprovedAdminDashboard />
                </AdminRoute>
              } 
            />
            
            {/* User dashboard - requires user role or higher */}
            <Route 
              path="/dashboard" 
              element={
                <UserRoute>
                  <UserDashboard />
                </UserRoute>
              } 
            />
            
            {/* Builder dashboard */}
            <Route 
              path="/builder" 
              element={
                <BuilderRoute>
                  <BuilderDashboard />
                </BuilderRoute>
              } 
            />
            
            {/* Trader dashboard */}
            <Route 
              path="/trader" 
              element={
                <TraderRoute>
                  <TraderDashboard />
                </TraderRoute>
              } 
            />
            
            {/* Role selector for testing */}
            <Route 
              path="/roles" 
              element={
                <ProtectedRoute>
                  <RoleSelector />
                </ProtectedRoute>
              } 
            />
            
            {/* Legacy user route */}
            <Route 
              path="/user" 
              element={
                <UserRoute>
                  <UserDashboard />
                </UserRoute>
              } 
            />
            
            {/* Common pages accessible to all authenticated users */}
            <Route 
              path="/wallet" 
              element={
                <ProtectedRoute>
                  <WalletPage />
                </ProtectedRoute>
              } 
            />
            
            <Route 
              path="/settings" 
              element={
                <ProtectedRoute>
                  <SettingsPage />
                </ProtectedRoute>
              } 
            />
            
            <Route 
              path="/security" 
              element={
                <AdminRoute>
                  <SecurityPage />
                </AdminRoute>
              } 
            />
            
            <Route 
              path="/protection" 
              element={
                <ProtectedRoute>
                  <DEXProtection />
                </ProtectedRoute>
              } 
            />
            
            {/* Redirect common navigation paths to appropriate dashboards */}
            <Route path="/analytics" element={<Navigate to="/admin" replace />} />
            <Route path="/users" element={<Navigate to="/admin" replace />} />
            <Route path="/trading" element={<Navigate to="/trader" replace />} />
            <Route path="/portfolio" element={<Navigate to="/trader" replace />} />
            <Route path="/builds" element={<Navigate to="/builder" replace />} />
            <Route path="/performance" element={<Navigate to="/builder" replace />} />
            <Route path="/metrics" element={<Navigate to="/builder" replace />} />
            
            {/* Profile page redirects to settings */}
            <Route path="/profile" element={<Navigate to="/settings" replace />} />
            
            {/* Catch all - redirect to login */}
            <Route path="*" element={<Navigate to="/login" replace />} />
          </Routes>
        </Router>
      </AuthProvider>
    </ThemeProvider>
  );
}

export default App;