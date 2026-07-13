"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hubTheme = void 0;
const styles_1 = require("@mui/material/styles");
const tokens_1 = require("./tokens");
exports.hubTheme = (0, styles_1.createTheme)({
    palette: {
        mode: "dark",
        background: {
            default: tokens_1.hubTokens.colors.background,
            paper: tokens_1.hubTokens.colors.panel,
        },
        primary: {
            main: tokens_1.hubTokens.colors.accent,
            contrastText: tokens_1.hubTokens.colors.textOnPrimary,
        },
        success: {
            main: tokens_1.hubTokens.colors.success,
        },
        warning: {
            main: tokens_1.hubTokens.colors.warning,
        },
        error: {
            main: tokens_1.hubTokens.colors.error,
        },
        text: {
            primary: tokens_1.hubTokens.colors.text,
            secondary: tokens_1.hubTokens.colors.textSoft,
            disabled: tokens_1.hubTokens.colors.textMuted,
        },
        divider: tokens_1.hubTokens.colors.line,
    },
    shape: {
        borderRadius: tokens_1.hubTokens.radius.compact,
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
                    borderRadius: tokens_1.hubTokens.radius.compact,
                    boxShadow: "none",
                    textTransform: "none",
                    whiteSpace: "nowrap",
                },
            },
        },
        MuiCard: {
            styleOverrides: {
                root: {
                    borderRadius: tokens_1.hubTokens.radius.card,
                    backgroundImage: "none",
                    backgroundColor: tokens_1.hubTokens.colors.panel,
                    border: `1px solid ${tokens_1.hubTokens.colors.lineStrong}`,
                    boxShadow: tokens_1.hubTokens.shadows.panel,
                },
            },
        },
        MuiIconButton: {
            styleOverrides: {
                root: {
                    borderRadius: tokens_1.hubTokens.radius.compact,
                },
            },
        },
        MuiMenu: {
            styleOverrides: {
                paper: {
                    backgroundImage: "none",
                    backgroundColor: tokens_1.hubTokens.colors.panel,
                    border: `1px solid ${tokens_1.hubTokens.colors.lineStrong}`,
                },
            },
        },
        MuiOutlinedInput: {
            styleOverrides: {
                root: {
                    borderRadius: tokens_1.hubTokens.radius.compact,
                    backgroundColor: tokens_1.hubTokens.colors.panelLow,
                },
                notchedOutline: {
                    borderColor: tokens_1.hubTokens.colors.lineStrong,
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
                    backgroundColor: tokens_1.hubTokens.colors.tooltip,
                    border: `1px solid ${tokens_1.hubTokens.colors.line}`,
                },
            },
        },
    },
});
