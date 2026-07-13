"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HubPanel = HubPanel;
const jsx_runtime_1 = require("react/jsx-runtime");
const material_1 = require("@mui/material");
const tokens_1 = require("../../theme/tokens");
function HubPanel({ title, action, children }) {
    return ((0, jsx_runtime_1.jsxs)(material_1.Card, { component: "section", sx: {
            p: 2,
            minWidth: 0,
            overflow: "hidden",
        }, children: [(0, jsx_runtime_1.jsxs)(material_1.Box, { sx: { display: "flex", alignItems: "center", gap: 2, mb: 1.6 }, children: [(0, jsx_runtime_1.jsx)(material_1.Typography, { variant: "h6", sx: { flex: "1 1 auto" }, children: title }), action] }), (0, jsx_runtime_1.jsx)(material_1.Box, { sx: { color: tokens_1.hubTokens.colors.textSoft }, children: children })] }));
}
