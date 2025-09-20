import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Box,
  TextField,
  Button,
  Typography,
  Container,
  Paper,
  Alert,
  CircularProgress,
  IconButton,
  InputAdornment,
  Fade,
  Slide,
  useTheme,
  alpha,
  Checkbox,
  FormControlLabel,
  Link,
} from '@mui/material';
import {
  Visibility,
  VisibilityOff,
  Shield,
  Lock,
  Email,
  ArrowForward,
  TrendingUp,
  Security,
  Speed,
} from '@mui/icons-material';
import { useAuth } from './AuthContext';
import { keyframes } from '@mui/system';

// Custom animations
const float = keyframes`
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-20px); }
`;

const glow = keyframes`
  0%, 100% { box-shadow: 0 0 20px rgba(0, 212, 255, 0.4); }
  50% { box-shadow: 0 0 40px rgba(0, 212, 255, 0.8); }
`;

const slideInLeft = keyframes`
  from { transform: translateX(-100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
`;

const slideInRight = keyframes`
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
`;

const ModernLoginPage: React.FC = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [rememberMe, setRememberMe] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedRole, setSelectedRole] = useState<string | null>(null);
  
  const { login } = useAuth();
  const navigate = useNavigate();
  const theme = useTheme();

  // Pre-fill credentials based on role selection
  const handleQuickLogin = (role: string) => {
    setSelectedRole(role);
    switch(role) {
      case 'Admin':
        setEmail('admin@mevshield.ai');
        setPassword('admin123');
        break;
      case 'User':
        setEmail('user@mevshield.ai');
        setPassword('user123');
        break;
      case 'Builder':
        setEmail('builder@mevshield.ai');
        setPassword('builder123');
        break;
      case 'Trader':
        setEmail('trader@mevshield.ai');
        setPassword('trader123');
        break;
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsLoading(true);
    
    try {
      await login(email, password);
      navigate('/dashboard');
    } catch (err: any) {
      setError(err.message || 'Login failed. Please try again.');
      setIsLoading(false);
    }
  };

  // Features showcase
  const features = [
    { icon: Shield, title: 'MEV Protection', description: 'Advanced protection against sandwich attacks' },
    { icon: Speed, title: 'Fast Execution', description: 'Lightning-fast transaction processing' },
    { icon: Security, title: 'Secure Platform', description: 'Enterprise-grade security standards' },
    { icon: TrendingUp, title: 'Real-time Analytics', description: 'Monitor your transactions in real-time' },
  ];

  return (
    <Box
      sx={{
        minHeight: '100vh',
        display: 'flex',
        background: `linear-gradient(135deg, ${theme.palette.background.default} 0%, ${alpha(theme.palette.primary.dark, 0.1)} 100%)`,
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      {/* Animated background elements */}
      <Box
        sx={{
          position: 'absolute',
          top: '-10%',
          left: '-10%',
          width: '300px',
          height: '300px',
          borderRadius: '50%',
          background: `radial-gradient(circle, ${alpha(theme.palette.primary.main, 0.3)} 0%, transparent 70%)`,
          animation: `${float} 6s ease-in-out infinite`,
          filter: 'blur(40px)',
        }}
      />
      <Box
        sx={{
          position: 'absolute',
          bottom: '-10%',
          right: '-10%',
          width: '400px',
          height: '400px',
          borderRadius: '50%',
          background: `radial-gradient(circle, ${alpha(theme.palette.secondary.main, 0.3)} 0%, transparent 70%)`,
          animation: `${float} 8s ease-in-out infinite`,
          animationDelay: '2s',
          filter: 'blur(40px)',
        }}
      />

      <Container maxWidth="lg">
        <Box sx={{ minHeight: '100vh', display: 'flex', alignItems: 'center', py: 4 }}>
          <Box sx={{ width: '100%', display: 'flex', gap: 4 }}>
            {/* Left side - Features */}
            <Box 
              sx={{ 
                flex: 1,
                display: { xs: 'none', md: 'flex' },
                flexDirection: 'column',
                justifyContent: 'center',
                animation: `${slideInLeft} 0.8s ease-out`,
              }}
            >
              <Typography 
                variant="h2" 
                sx={{ 
                  mb: 2,
                  fontWeight: 800,
                  background: `linear-gradient(135deg, ${theme.palette.primary.main} 0%, ${theme.palette.secondary.main} 100%)`,
                  backgroundClip: 'text',
                  WebkitBackgroundClip: 'text',
                  color: 'transparent',
                }}
              >
                MEV Shield
              </Typography>
              <Typography variant="h5" color="text.secondary" sx={{ mb: 4 }}>
                Protect your trades from MEV attacks with advanced blockchain technology
              </Typography>
              
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
                {features.map((feature, index) => (
                  <Fade in timeout={1000 + index * 200} key={feature.title}>
                    <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
                      <Box
                        sx={{
                          width: 50,
                          height: 50,
                          borderRadius: 2,
                          background: alpha(theme.palette.primary.main, 0.1),
                          border: `1px solid ${alpha(theme.palette.primary.main, 0.3)}`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                      >
                        <feature.icon sx={{ color: theme.palette.primary.main }} />
                      </Box>
                      <Box>
                        <Typography variant="h6" sx={{ fontWeight: 600 }}>
                          {feature.title}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {feature.description}
                        </Typography>
                      </Box>
                    </Box>
                  </Fade>
                ))}
              </Box>
            </Box>

            {/* Right side - Login form */}
            <Box 
              sx={{ 
                flex: 1,
                maxWidth: 500,
                mx: 'auto',
                animation: `${slideInRight} 0.8s ease-out`,
              }}
            >
              <Paper
                elevation={0}
                sx={{
                  p: 4,
                  borderRadius: 4,
                  background: alpha(theme.palette.background.paper, 0.8),
                  backdropFilter: 'blur(20px)',
                  border: `1px solid ${alpha(theme.palette.primary.main, 0.1)}`,
                  animation: `${glow} 3s ease-in-out infinite`,
                  transition: 'all 0.3s ease',
                  '&:hover': {
                    transform: 'translateY(-5px)',
                    boxShadow: `0 20px 40px ${alpha(theme.palette.primary.main, 0.2)}`,
                  },
                }}
              >
                <Box sx={{ textAlign: 'center', mb: 3 }}>
                  <Shield 
                    sx={{ 
                      fontSize: 60, 
                      color: theme.palette.primary.main,
                      mb: 2,
                    }} 
                  />
                  <Typography variant="h4" sx={{ fontWeight: 700, mb: 1 }}>
                    Welcome Back
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Sign in to access your MEV Shield dashboard
                  </Typography>
                </Box>

                {/* Quick login buttons */}
                <Box sx={{ mb: 3 }}>
                  <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>
                    Quick Login (Demo)
                  </Typography>
                  <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
                    {['Admin', 'User', 'Builder', 'Trader'].map((role) => (
                      <Button
                        key={role}
                        size="small"
                        variant={selectedRole === role ? "contained" : "outlined"}
                        onClick={() => handleQuickLogin(role)}
                        sx={{
                          borderRadius: 2,
                          textTransform: 'none',
                          fontSize: '0.75rem',
                        }}
                      >
                        {role}
                      </Button>
                    ))}
                  </Box>
                </Box>

                <form onSubmit={handleSubmit}>
                  <TextField
                    fullWidth
                    label="Email"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    sx={{ mb: 2 }}
                    InputProps={{
                      startAdornment: (
                        <InputAdornment position="start">
                          <Email sx={{ color: theme.palette.primary.main }} />
                        </InputAdornment>
                      ),
                    }}
                  />
                  
                  <TextField
                    fullWidth
                    label="Password"
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    sx={{ mb: 1 }}
                    InputProps={{
                      startAdornment: (
                        <InputAdornment position="start">
                          <Lock sx={{ color: theme.palette.primary.main }} />
                        </InputAdornment>
                      ),
                      endAdornment: (
                        <InputAdornment position="end">
                          <IconButton
                            onClick={() => setShowPassword(!showPassword)}
                            edge="end"
                            size="small"
                          >
                            {showPassword ? <VisibilityOff /> : <Visibility />}
                          </IconButton>
                        </InputAdornment>
                      ),
                    }}
                  />

                  <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
                    <FormControlLabel
                      control={
                        <Checkbox 
                          checked={rememberMe}
                          onChange={(e) => setRememberMe(e.target.checked)}
                          sx={{ 
                            '&.Mui-checked': {
                              color: theme.palette.primary.main,
                            }
                          }}
                        />
                      }
                      label="Remember me"
                    />
                    <Link
                      href="#"
                      sx={{
                        color: theme.palette.primary.main,
                        textDecoration: 'none',
                        '&:hover': {
                          textDecoration: 'underline',
                        },
                      }}
                    >
                      Forgot password?
                    </Link>
                  </Box>

                  {error && (
                    <Slide direction="down" in={!!error}>
                      <Alert 
                        severity="error" 
                        sx={{ mb: 2 }}
                        onClose={() => setError(null)}
                      >
                        {error}
                      </Alert>
                    </Slide>
                  )}

                  <Button
                    fullWidth
                    size="large"
                    type="submit"
                    variant="contained"
                    disabled={isLoading || !email || !password}
                    sx={{
                      py: 1.5,
                      fontSize: '1rem',
                      fontWeight: 600,
                      position: 'relative',
                      overflow: 'hidden',
                      '&::before': {
                        content: '""',
                        position: 'absolute',
                        top: 0,
                        left: '-100%',
                        width: '100%',
                        height: '100%',
                        background: `linear-gradient(90deg, transparent, ${alpha(theme.palette.common.white, 0.2)}, transparent)`,
                        transition: 'left 0.5s',
                      },
                      '&:hover::before': {
                        left: '100%',
                      },
                    }}
                    endIcon={isLoading ? null : <ArrowForward />}
                  >
                    {isLoading ? (
                      <CircularProgress size={24} color="inherit" />
                    ) : (
                      'Sign In'
                    )}
                  </Button>

                  <Box sx={{ textAlign: 'center', mt: 3 }}>
                    <Typography variant="body2" color="text.secondary">
                      Don't have an account?{' '}
                      <Link
                        href="#"
                        sx={{
                          color: theme.palette.primary.main,
                          textDecoration: 'none',
                          fontWeight: 600,
                          '&:hover': {
                            textDecoration: 'underline',
                          },
                        }}
                      >
                        Sign up
                      </Link>
                    </Typography>
                  </Box>
                </form>
              </Paper>
            </Box>
          </Box>
        </Box>
      </Container>
    </Box>
  );
};

export default ModernLoginPage;