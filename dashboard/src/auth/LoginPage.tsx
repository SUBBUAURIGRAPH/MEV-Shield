import React, { useState } from 'react';
import {
  Box,
  Paper,
  TextField,
  Button,
  Typography,
  Alert,
  IconButton,
  InputAdornment,
  Link,
  Divider,
  CircularProgress,
  Container,
} from '@mui/material';
import {
  Visibility,
  VisibilityOff,
  Security as SecurityIcon,
  Login as LoginIcon,
} from '@mui/icons-material';
import { useNavigate, useLocation } from 'react-router-dom';
import { useLoginForm } from './useAuth';

interface LocationState {
  from?: {
    pathname: string;
  };
}

export const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const state = location.state as LocationState;
  
  const {
    email,
    setEmail,
    password,
    setPassword,
    showPassword,
    togglePasswordVisibility,
    handleLogin,
    resetForm,
    isLoading,
    error,
    clearError,
  } = useLoginForm();

  const [rememberMe, setRememberMe] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    try {
      await handleLogin();
      
      // Redirect to the page user was trying to access, or to dashboard
      const redirectTo = state?.from?.pathname || '/dashboard';
      navigate(redirectTo, { replace: true });
    } catch (error) {
      // Error is handled by the useLoginForm hook
      console.error('Login error:', error);
    }
  };

  const handleForgotPassword = () => {
    navigate('/forgot-password');
  };

  return (
    <Container maxWidth="sm" sx={{ minHeight: '100vh', display: 'flex', alignItems: 'center' }}>
      <Paper
        elevation={8}
        sx={{
          p: 4,
          width: '100%',
          borderRadius: 3,
          background: 'linear-gradient(135deg, #ffffff 0%, #f8f9fa 100%)',
        }}
      >
        {/* Header */}
        <Box textAlign="center" mb={4}>
          <Box
            sx={{
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: 64,
              height: 64,
              borderRadius: '50%',
              background: 'linear-gradient(135deg, #4CAF50 0%, #2196F3 100%)',
              mb: 2,
            }}
          >
            <SecurityIcon sx={{ color: 'white', fontSize: 32 }} />
          </Box>
          <Typography variant="h4" gutterBottom sx={{ fontWeight: 600 }}>
            MEV Shield
          </Typography>
          <Typography variant="subtitle1" color="textSecondary">
            Sign in to your account
          </Typography>
        </Box>

        {/* Error Alert */}
        {error && (
          <Alert 
            severity="error" 
            onClose={clearError}
            sx={{ mb: 3 }}
          >
            {error}
          </Alert>
        )}

        {/* Redirect Notice */}
        {state?.from && (
          <Alert severity="info" sx={{ mb: 3 }}>
            Please log in to access {state.from.pathname}
          </Alert>
        )}

        {/* Login Form */}
        <Box component="form" onSubmit={handleSubmit}>
          <TextField
            fullWidth
            label="Email Address"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            disabled={isLoading}
            required
            autoComplete="email"
            autoFocus
            sx={{ mb: 3 }}
            error={!!error && !email}
          />

          <TextField
            fullWidth
            label="Password"
            type={showPassword ? 'text' : 'password'}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={isLoading}
            required
            autoComplete="current-password"
            sx={{ mb: 2 }}
            error={!!error && !password}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton
                    onClick={togglePasswordVisibility}
                    edge="end"
                    disabled={isLoading}
                  >
                    {showPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />

          {/* Forgot Password Link */}
          <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
            <Box />
            <Link
              component="button"
              type="button"
              variant="body2"
              onClick={handleForgotPassword}
              disabled={isLoading}
              sx={{ textDecoration: 'none' }}
            >
              Forgot password?
            </Link>
          </Box>

          {/* Login Button */}
          <Button
            type="submit"
            fullWidth
            variant="contained"
            size="large"
            disabled={isLoading || !email || !password}
            startIcon={isLoading ? <CircularProgress size={20} /> : <LoginIcon />}
            sx={{
              mb: 2,
              py: 1.5,
              background: 'linear-gradient(135deg, #4CAF50 0%, #2196F3 100%)',
              '&:hover': {
                background: 'linear-gradient(135deg, #45a049 0%, #1976D2 100%)',
              },
            }}
          >
            {isLoading ? 'Signing In...' : 'Sign In'}
          </Button>

          <Divider sx={{ my: 3 }}>
            <Typography variant="body2" color="textSecondary">
              or
            </Typography>
          </Divider>

          {/* Demo Account Info */}
          <Alert severity="info" sx={{ mb: 2 }}>
            <Typography variant="body2" gutterBottom>
              <strong>Demo Account:</strong>
            </Typography>
            <Typography variant="body2">
              Email: admin@mevshield.com<br />
              Password: AdminPassword123!
            </Typography>
          </Alert>

          {/* Register Link */}
          <Box textAlign="center">
            <Typography variant="body2" color="textSecondary">
              Don't have an account?{' '}
              <Link
                component="button"
                type="button"
                variant="body2"
                onClick={() => navigate('/register')}
                disabled={isLoading}
                sx={{ fontWeight: 500 }}
              >
                Contact Administrator
              </Link>
            </Typography>
          </Box>
        </Box>

        {/* Footer */}
        <Box mt={4} pt={3} borderTop="1px solid #e0e0e0">
          <Typography variant="caption" color="textSecondary" align="center" display="block">
            MEV Shield © 2024 Aurigraph DLT. All rights reserved.
          </Typography>
        </Box>
      </Paper>
    </Container>
  );
};

export default LoginPage;