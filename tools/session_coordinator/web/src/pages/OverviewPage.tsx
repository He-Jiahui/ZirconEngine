import { Grid, Typography } from "@mui/material";
import type { ControlSnapshot } from "../api/contracts";
import { HubPanel } from "../theme";

export function OverviewPage({ snapshot }: { snapshot: ControlSnapshot }) {
  const metrics = overviewMetrics(snapshot);
  return <Grid container spacing={2}>{metrics.map(([label, value]) => <Grid key={label} size={{ xs: 12, sm: 6, lg: 3 }}><HubPanel title={label}><Typography variant="h4">{value}</Typography></HubPanel></Grid>)}</Grid>;
}

export function overviewMetrics(snapshot: ControlSnapshot) {
  return [["工作流", snapshot.workflows.length], ["活动会话", snapshot.sessions.filter((item) => item.status === "active").length], ["Failure", snapshot.failures.nodes.length], ["运行验证", snapshot.validation.cargoJobs.filter((item) => item.status === "running").length]] as const;
}
