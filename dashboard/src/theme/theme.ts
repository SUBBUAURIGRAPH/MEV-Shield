import { createTheme, alpha } from '@mui/material/styles';

// Modern color palette with better contrast and visual hierarchy
const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#00D4FF', // Cyan - represents technology and protection
      light: '#5DEFF9',
      dark: '#00A3CC',
      contrastText: '#000000',
    },
    secondary: {
      main: '#7B61FF', // Purple - represents premium features
      light: '#A491FF',
      dark: '#5841CC',
      contrastText: '#FFFFFF',
    },
    success: {
      main: '#00FF88', // Bright green for success
      light: '#5DFFAA',
      dark: '#00CC6A',
      contrastText: '#000000',
    },
    error: {
      main: '#FF4757', // Soft red for errors
      light: '#FF7A85',
      dark: '#CC3946',
      contrastText: '#FFFFFF',
    },
    warning: {
      main: '#FFB800', // Amber for warnings
      light: '#FFC94D',
      dark: '#CC9300',
      contrastText: '#000000',
    },
    info: {
      main: '#00B8D4',
      light: '#4DD0E1',
      dark: '#00838F',
      contrastText: '#FFFFFF',
    },
    background: {
      default: '#0A0E1A', // Dark navy background
      paper: '#141925', // Slightly lighter for cards
    },
    text: {
      primary: '#FFFFFF',
      secondary: alpha('#FFFFFF', 0.7),
    },
    divider: alpha('#FFFFFF', 0.12),
  },
  typography: {
    fontFamily: '"Inter", "SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    h1: {
      fontSize: '3rem',
      fontWeight: 800,
      lineHeight: 1.2,
      letterSpacing: '-0.02em',
    },
    h2: {
      fontSize: '2.25rem',
      fontWeight: 700,
      lineHeight: 1.3,
      letterSpacing: '-0.01em',
    },
    h3: {
      fontSize: '1.875rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    h4: {
      fontSize: '1.5rem',
      fontWeight: 600,
      lineHeight: 1.5,
    },
    h5: {
      fontSize: '1.25rem',
      fontWeight: 600,
      lineHeight: 1.6,
    },
    h6: {
      fontSize: '1rem',
      fontWeight: 600,
      lineHeight: 1.6,
    },
    body1: {
      fontSize: '1rem',
      lineHeight: 1.7,
    },
    body2: {
      fontSize: '0.875rem',
      lineHeight: 1.6,
    },
    button: {
      fontSize: '0.9375rem',
      fontWeight: 600,
      letterSpacing: '0.02em',
      textTransform: 'none',
    },
    caption: {
      fontSize: '0.75rem',
      lineHeight: 1.5,
    },
  },
  shape: {
    borderRadius: 16,
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 12,
          padding: '10px 24px',
          fontSize: '0.9375rem',
          fontWeight: 600,
          textTransform: 'none',
          boxShadow: 'none',
          transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
          '&:hover': {
            boxShadow: '0 8px 16px rgba(0, 212, 255, 0.24)',
            transform: 'translateY(-2px)',
          },
        },
        containedPrimary: {
          background: 'linear-gradient(135deg, #00D4FF 0%, #00A3CC 100%)',
          '&:hover': {
            background: 'linear-gradient(135deg, #5DEFF9 0%, #00D4FF 100%)',
          },
        },
        containedSecondary: {
          background: 'linear-gradient(135deg, #7B61FF 0%, #5841CC 100%)',
          '&:hover': {
            background: 'linear-gradient(135deg, #A491FF 0%, #7B61FF 100%)',
          },
        },
        outlined: {
          borderWidth: 2,
          '&:hover': {
            borderWidth: 2,
            backgroundColor: alpha('#00D4FF', 0.08),
          },
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          backgroundImage: 'linear-gradient(135deg, rgba(20, 25, 37, 0.8) 0%, rgba(20, 25, 37, 0.95) 100%)',
          backdropFilter: 'blur(20px)',
          border: '1px solid',
          borderColor: alpha('#00D4FF', 0.1),
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4)',
          transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
          '&:hover': {
            boxShadow: '0 12px 48px rgba(0, 212, 255, 0.15)',
            borderColor: alpha('#00D4FF', 0.3),
            transform: 'translateY(-4px)',
          },
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundImage: 'linear-gradient(135deg, rgba(20, 25, 37, 0.8) 0%, rgba(20, 25, 37, 0.95) 100%)',
          backdropFilter: 'blur(10px)',
          border: '1px solid',
          borderColor: alpha('#00D4FF', 0.08),
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          '& .MuiOutlinedInput-root': {
            borderRadius: 12,
            backgroundColor: alpha('#141925', 0.5),
            transition: 'all 0.3s ease',
            '&:hover': {
              backgroundColor: alpha('#141925', 0.8),
              '& .MuiOutlinedInput-notchedOutline': {
                borderColor: alpha('#00D4FF', 0.5),
              },
            },
            '&.Mui-focused': {
              backgroundColor: alpha('#141925', 0.9),
              '& .MuiOutlinedInput-notchedOutline': {
                borderColor: '#00D4FF',
                borderWidth: 2,
              },
            },
          },
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          fontWeight: 600,
          fontSize: '0.75rem',
          height: 28,
        },
        colorPrimary: {
          background: alpha('#00D4FF', 0.15),
          color: '#00D4FF',
          border: `1px solid ${alpha('#00D4FF', 0.3)}`,
        },
        colorSuccess: {
          background: alpha('#00FF88', 0.15),
          color: '#00FF88',
          border: `1px solid ${alpha('#00FF88', 0.3)}`,
        },
        colorError: {
          background: alpha('#FF4757', 0.15),
          color: '#FF4757',
          border: `1px solid ${alpha('#FF4757', 0.3)}`,
        },
        colorWarning: {
          background: alpha('#FFB800', 0.15),
          color: '#FFB800',
          border: `1px solid ${alpha('#FFB800', 0.3)}`,
        },
      },
    },
    MuiAlert: {
      styleOverrides: {
        root: {
          borderRadius: 12,
          backdropFilter: 'blur(10px)',
        },
        standardSuccess: {
          backgroundColor: alpha('#00FF88', 0.15),
          color: '#00FF88',
          border: `1px solid ${alpha('#00FF88', 0.3)}`,
          '& .MuiAlert-icon': {
            color: '#00FF88',
          },
        },
        standardError: {
          backgroundColor: alpha('#FF4757', 0.15),
          color: '#FF4757',
          border: `1px solid ${alpha('#FF4757', 0.3)}`,
          '& .MuiAlert-icon': {
            color: '#FF4757',
          },
        },
        standardWarning: {
          backgroundColor: alpha('#FFB800', 0.15),
          color: '#FFB800',
          border: `1px solid ${alpha('#FFB800', 0.3)}`,
          '& .MuiAlert-icon': {
            color: '#FFB800',
          },
        },
        standardInfo: {
          backgroundColor: alpha('#00B8D4', 0.15),
          color: '#00B8D4',
          border: `1px solid ${alpha('#00B8D4', 0.3)}`,
          '& .MuiAlert-icon': {
            color: '#00B8D4',
          },
        },
      },
    },
    MuiLinearProgress: {
      styleOverrides: {
        root: {
          height: 6,
          borderRadius: 3,
          backgroundColor: alpha('#00D4FF', 0.1),
        },
        bar: {
          borderRadius: 3,
          background: 'linear-gradient(90deg, #00D4FF 0%, #7B61FF 100%)',
        },
      },
    },
    MuiTableCell: {
      styleOverrides: {
        root: {
          borderBottomColor: alpha('#00D4FF', 0.08),
        },
      },
    },
    MuiDivider: {
      styleOverrides: {
        root: {
          borderColor: alpha('#00D4FF', 0.08),
        },
      },
    },
    MuiTooltip: {
      styleOverrides: {
        tooltip: {
          backgroundColor: '#1F2633',
          borderRadius: 8,
          fontSize: '0.875rem',
          padding: '8px 12px',
          border: `1px solid ${alpha('#00D4FF', 0.1)}`,
        },
      },
    },
    MuiIconButton: {
      styleOverrides: {
        root: {
          transition: 'all 0.2s ease',
          '&:hover': {
            backgroundColor: alpha('#00D4FF', 0.08),
            transform: 'scale(1.1)',
          },
        },
      },
    },
    MuiSwitch: {
      styleOverrides: {
        switchBase: {
          '&.Mui-checked': {
            '& .MuiSwitch-thumb': {
              background: 'linear-gradient(135deg, #00D4FF 0%, #00A3CC 100%)',
            },
          },
        },
        track: {
          backgroundColor: alpha('#FFFFFF', 0.2),
        },
      },
    },
  },
});

export default theme;