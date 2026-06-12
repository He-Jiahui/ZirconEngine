import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorIcon from "@mui/icons-material/Error";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import WarningIcon from "@mui/icons-material/Warning";
import { Box, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { StatusTone } from "../../types/hub";

export interface StatusBadgeProps {
  label: string;
  tone: StatusTone;
}

const toneMap: Record<StatusTone, { color: string; background: string; border: string; Icon?: typeof PlayArrowIcon }> = {
  running: {
    color: hubTokens.colors.accent,
    background: "rgba(8, 91, 90, 0.38)",
    border: "rgba(33, 213, 207, 0.32)",
    Icon: PlayArrowIcon,
  },
  success: {
    color: hubTokens.colors.success,
    background: "rgba(54, 111, 42, 0.34)",
    border: "rgba(119, 215, 122, 0.3)",
    Icon: CheckCircleIcon,
  },
  warning: {
    color: hubTokens.colors.warning,
    background: "rgba(104, 74, 14, 0.38)",
    border: "rgba(255, 194, 77, 0.32)",
    Icon: WarningIcon,
  },
  error: {
    color: hubTokens.colors.error,
    background: "rgba(105, 31, 29, 0.42)",
    border: "rgba(239, 101, 94, 0.32)",
    Icon: ErrorIcon,
  },
  neutral: {
    color: hubTokens.colors.textSoft,
    background: "rgba(255, 255, 255, 0.06)",
    border: hubTokens.colors.lineStrong,
  },
};

export function StatusBadge({ label, tone }: StatusBadgeProps) {
  const toneStyle = toneMap[tone];
  const Icon = toneStyle.Icon;

  return (
    <Box
      sx={{
        height: 36,
        minWidth: 112,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 0.9,
        px: 1.8,
        color: toneStyle.color,
        backgroundColor: toneStyle.background,
        border: `1px solid ${toneStyle.border}`,
        borderRadius: `${hubTokens.radius.compact}px`,
      }}
    >
      {Icon ? <Icon sx={{ fontSize: 18 }} /> : null}
      <Typography variant="body2" sx={{ color: "inherit", fontWeight: 600 }}>
        {label}
      </Typography>
      {tone === "running" ? (
        <Box sx={{ width: 6, height: 6, borderRadius: hubTokens.radius.pill, backgroundColor: toneStyle.color }} />
      ) : null}
    </Box>
  );
}
