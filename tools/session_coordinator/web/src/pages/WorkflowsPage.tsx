import { Alert, Box, Chip, Grid, MenuItem, Select, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { CollaborationProjection, FailureProjection, ValidationProjection, WorkflowDetail, WorkflowSummary } from "../api/contracts";
import { controlClient } from "../api/client";
import { ValidationQueueBoard } from "../components/validation/ValidationQueueBoard";
import { WorkflowPipeline } from "../components/workflow/WorkflowPipeline";
import { HubPanel } from "../theme";

export function WorkflowsPage({ workflows, collaboration, failures: initialFailures, validation, refreshKey }: { workflows: WorkflowSummary[]; collaboration: CollaborationProjection; failures: FailureProjection; validation: ValidationProjection; refreshKey: number }) {
  const [runId, setRunId] = useState(workflows[0]?.runId ?? "");
  const [detail, setDetail] = useState<WorkflowDetail | null>(null);
  const [failures, setFailures] = useState<FailureProjection>(initialFailures);
  const [error, setError] = useState<string | null>(null);
  const selectedUpdatedAt = workflows.find((workflow) => workflow.runId === runId)?.updatedAt;
  useEffect(() => {
    if (!workflows.length) { setRunId(""); setDetail(null); return; }
    if (!workflows.some((workflow) => workflow.runId === runId)) setRunId(workflows[0].runId);
  }, [runId, workflows]);
  useEffect(() => { const controller = new AbortController(); controlClient.failures(controller.signal).then(setFailures).catch(() => {}); return () => controller.abort(); }, [refreshKey]);
  useEffect(() => { if (!runId) return; const controller = new AbortController(); controlClient.workflow(runId, controller.signal).then((next) => { setDetail(next); setError(null); }).catch((reason) => { if (!controller.signal.aborted) setError(String(reason)); }); return () => controller.abort(); }, [runId, selectedUpdatedAt]);
  return <Stack spacing={2}>
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, lg: 5 }}><HubPanel title="工作流选择"><Select size="small" fullWidth value={runId} onChange={(event) => setRunId(event.target.value)} displayEmpty>{workflows.map((item) => <MenuItem value={item.runId} key={item.runId}>{item.workflowKey} · {item.sessionId} · {item.state}</MenuItem>)}</Select></HubPanel></Grid>
      <Grid size={{ xs: 12, lg: 7 }}><HubPanel title="构建与验证队列"><ValidationQueueBoard reservations={validation.cargoReservations} cpuBurst={validation.cpuBurst} /></HubPanel></Grid>
    </Grid>
    {error && <Typography role="alert">{error}</Typography>}
    {detail && <>
      <HubPanel title="流水线状态"><WorkflowRunSummary detail={detail} /><WorkflowPipeline detail={detail} leases={collaboration.leases} failures={failures.nodes} /></HubPanel>
      <Grid container spacing={2}>
        <Grid size={{ xs: 12, lg: 7 }}><HubPanel title="拓扑版本与门禁"><Stack spacing={1}>{detail.topologyVersions.map((version) => <Typography key={version.topologyVersionId}>v{version.versionNumber} · {version.sourceKind} · {version.topologyHash.slice(0, 12)} {version.active && <Chip size="small" color="success" label="当前" />}</Typography>)}{detail.gates.map((gate) => <Alert key={gate.evidenceId} severity={gate.decision === "accepted" ? "success" : "warning"}>{gate.kind} · {gate.decision} · {gate.code} · 指纹 {gate.inputFingerprint.slice(0, 12)}</Alert>)}</Stack></HubPanel></Grid>
        <Grid size={{ xs: 12, lg: 5 }}><HubPanel title="评审与通知记录"><Stack spacing={1}>{detail.reviews.map((review) => <Typography key={review.reviewId}>{review.reviewer} · {review.verdict} · Critical {review.criticalCount} / Important {review.importantCount}</Typography>)}{detail.notifications.map((attempt) => <Alert key={attempt.attemptId} severity={attempt.status === "succeeded" ? "success" : "warning"}>企业微信 {attempt.status} · {attempt.commitSha.slice(0, 12)} · 禁止自动重试</Alert>)}{!detail.reviews.length && !detail.notifications.length && <Typography color="text.secondary">暂无评审或通知记录。</Typography>}</Stack></HubPanel></Grid>
      </Grid>
    </>}
    {!workflows.length && <Typography>没有可显示的工作流。</Typography>}
  </Stack>;
}

function WorkflowRunSummary({ detail }: { detail: WorkflowDetail }) {
  const totals = {
    complete: detail.nodes.filter((node) => ["succeeded", "skipped"].includes(node.state)).length,
    running: detail.nodes.filter((node) => ["running", "leased"].includes(node.state)).length,
    failed: detail.nodes.filter((node) => ["failed", "blocked"].includes(node.state)).length,
  };
  const metrics = [["节点", detail.nodes.length], ["已完成", totals.complete], ["运行中", totals.running], ["需处理", totals.failed]];
  return <Stack spacing={1.25} sx={{ mb: 2 }}><Typography variant="caption" color="text.secondary" sx={{ overflowWrap: "anywhere" }}>{detail.workflowKey}</Typography><Box sx={{ display: "grid", gridTemplateColumns: { xs: "repeat(2, minmax(0, 1fr))", md: "repeat(4, minmax(0, 1fr))" }, gap: 1 }}>
    {metrics.map(([label, value]) => <Box key={label} sx={{ borderLeft: 2, borderColor: label === "需处理" && value ? "error.main" : label === "运行中" && value ? "warning.main" : "divider", pl: 1 }}><Typography variant="caption" color="text.secondary">{label}</Typography><Typography variant="h6">{value}</Typography></Box>)}
  </Box></Stack>;
}
