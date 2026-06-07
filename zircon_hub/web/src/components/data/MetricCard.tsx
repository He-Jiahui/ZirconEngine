import type { ReactNode } from "react";
import { Box, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface MetricCardProps {
  label: string;
  value: string;
  detail?: string;
  icon?: ReactNode;
  tone?: "neutral" | "accent" | "success" | "warning" | "error";
}

const toneColor = {
  neutral: hubTokens.colors.textSoft,
  accent: hubTokens.colors.accent,
  success: hubTokens.colors.success,
  warning: hubTokens.colors.warning,
  error: hubTokens.colors.error,
};

export function MetricCard({ label, value, detail, icon, tone = "neutral" }: MetricCardProps) {
  return (
    <Box
      sx={{
        minHeight: 86,
        display: "grid",
        gridTemplateColumns: icon ? "34px minmax(0, 1fr)" : "1fr",
        alignItems: "center",
        gap: 1.1,
        p: 1.4,
        borderRadius: `${hubTokens.radius.panel}px`,
        border: `1px solid ${hubTokens.colors.lineStrong}`,
        backgroundColor: "rgba(32,32,32,0.62)",
      }}
    >
      {icon ? <Box sx={{ color: toneColor[tone], display: "grid", placeItems: "center" }}>{icon}</Box> : null}
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
          {label}
        </Typography>
        <Typography variant="h6" noWrap sx={{ color: toneColor[tone] }}>
          {value}
        </Typography>
        {detail ? (
          <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
            {detail}
          </Typography>
        ) : null}
      </Box>
    </Box>
  );
}
