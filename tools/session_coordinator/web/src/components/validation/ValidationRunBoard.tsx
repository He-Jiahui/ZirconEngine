import { Box, LinearProgress, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { CargoLaneProjection, CargoRunHealthProjection } from "../../api/contracts";
import { StatusText } from "../StatusText";
import { validationRunProgress } from "./validationRunModel";

export function ValidationRunBoard({ jobs, runHealth }: { jobs: CargoLaneProjection[]; runHealth: CargoRunHealthProjection[] }) {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  const runs = validationRunProgress(jobs, runHealth, now);
  if (!runs.length) return <Typography color="text.secondary">暂无验证作业；从上方选择工作流后即可立即投入通道。</Typography>;
  return <Box sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", lg: "repeat(2, minmax(0, 1fr))" }, gap: 1.5 }}>
    {runs.map((run) => <Stack key={run.jobId} component="section" spacing={1} aria-label={`验证作业 ${run.jobId}`} sx={{ minWidth: 0, border: 1, borderColor: run.state === "failed" ? "error.main" : "divider", borderRadius: 1, p: 1.5 }}>
      <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><StatusText value={run.state} /><Typography variant="subtitle2" sx={{ minWidth: 0, flex: 1 }} noWrap title={run.sessionId}>{run.sessionId}</Typography><Typography variant="caption">{run.elapsed}</Typography></Stack>
      <LinearProgress variant="determinate" value={(run.stepIndex / run.stepCount) * 100} color={run.state === "failed" ? "error" : run.state === "succeeded" ? "success" : "primary"} />
      <Typography variant="body2">第 {run.stepIndex}/{run.stepCount} 项 · {run.stepLabel}</Typography>
      <Typography variant="caption" color="text.secondary">{run.lane.toUpperCase()} · {run.outputLabel} · Job {run.jobId}</Typography>
    </Stack>)}
  </Box>;
}
