import { Alert, Box, LinearProgress, Typography } from "@mui/material";
import type { HubTaskSummary } from "../../types/hub";

export interface HubStatusBannerProps {
  task: HubTaskSummary;
}

export function HubStatusBanner({ task }: HubStatusBannerProps) {
  const severity = task.tone === "neutral" || task.tone === "running" ? "info" : task.tone;
  const shouldShowProgress = task.running || task.progressPercent > 0;

  return (
    <Alert severity={severity} variant="outlined">
      <Box sx={{ display: "grid", gap: 0.6, minWidth: 0 }}>
        <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1.2, minWidth: 0 }}>
          <Typography variant="subtitle2">{task.label}</Typography>
          <Typography variant="caption" color="text.secondary">
            {task.operation}
          </Typography>
        </Box>
        <Typography variant="body2">{task.detail}</Typography>
        {shouldShowProgress ? <LinearProgress variant="determinate" value={task.progressPercent} sx={{ height: 5, borderRadius: 999 }} /> : null}
        {task.recovery ? (
          <Typography variant="caption" color="text.secondary">
            {task.recovery}
          </Typography>
        ) : null}
      </Box>
    </Alert>
  );
}
