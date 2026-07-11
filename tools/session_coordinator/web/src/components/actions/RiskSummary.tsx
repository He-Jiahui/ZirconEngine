import { Alert, Chip, Stack, Typography } from "@mui/material";
import type { ActionRecord } from "../../api/contracts";
import { riskLabel } from "../../actions/catalog";

export function RiskSummary({ action }: { action: ActionRecord }) {
  const severity = action.risk === "red" ? "error" : action.risk === "yellow" ? "warning" : "info";
  return <Stack spacing={1}>
    <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><Chip label={riskLabel[action.risk]} color={severity} size="small" /><Typography variant="body2">状态指纹 {action.stateFingerprint.slice(0, 12)}</Typography></Stack>
    <Alert severity={severity}><strong>影响范围</strong><ul>{action.impact.map((item) => <li key={item}>{item}</li>)}</ul>{action.warnings.map((item) => <div key={item}>{item}</div>)}</Alert>
  </Stack>;
}
