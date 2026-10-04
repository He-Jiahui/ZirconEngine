import CancelOutlinedIcon from "@mui/icons-material/CancelOutlined";
import { Alert, Box, IconButton, LinearProgress, Tooltip, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubTaskSummary } from "../../types/hub";

export interface HubStatusBannerProps {
  task: HubTaskSummary;
  cancelLabel: string;
  onCancel: () => void;
}

export function HubStatusBanner({ task, cancelLabel, onCancel }: HubStatusBannerProps) {
  const severity = task.tone === "neutral" || task.tone === "running" ? "info" : task.tone;
  const shouldShowProgress = task.running || task.progressPercent > 0;

  return (
    <Alert
      severity={severity}
      variant="outlined"
      action={task.running && task.cancellable ? (
        <Tooltip title={cancelLabel}>
          <IconButton aria-label={cancelLabel} color="inherit" size="small" onClick={onCancel}>
            <CancelOutlinedIcon fontSize="small" />
          </IconButton>
        </Tooltip>
      ) : undefined}
    >
      <Box sx={{ display: "grid", gap: 0.6, minWidth: 0 }}>
        <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1.2, minWidth: 0 }}>
          <Typography variant="subtitle2">{task.label}</Typography>
          <Typography variant="caption" color="text.secondary">
            {task.operation}
          </Typography>
        </Box>
        <Typography variant="body2">{task.detail}</Typography>
        {shouldShowProgress ? (
          <LinearProgress aria-label={task.operation || task.label} variant="determinate" value={task.progressPercent} sx={{ height: 5, borderRadius: hubTokens.radius.pill }} />
        ) : null}
        {task.recovery ? (
          <Typography variant="caption" color="text.secondary">
            {task.recovery}
          </Typography>
        ) : null}
      </Box>
    </Alert>
  );
}
