import { useState, useCallback } from 'react';
import { useAuth } from './AuthContext';

// Re-export the useAuth hook from AuthContext for convenience
export { useAuth, usePermissions } from './AuthContext';
export type { User, AuthContextType } from './AuthContext';

// Additional auth-related hooks and utilities

// Hook for handling login form state
export const useLoginForm = () => {
  const { login, isLoading, error, clearError } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);

  const handleLogin = useCallback(async () => {
    if (!email || !password) {
      throw new Error('Email and password are required');
    }
    
    await login(email, password);
  }, [email, password, login]);

  const resetForm = useCallback(() => {
    setEmail('');
    setPassword('');
    setShowPassword(false);
    clearError();
  }, [clearError]);

  const togglePasswordVisibility = useCallback(() => {
    setShowPassword(prev => !prev);
  }, []);

  return {
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
  };
};

// Hook for handling registration form state
export const useRegistrationForm = () => {
  const { register, isLoading, error, clearError } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [role, setRole] = useState('User');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);

  const handleRegister = useCallback(async () => {
    if (!email || !password || !confirmPassword) {
      throw new Error('All fields are required');
    }

    if (password !== confirmPassword) {
      throw new Error('Passwords do not match');
    }

    await register(email, password, role);
  }, [email, password, confirmPassword, role, register]);

  const resetForm = useCallback(() => {
    setEmail('');
    setPassword('');
    setConfirmPassword('');
    setRole('User');
    setShowPassword(false);
    setShowConfirmPassword(false);
    clearError();
  }, [clearError]);

  const togglePasswordVisibility = useCallback(() => {
    setShowPassword(prev => !prev);
  }, []);

  const toggleConfirmPasswordVisibility = useCallback(() => {
    setShowConfirmPassword(prev => !prev);
  }, []);

  const passwordsMatch = password === confirmPassword;
  const isValid = email && password && confirmPassword && passwordsMatch;

  return {
    email,
    setEmail,
    password,
    setPassword,
    confirmPassword,
    setConfirmPassword,
    role,
    setRole,
    showPassword,
    togglePasswordVisibility,
    showConfirmPassword,
    toggleConfirmPasswordVisibility,
    handleRegister,
    resetForm,
    passwordsMatch,
    isValid,
    isLoading,
    error,
    clearError,
  };
};

// Hook for handling password change form state
export const usePasswordChangeForm = () => {
  const { changePassword, error, clearError } = useAuth();
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmNewPassword, setConfirmNewPassword] = useState('');
  const [showCurrentPassword, setShowCurrentPassword] = useState(false);
  const [showNewPassword, setShowNewPassword] = useState(false);
  const [showConfirmNewPassword, setShowConfirmNewPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const handlePasswordChange = useCallback(async () => {
    if (!currentPassword || !newPassword || !confirmNewPassword) {
      throw new Error('All fields are required');
    }

    if (newPassword !== confirmNewPassword) {
      throw new Error('New passwords do not match');
    }

    if (newPassword === currentPassword) {
      throw new Error('New password must be different from current password');
    }

    setIsLoading(true);
    setSuccess(false);

    try {
      await changePassword(currentPassword, newPassword);
      setSuccess(true);
      resetForm();
    } finally {
      setIsLoading(false);
    }
  }, [currentPassword, newPassword, confirmNewPassword, changePassword]);

  const resetForm = useCallback(() => {
    setCurrentPassword('');
    setNewPassword('');
    setConfirmNewPassword('');
    setShowCurrentPassword(false);
    setShowNewPassword(false);
    setShowConfirmNewPassword(false);
    setSuccess(false);
    clearError();
  }, [clearError]);

  const toggleCurrentPasswordVisibility = useCallback(() => {
    setShowCurrentPassword(prev => !prev);
  }, []);

  const toggleNewPasswordVisibility = useCallback(() => {
    setShowNewPassword(prev => !prev);
  }, []);

  const toggleConfirmNewPasswordVisibility = useCallback(() => {
    setShowConfirmNewPassword(prev => !prev);
  }, []);

  const passwordsMatch = newPassword === confirmNewPassword;
  const isValid = currentPassword && newPassword && confirmNewPassword && passwordsMatch;

  return {
    currentPassword,
    setCurrentPassword,
    newPassword,
    setNewPassword,
    confirmNewPassword,
    setConfirmNewPassword,
    showCurrentPassword,
    toggleCurrentPasswordVisibility,
    showNewPassword,
    toggleNewPasswordVisibility,
    showConfirmNewPassword,
    toggleConfirmNewPasswordVisibility,
    handlePasswordChange,
    resetForm,
    passwordsMatch,
    isValid,
    isLoading,
    success,
    error,
    clearError,
  };
};

// Hook for handling password reset request
export const usePasswordResetForm = () => {
  const { requestPasswordReset, error, clearError } = useAuth();
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const handlePasswordResetRequest = useCallback(async () => {
    if (!email) {
      throw new Error('Email is required');
    }

    setIsLoading(true);
    setSuccess(false);

    try {
      await requestPasswordReset(email);
      setSuccess(true);
    } finally {
      setIsLoading(false);
    }
  }, [email, requestPasswordReset]);

  const resetForm = useCallback(() => {
    setEmail('');
    setSuccess(false);
    clearError();
  }, [clearError]);

  return {
    email,
    setEmail,
    handlePasswordResetRequest,
    resetForm,
    isLoading,
    success,
    error,
    clearError,
  };
};

// Hook for session management
export const useSession = () => {
  const { user, token, logout, isAuthenticated } = useAuth();

  const getSessionInfo = useCallback(() => {
    if (!isAuthenticated || !user) {
      return null;
    }

    return {
      user,
      isAuthenticated,
      sessionStart: localStorage.getItem('mev_shield_session_start'),
      lastActivity: localStorage.getItem('mev_shield_last_activity'),
    };
  }, [user, isAuthenticated]);

  const updateLastActivity = useCallback(() => {
    if (isAuthenticated) {
      localStorage.setItem('mev_shield_last_activity', new Date().toISOString());
    }
  }, [isAuthenticated]);

  const checkSessionExpiry = useCallback(() => {
    const lastActivity = localStorage.getItem('mev_shield_last_activity');
    if (!lastActivity) return false;

    const lastActivityTime = new Date(lastActivity).getTime();
    const now = new Date().getTime();
    const sessionTimeout = 24 * 60 * 60 * 1000; // 24 hours

    if (now - lastActivityTime > sessionTimeout) {
      logout();
      return true;
    }

    return false;
  }, [logout]);

  return {
    getSessionInfo,
    updateLastActivity,
    checkSessionExpiry,
    logout,
  };
};

// Hook for role-based feature flags
export const useFeatureFlags = () => {
  const { user } = useAuth();

  const hasFeature = useCallback((feature: string): boolean => {
    if (!user) return false;

    // Define role-based feature access
    const roleFeatures: Record<string, string[]> = {
      'Admin': [
        'admin_panel',
        'user_management',
        'system_config',
        'analytics_full',
        'transaction_management',
        'validator_control',
        'security_logs',
        'backup_restore',
      ],
      'Validator': [
        'validator_panel',
        'transaction_validation',
        'analytics_limited',
        'security_logs',
      ],
      'User': [
        'user_dashboard',
        'transaction_submit',
        'analytics_basic',
        'profile_management',
      ],
      'ReadOnly': [
        'analytics_basic',
        'dashboard_view',
      ],
    };

    const userFeatures = roleFeatures[user.role] || [];
    return userFeatures.includes(feature);
  }, [user]);

  return { hasFeature };
};

export default useAuth;