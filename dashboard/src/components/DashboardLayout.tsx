import React from 'react';
import { Box, Container } from '@mui/material';
import ModernNavigation from './ModernNavigation';

interface DashboardLayoutProps {
  children: React.ReactNode;
  onThemeToggle?: () => void;
  isDarkMode?: boolean;
  maxWidth?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | false;
  disablePadding?: boolean;
}

const DashboardLayout: React.FC<DashboardLayoutProps> = ({ 
  children, 
  onThemeToggle, 
  isDarkMode = true,
  maxWidth = 'xl',
  disablePadding = false 
}) => {
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <ModernNavigation onThemeToggle={onThemeToggle} isDarkMode={isDarkMode} />
      <Box sx={{ flex: 1, py: disablePadding ? 0 : 3 }}>
        <Container maxWidth={maxWidth} sx={{ height: '100%' }}>
          {children}
        </Container>
      </Box>
    </Box>
  );
};

export default DashboardLayout;