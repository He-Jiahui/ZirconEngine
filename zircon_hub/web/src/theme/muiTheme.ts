import { createTheme } from "@mui/material/styles";
import { hubTokens } from "./tokens";

export const hubTheme = createTheme({
  palette: {
    mode: "dark",
    background: {
      default: hubTokens.colors.background,
      paper: hubTokens.colors.panel,
    },
    primary: {
      main: hubTokens.colors.accent,
      contrastText: "#071515",
    },
    success: {
      main: hubTokens.colors.success,
    },
    warning: {
      main: hubTokens.colors.warning,
    },
    error: {
      main: hubTokens.colors.error,
    },
    text: {
      primary: hubTokens.colors.text,
      secondary: hubTokens.colors.textSoft,
      disabled: hubTokens.colors.textMuted,
    },
    divider: hubTokens.colors.line,
  },
  shape: {
    borderRadius: hubTokens.radius.compact,
  },
  typography: {
    fontFamily: 'Inter, Roboto, "Segoe UI", Arial, sans-serif',
    h4: {
      fontSize: 28,
      lineHeight: 1.2,
      fontWeight: 700,
      letterSpacing: 0,
    },
    h6: {
      fontSize: 16,
      lineHeight: 1.25,
      fontWeight: 700,
      letterSpacing: 0,
    },
    body1: {
      fontSize: 14,
      letterSpacing: 0,
    },
    body2: {
      fontSize: 13,
      letterSpacing: 0,
    },
    caption: {
      fontSize: 12,
      letterSpacing: 0,
    },
    button: {
      fontSize: 14,
      fontWeight: 500,
      letterSpacing: 0,
      textTransform: "none",
    },
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          height: 42,
          borderRadius: hubTokens.radius.compact,
          boxShadow: "none",
          textTransform: "none",
          whiteSpace: "nowrap",
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: hubTokens.radius.card,
          backgroundImage: "none",
          backgroundColor: hubTokens.colors.panel,
          border: `1px solid ${hubTokens.colors.lineStrong}`,
          boxShadow: hubTokens.shadows.panel,
        },
      },
    },
    MuiIconButton: {
      styleOverrides: {
        root: {
          borderRadius: hubTokens.radius.compact,
        },
      },
    },
    MuiMenu: {
      styleOverrides: {
        paper: {
          backgroundImage: "none",
          backgroundColor: "#202020",
          border: `1px solid ${hubTokens.colors.lineStrong}`,
        },
      },
    },
    MuiOutlinedInput: {
      styleOverrides: {
        root: {
          borderRadius: hubTokens.radius.compact,
          backgroundColor: hubTokens.colors.panelLow,
        },
        notchedOutline: {
          borderColor: hubTokens.colors.lineStrong,
        },
      },
    },
    MuiSelect: {
      styleOverrides: {
        select: {
          display: "flex",
          alignItems: "center",
        },
      },
    },
    MuiTooltip: {
      styleOverrides: {
        tooltip: {
          fontSize: 12,
          backgroundColor: "#242424",
          border: `1px solid ${hubTokens.colors.line}`,
        },
      },
    },
  },
});
