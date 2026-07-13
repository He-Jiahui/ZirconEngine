"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HubButton = HubButton;
const jsx_runtime_1 = require("react/jsx-runtime");
const material_1 = require("@mui/material");
const tokens_1 = require("../../theme/tokens");
const toneStyles = {
    primary: {
        color: tokens_1.hubTokens.colors.textOnAccent,
        backgroundColor: tokens_1.hubTokens.colors.accentDim,
        borderColor: "rgba(45, 212, 207, 0.48)",
        "&:hover": {
            backgroundColor: "rgba(17, 127, 124, 0.92)",
            borderColor: "rgba(45, 212, 207, 0.68)",
        },
    },
    secondary: {
        color: tokens_1.hubTokens.colors.text,
        backgroundColor: "rgba(32,32,32,0.82)",
        borderColor: tokens_1.hubTokens.colors.lineStrong,
        "&:hover": {
            backgroundColor: tokens_1.hubTokens.colors.panelHover,
            borderColor: "rgba(255,255,255,0.22)",
        },
    },
    tertiary: {
        color: tokens_1.hubTokens.colors.accent,
        backgroundColor: "transparent",
        borderColor: "transparent",
        "&:hover": {
            backgroundColor: "rgba(33,213,207,0.08)",
            borderColor: "transparent",
        },
    },
    danger: {
        color: tokens_1.hubTokens.colors.dangerText,
        backgroundColor: "rgba(120,25,25,0.54)",
        borderColor: "rgba(245,111,102,0.48)",
        "&:hover": {
            backgroundColor: "rgba(142,30,29,0.72)",
        },
    },
};
function HubButton({ tone = "secondary", sx, ...props }) {
    return ((0, jsx_runtime_1.jsx)(material_1.Button, { ...props, variant: "contained", sx: [
            {
                border: "1px solid",
                px: 2.5,
                minWidth: 0,
            },
            toneStyles[tone],
            ...asSxArray(sx),
        ] }));
}
function asSxArray(sx) {
    if (!sx) {
        return [];
    }
    return Array.isArray(sx) ? sx : [sx];
}
