import type { ReactNode } from "react";
import { Box, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

// 概览值和文案由页面预先格式化；色调表达页面已判断的状态，本组件不负责单位、计数或健康校验。
export interface MetricCardProps {
  label: string;
  value: string;
  detail?: string;
  icon?: ReactNode;
  tone?: "neutral" | "accent" | "success" | "warning" | "error";
}

// 各页共用语义色调，避免同一种概览状态因使用场景不同而改变颜色含义。
const toneColor = {
  neutral: hubTokens.colors.textSoft,
  accent: hubTokens.colors.accent,
  success: hubTokens.colors.success,
  warning: hubTokens.colors.warning,
  error: hubTokens.colors.error,
};

// 供概览网格显示短摘要；长值在此受单行约束，需要完整信息的页面应另提供详情区域。
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
