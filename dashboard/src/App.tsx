import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ThemeProvider, createTheme, CssBaseline } from '@mui/material';
import { AuthProvider, useAuth } from './auth/AuthContext';
import { ProtectedRoute, AdminRoute, UserRoute, BuilderRoute, TraderRoute } from './auth/ProtectedRoute';
import LoginPage from './auth/LoginPage';
import AdminDashboard from './dashboards/AdminDashboard';
import ImprovedAdminDashboard from './dashboards/ImprovedAdminDashboard';
import UserDashboard from './dashboards/UserDashboard';
import BuilderDashboard from './dashboards/BuilderDashboard';
import TraderDashboard from './dashboards/TraderDashboard';
import RoleSelector from './components/RoleSelector';

const theme = createTheme({
  palette: {
    primary: {
      main: '#4CAF50',
    },
    secondary: {
      main: '#2196F3',
    },
    success: {
      main: '#4CAF50',
    },
    error: {
      main: '#f44336',
    },
    warning: {
      main: '#ff9800',
    },
    background: {
      default: '#f5f5f5',
    },
  },
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    h1: {
      fontWeight: 600,
    },
    h2: {
      fontWeight: 600,
    },
    h3: {
      fontWeight: 600,
    },
    h4: {
      fontWeight: 600,
    },
    h5: {
      fontWeight: 600,
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
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <AuthProvider>
        <Router>
          <Routes>
            {/* Public routes */}
            <Route path="/login" element={<LoginPage />} />
            
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
            
            {/* Catch all - redirect to login */}
            <Route path="*" element={<Navigate to="/login" replace />} />
          </Routes>
        </Router>
      </AuthProvider>
    </ThemeProvider>
  );
}

export default App;