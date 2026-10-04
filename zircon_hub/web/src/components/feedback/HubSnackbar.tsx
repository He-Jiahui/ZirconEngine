import { Alert, Box, Snackbar, Typography } from "@mui/material";
import { shouldAutoHideTask } from "../../tauri/hubNotification";
import type { HubTaskSummary } from "../../types/hub";

export interface HubSnackbarProps {
  task: HubTaskSummary;
  open: boolean;
  onClose: () => void;
}

export function HubSnackbar({ task, open, onClose }: HubSnackbarProps) {
  const severity = task.tone === "neutral" || task.tone === "running" ? "info" : task.tone;

  return (
    <Snackbar
      open={open}
      autoHideDuration={shouldAutoHideTask(task) ? 4200 : null}
      onClose={onClose}
      anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
    >
      <Alert data-testid="hub-task-summary-alert" severity={severity} variant="filled" onClose={onClose} sx={{ maxWidth: 520 }}>
        <Box sx={{ display: "grid", gap: 0.45, minWidth: 0 }}>
          <Typography variant="subtitle2">{task.label}</Typography>
          <Typography variant="body2">{task.detail}</Typography>
          {task.recovery ? (
            <Typography variant="caption" sx={{ opacity: 0.86 }}>
              {task.recovery}
            </Typography>
          ) : null}
        </Box>
      </Alert>
    </Snackbar>
  );
}
