import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import axios, { AxiosResponse } from 'axios';

// Types
interface User {
  id: string;
  email: string;
  role: 'Admin' | 'User' | 'Builder' | 'Trader' | 'Validator' | 'ReadOnly';
  lastLogin?: string;
}

interface AuthContextType {
  user: User | null;
  token: string | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  register: (email: string, password: string, role?: string) => Promise<void>;
  changePassword: (currentPassword: string, newPassword: string) => Promise<void>;
  requestPasswordReset: (email: string) => Promise<void>;
  isAuthenticated: boolean;
  isAdmin: boolean;
  isLoading: boolean;
  error: string | null;
  clearError: () => void;
}

interface LoginResponse {
  success: boolean;
  data?: {
    access_token: string;
    refresh_token: string;
    token_type: string;
    expires_in: number;
    user: User;
  };
  error?: string;
}

interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// Create context
const AuthContext = createContext<AuthContextType | undefined>(undefined);

// Auth provider props
interface AuthProviderProps {
  children: ReactNode;
}

// API base URL
const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';

// Configure axios defaults
axios.defaults.baseURL = API_BASE_URL;
// Don't use withCredentials for JWT-based auth
axios.defaults.withCredentials = false;

// Auth provider component
export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Clear error
  const clearError = () => setError(null);

  // Set auth header for axios
  const setAuthHeader = (token: string | null) => {
    if (token) {
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`;
    } else {
      delete axios.defaults.headers.common['Authorization'];
    }
  };

  // Initialize auth from localStorage
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        const savedToken = localStorage.getItem('mev_shield_token');
        const savedUser = localStorage.getItem('mev_shield_user');

        if (savedToken && savedUser) {
          setToken(savedToken);
          setUser(JSON.parse(savedUser));
          setAuthHeader(savedToken);

          // Verify token is still valid
          try {
            const response: AxiosResponse<ApiResponse<User>> = await axios.get('/auth/me');
            if (response.data.success && response.data.data) {
              setUser(response.data.data);
            } else {
              // Token is invalid, clear auth
              handleLogout();
            }
          } catch (error) {
            // Token is invalid, clear auth
            handleLogout();
          }
        }
      } catch (error) {
        console.error('Auth initialization error:', error);
        handleLogout();
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();
  }, []);

  // Set up axios interceptor for token refresh
  useEffect(() => {
    const interceptor = axios.interceptors.response.use(
      (response) => response,
      async (error) => {
        const originalRequest = error.config;

        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;

          try {
            // Try to refresh token
            const response: AxiosResponse<ApiResponse<{ access_token: string }>> = 
              await axios.post('/auth/refresh', {
                refresh_token: localStorage.getItem('mev_shield_refresh_token')
              });

            if (response.data.success && response.data.data) {
              const newToken = response.data.data.access_token;
              setToken(newToken);
              setAuthHeader(newToken);
              localStorage.setItem('mev_shield_token', newToken);

              // Retry original request with new token
              originalRequest.headers['Authorization'] = `Bearer ${newToken}`;
              return axios(originalRequest);
            }
          } catch (refreshError) {
            // Refresh failed, logout user
            handleLogout();
          }
        }

        return Promise.reject(error);
      }
    );

    return () => {
      axios.interceptors.response.eject(interceptor);
    };
  }, []);

  // Handle logout
  const handleLogout = () => {
    setUser(null);
    setToken(null);
    setAuthHeader(null);
    localStorage.removeItem('mev_shield_token');
    localStorage.removeItem('mev_shield_refresh_token');
    localStorage.removeItem('mev_shield_user');
  };

  // Login function
  const login = async (email: string, password: string): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);

      const response: AxiosResponse<LoginResponse> = await axios.post('/auth/login', {
        email,
        password,
      });

      if (response.data.success && response.data.data) {
        const { access_token, refresh_token, user: userData } = response.data.data;
        
        setToken(access_token);
        setUser(userData);
        setAuthHeader(access_token);
        
        // Store in localStorage
        localStorage.setItem('mev_shield_token', access_token);
        localStorage.setItem('mev_shield_refresh_token', refresh_token);
        localStorage.setItem('mev_shield_user', JSON.stringify(userData));
      } else {
        throw new Error(response.data.error || 'Login failed');
      }
    } catch (error: any) {
      const message = error.response?.data?.error || error.message || 'Login failed';
      setError(message);
      throw new Error(message);
    } finally {
      setIsLoading(false);
    }
  };

  // Logout function
  const logout = async (): Promise<void> => {
    try {
      // Call logout endpoint to blacklist token
      await axios.post('/auth/logout');
    } catch (error) {
      console.error('Logout API error:', error);
    } finally {
      handleLogout();
    }
  };

  // Register function
  const register = async (email: string, password: string, role = 'User'): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);

      const response: AxiosResponse<ApiResponse> = await axios.post('/auth/register', {
        email,
        password,
        role,
      });

      if (!response.data.success) {
        throw new Error(response.data.error || 'Registration failed');
      }
    } catch (error: any) {
      const message = error.response?.data?.error || error.message || 'Registration failed';
      setError(message);
      throw new Error(message);
    } finally {
      setIsLoading(false);
    }
  };

  // Change password function
  const changePassword = async (currentPassword: string, newPassword: string): Promise<void> => {
    try {
      setError(null);

      const response: AxiosResponse<ApiResponse> = await axios.post('/auth/change-password', {
        current_password: currentPassword,
        new_password: newPassword,
      });

      if (!response.data.success) {
        throw new Error(response.data.error || 'Password change failed');
      }
    } catch (error: any) {
      const message = error.response?.data?.error || error.message || 'Password change failed';
      setError(message);
      throw new Error(message);
    }
  };

  // Request password reset function
  const requestPasswordReset = async (email: string): Promise<void> => {
    try {
      setError(null);

      const response: AxiosResponse<ApiResponse> = await axios.post('/auth/reset-password', {
        email,
      });

      if (!response.data.success) {
        throw new Error(response.data.error || 'Password reset request failed');
      }
    } catch (error: any) {
      const message = error.response?.data?.error || error.message || 'Password reset request failed';
      setError(message);
      throw new Error(message);
    }
  };

  // Computed values
  const isAuthenticated = !!user && !!token;
  const isAdmin = user?.role === 'Admin';

  const value: AuthContextType = {
    user,
    token,
    login,
    logout,
    register,
    changePassword,
    requestPasswordReset,
    isAuthenticated,
    isAdmin,
    isLoading,
    error,
    clearError,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};

// Custom hook to use auth context
export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

// Hook for checking permissions in components
export const usePermissions = () => {
  const { user, isAuthenticated } = useAuth();

  // Role hierarchy for access control
  const ROLE_HIERARCHY: Record<string, number> = {
    'ReadOnly': 1,
    'User': 2,
    'Trader': 2,
    'Builder': 3,
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

  const checkPermission = (requiredRole?: string, adminOnly?: boolean): boolean => {
    if (!isAuthenticated || !user) {
      return false;
    }
    
    return hasRequiredAccess(user.role, requiredRole, adminOnly);
  };

  return {
    isAuthenticated,
    user,
    canAccess: checkPermission,
    isAdmin: user?.role === 'Admin',
    isUser: user?.role === 'User' || user?.role === 'Admin',
    isBuilder: user?.role === 'Builder' || user?.role === 'Admin',
    isTrader: user?.role === 'Trader' || user?.role === 'Admin',
    isValidator: user?.role === 'Validator' || user?.role === 'Admin',
    isReadOnly: !!user, // All authenticated users can read
  };
};

// Export types for external use
export type { User, AuthContextType };