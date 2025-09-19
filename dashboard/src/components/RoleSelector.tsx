import React from 'react';
import { useNavigate } from 'react-router-dom';
import { 
  Box, 
  Button, 
  Card, 
  CardContent, 
  Typography, 
  Grid,
  Container 
} from '@mui/material';
import {
  AdminPanelSettings,
  Person,
  Build,
  TrendingUp,
  Security
} from '@mui/icons-material';
import { useAuth } from '../auth/AuthContext';

const RoleSelector: React.FC = () => {
  const navigate = useNavigate();
  const { user } = useAuth();

  const roles = [
    {
      title: 'Admin Dashboard',
      description: 'Full system control and monitoring',
      icon: <AdminPanelSettings sx={{ fontSize: 40 }} />,
      path: '/admin',
      color: 'error',
      role: 'Admin'
    },
    {
      title: 'User Dashboard',
      description: 'Standard user interface and features',
      icon: <Person sx={{ fontSize: 40 }} />,
      path: '/dashboard',
      color: 'primary',
      role: 'User'
    },
    {
      title: 'Builder Dashboard',
      description: 'Block building and MEV bundle management',
      icon: <Build sx={{ fontSize: 40 }} />,
      path: '/builder',
      color: 'warning',
      role: 'Builder'
    },
    {
      title: 'Trader Dashboard',
      description: 'Trading interface with MEV protection',
      icon: <TrendingUp sx={{ fontSize: 40 }} />,
      path: '/trader',
      color: 'success',
      role: 'Trader'
    },
  ];

  return (
    <Container maxWidth="lg">
      <Box sx={{ py: 5 }}>
        <Typography variant="h3" align="center" gutterBottom sx={{ fontWeight: 700 }}>
          MEV Shield Dashboard Selector
        </Typography>
        <Typography variant="h6" align="center" color="text.secondary" sx={{ mb: 4 }}>
          Current Role: {user?.role || 'Not authenticated'}
        </Typography>
        
        <Grid container spacing={3}>
          {roles.map((role) => (
            <Grid item xs={12} sm={6} md={3} key={role.title}>
              <Card 
                sx={{ 
                  height: '100%',
                  cursor: 'pointer',
                  transition: 'all 0.3s ease',
                  '&:hover': {
                    transform: 'translateY(-5px)',
                    boxShadow: 4,
                  }
                }}
                onClick={() => navigate(role.path)}
              >
                <CardContent sx={{ textAlign: 'center', p: 3 }}>
                  <Box 
                    sx={{ 
                      color: `${role.color}.main`,
                      mb: 2,
                      display: 'flex',
                      justifyContent: 'center'
                    }}
                  >
                    {role.icon}
                  </Box>
                  <Typography variant="h6" gutterBottom sx={{ fontWeight: 600 }}>
                    {role.title}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                    {role.description}
                  </Typography>
                  <Button 
                    variant="contained" 
                    color={role.color as any}
                    fullWidth
                    onClick={(e) => {
                      e.stopPropagation();
                      navigate(role.path);
                    }}
                  >
                    Open {role.role} View
                  </Button>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
        
        <Box sx={{ mt: 4, textAlign: 'center' }}>
          <Typography variant="body2" color="text.secondary">
            Note: Access to dashboards depends on your authenticated role.
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Some dashboards may be restricted based on permissions.
          </Typography>
        </Box>
      </Box>
    </Container>
  );
};

export default RoleSelector;