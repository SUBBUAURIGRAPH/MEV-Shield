import React, { ReactNode } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { Box, CircularProgress, Alert, Typography } from '@mui/material';
import { useAuth, User } from './AuthContext';

interface ProtectedRouteProps {
  children: ReactNode;
  requiredRole?: 'Admin' | 'User' | 'Validator' | 'ReadOnly';
  adminOnly?: boolean;
  fallback?: ReactNode;
  redirectTo?: string;
}

// Role hierarchy for access control
const ROLE_HIERARCHY: Record<string, number> = {
  'ReadOnly': 1,
  'User': 2,
  'Validator': 3,
  'Admin': 4,
};

// Check if user has required access level
const hasRequiredAccess = (userRole: string, requiredRole?: string, adminOnly?: boolean): boolean => {
  if (adminOnly) {
    return userRole === 'Admin';
  }
  
  if (!requiredRole) {
    return true; // No specific role required
  }
  
  const userLevel = ROLE_HIERARCHY[userRole] || 0;
  const requiredLevel = ROLE_HIERARCHY[requiredRole] || 0;
  
  return userLevel >= requiredLevel;
};

// Loading component
const LoadingScreen: React.FC = () => (
  <Box
    display="flex"
    flexDirection="column"
    justifyContent="center"
    alignItems="center"
    minHeight="400px"
    gap={2}
  >
    <CircularProgress size={48} />
    <Typography variant="h6" color="textSecondary">
      Authenticating...
    </Typography>
  </Box>
);

// Access denied component
const AccessDeniedScreen: React.FC<{ requiredRole?: string; adminOnly?: boolean }> = ({ 
  requiredRole, 
  adminOnly 
}) => (
  <Box
    display="flex"
    flexDirection="column"
    justifyContent="center"
    alignItems="center"
    minHeight="400px"
    gap={2}
    p={4}
  >
    <Alert severity="error" sx={{ width: '100%', maxWidth: 500 }}>
      <Typography variant="h6" gutterBottom>
        Access Denied
      </Typography>
      <Typography>
        You don't have the required permissions to access this page.
        {adminOnly && ' This page is restricted to administrators only.'}
        {requiredRole && !adminOnly && ` Required role: ${requiredRole} or higher.`}
      </Typography>
    </Alert>
    <Typography variant="body2" color="textSecondary">
      If you believe this is an error, please contact your administrator.
    </Typography>
  </Box>
);

// Error boundary for auth errors
interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

class AuthErrorBoundary extends React.Component<
  { children: ReactNode },
  ErrorBoundaryState
> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Auth Error Boundary caught an error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <Box
          display="flex"
          flexDirection="column"
          justifyContent="center"
          alignItems="center"
          minHeight="400px"
          gap={2}
          p={4}
        >
          <Alert severity="error" sx={{ width: '100%', maxWidth: 500 }}>
            <Typography variant="h6" gutterBottom>
              Authentication Error
            </Typography>
            <Typography>
              Something went wrong with authentication. Please try refreshing the page or logging in again.
            </Typography>
          </Alert>
        </Box>
      );
    }

    return this.props.children;
  }
}

// Main protected route component
export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requiredRole,
  adminOnly = false,
  fallback,
  redirectTo = '/login',
}) => {
  const { user, isAuthenticated, isLoading, error } = useAuth();
  const location = useLocation();

  // Show loading screen while checking authentication
  if (isLoading) {
    return <>{fallback || <LoadingScreen />}</>;
  }

  // Show error if authentication failed
  if (error) {
    return (
      <Box
        display="flex"
        flexDirection="column"
        justifyContent="center"
        alignItems="center"
        minHeight="400px"
        gap={2}
        p={4}
      >
        <Alert severity="error" sx={{ width: '100%', maxWidth: 500 }}>
          <Typography variant="h6" gutterBottom>
            Authentication Error
          </Typography>
          <Typography>{error}</Typography>
        </Alert>
      </Box>
    );
  }

  // Redirect to login if not authenticated
  if (!isAuthenticated || !user) {
    return (
      <Navigate 
        to={redirectTo} 
        state={{ from: location }} 
        replace 
      />
    );
  }

  // Check role-based access
  if (!hasRequiredAccess(user.role, requiredRole, adminOnly)) {
    return (
      <AccessDeniedScreen 
        requiredRole={requiredRole} 
        adminOnly={adminOnly} 
      />
    );
  }

  // User is authenticated and has required permissions
  return (
    <AuthErrorBoundary>
      {children}
    </AuthErrorBoundary>
  );
};

// Higher-order component version
export const withProtectedRoute = <P extends object>(
  Component: React.ComponentType<P>,
  options: Omit<ProtectedRouteProps, 'children'> = {}
) => {
  const WrappedComponent = (props: P) => (
    <ProtectedRoute {...options}>
      <Component {...props} />
    </ProtectedRoute>
  );

  WrappedComponent.displayName = `withProtectedRoute(${Component.displayName || Component.name})`;
  return WrappedComponent;
};

// Specific role-based route components
export const AdminRoute: React.FC<Omit<ProtectedRouteProps, 'adminOnly'>> = ({ children, ...props }) => (
  <ProtectedRoute {...props} adminOnly>
    {children}
  </ProtectedRoute>
);

export const UserRoute: React.FC<Omit<ProtectedRouteProps, 'requiredRole'>> = ({ children, ...props }) => (
  <ProtectedRoute {...props} requiredRole="User">
    {children}
  </ProtectedRoute>
);

export const ValidatorRoute: React.FC<Omit<ProtectedRouteProps, 'requiredRole'>> = ({ children, ...props }) => (
  <ProtectedRoute {...props} requiredRole="Validator">
    {children}
  </ProtectedRoute>
);

export const ReadOnlyRoute: React.FC<Omit<ProtectedRouteProps, 'requiredRole'>> = ({ children, ...props }) => (
  <ProtectedRoute {...props} requiredRole="ReadOnly">
    {children}
  </ProtectedRoute>
);

// Hook for checking permissions in components (re-exported from AuthContext)
export { usePermissions } from './AuthContext';

export default ProtectedRoute;