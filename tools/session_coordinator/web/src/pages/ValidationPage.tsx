import { Box, Chip, CircularProgress, Grid, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { JsonObject, ServiceProjection, ValidationHistoryProjection, ValidationProjection } from "../api/contracts";
import { controlClient } from "../api/client";
import { BoundedTable } from "../components/BoundedTable";
import { ValidationLaneTable } from "../components/validation/ValidationLaneTable";
import { ArtifactLifecycleSummary } from "../components/validation/ArtifactLifecycleSummary";
import { ValidationQueueBoard } from "../components/validation/ValidationQueueBoard";
import { ValidationLaunchPanel } from "../components/validation/ValidationLaunchPanel";
import { ValidationRunBoard } from "../components/validation/ValidationRunBoard";
import { ValidationHistory } from "../components/validation/ValidationHistory";
import { HubPanel } from "../theme";
import { DashboardKpi } from "../components/dashboard/DashboardKpi";
import { StageRail } from "../components/dashboard/StageRail";
import { SignalBar } from "../components/dashboard/SignalBar";
const value = (row: JsonObject, key: string) => String(row[key] ?? "—");

export function reservationAge(
  createdAt: string,
  now = new Date(),
  status: "pending" | "leased" | "running" = "pending",
): string {
  const created = Date.parse(createdAt);
  if (!Number.isFinite(created)) return "等待时间未知";
  const minutes = Math.max(0, Math.floor((now.getTime() - created) / 60_000));
  return status === "running" ? `排队等待 ${minutes} 分钟后已运行` : `已等待 ${minutes} 分钟`;
}

export function ValidationPage({ validation, service, refreshKey = 0 }: { validation: ValidationProjection; service?: ServiceProjection; refreshKey?: number }) {
  const [visibleValidation, setVisibleValidation] = useState(validation);
  const [history, setHistory] = useState<ValidationHistoryProjection | null>(null);
  const [historyLimit, setHistoryLimit] = useState(50);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [historyLoading, setHistoryLoading] = useState(true);
  useEffect(() => {
    const controller = new AbortController();
    setVisibleValidation(validation);
    controlClient.validation(controller.signal).then((next) => {
      setVisibleValidation(next);
      setDetailError(null);
    }).catch((reason) => {
      if (!controller.signal.aborted) setDetailError(String(reason));
    });
    return () => controller.abort();
  }, [refreshKey, validation]);
  useEffect(() => {
    const controller = new AbortController();
    setHistoryLoading(true);
    controlClient.validationHistory(historyLimit, controller.signal).then((next) => {
      setHistory(next);
      setHistoryError(null);
    }).catch((reason) => {
      if (!controller.signal.aborted) setHistoryError(String(reason));
    }).finally(() => {
      if (!controller.signal.aborted) setHistoryLoading(false);
    });
    return () => controller.abort();
  }, [refreshKey, historyLimit]);
  const runHealth = visibleValidation.runHealth ?? [];
  const silentRuns = runHealth.filter((run) => run.outputState === "awaiting_output");
  const observedRuns = runHealth.filter((run) => run.outputState === "output_observed");
  const unavailableRuns = runHealth.filter((run) => run.outputState === "log_unavailable");
  const reservations = visibleValidation.cargoReservations;
  const running = visibleValidation.currentCargoTargets.filter((job) => ["leased", "running"].includes(job.status)).length;
  const queued = reservations.filter((reservation) => reservation.status === "pending").length;
  const passed = visibleValidation.currentCargoTargets.filter((job) => job.status === "succeeded").length;
  const failed = visibleValidation.currentCargoTargets.filter((job) => job.status === "failed").length;
  const total = Math.max(running + queued + passed + failed, 1);
  const historyCounts = history?.statusCounts;
  return <Stack spacing={2} className="dashboard-page">
    <Box className="dashboard-band"><Stack direction={{ xs: "column", md: "row" }} spacing={2} sx={{ alignItems: { md: "center" } }}><Stack spacing={0.5} sx={{ flex: 1 }}><Typography variant="overline" color="primary.main">VALIDATION / LIVE CHANNEL</Typography><Typography variant="h4">验证控制台</Typography><Typography variant="body2" color="text.secondary">每个验证 ticket 都有队列位置、当前阶段、运行耗时和可追溯历史。</Typography></Stack><Stack direction="row" spacing={1}><Chip size="small" color={running ? "primary" : "default"} label={running ? `${running} 项运行中` : "通道空闲"} /><Chip size="small" variant="outlined" label={`${queued} 项排队`} /></Stack></Stack></Box>
    <Box className="dashboard-kpi-grid"><DashboardKpi label="运行中" value={running} detail="正在占用受管验证通道" tone={running ? "info" : "neutral"} /><DashboardKpi label="队列" value={queued} detail="FIFO 位置，不影响 Session 准入" tone={queued ? "warning" : "success"} /><DashboardKpi label="已通过" value={historyCounts?.passed ?? passed} detail="可作为里程碑提交证据" tone="success" /><DashboardKpi label="失败" value={historyCounts?.failed ?? failed} detail="进入 Failure 链逐项修复" tone={historyCounts?.failed || failed ? "danger" : "neutral"} /></Box>
    {service && <HubPanel title="立即继续验证"><ValidationLaunchPanel service={service} /></HubPanel>}
    {detailError && <Typography role="alert" color="warning.main">完整验证投影加载失败，当前显示首屏摘要：{detailError}</Typography>}
    <HubPanel title="验证流水线进度"><StageRail ariaLabel="验证阶段" stages={[{ label: "排队", state: queued ? "queued" : "done", detail: `${queued} 项等待通道` }, { label: "物化副本", state: running ? "active" : "queued", detail: `${visibleValidation.validationCopies.filter((copy) => ["materialized", "running"].includes(copy.status)).length} 个副本可用` }, { label: "运行命令", state: running ? "active" : passed ? "done" : "queued", detail: `${running} 运行 · ${passed} 通过` }, { label: "结论", state: failed ? "blocked" : passed ? "done" : "queued", detail: failed ? `${failed} 项失败` : "等待终态" }]} /><Box sx={{ mt: 2 }}><ValidationRunBoard jobs={visibleValidation.currentCargoTargets} runHealth={runHealth} /></Box></HubPanel>
    <Grid container spacing={2}><Grid size={{ xs: 12, lg: 7 }}><HubPanel title="验证通道队列"><ValidationQueueBoard reservations={reservations} cpuBurst={visibleValidation.cpuBurst ?? { capacity: 1, active: 0, eligiblePending: 0 }} /></HubPanel></Grid><Grid size={{ xs: 12, lg: 5 }}><HubPanel title="队列负载"><Stack spacing={1.5}><SignalBar label="正在运行" value={running} total={total} tone="primary.main" /><SignalBar label="等待中" value={queued} total={total} tone="warning.main" /><SignalBar label="终态通过" value={passed} total={total} tone="success.main" /><Typography variant="caption" color="text.secondary">只排验证，不阻塞 Session 注册、文件工作或 failure 修复。</Typography></Stack></HubPanel></Grid></Grid>
    <HubPanel title="验证运行健康">{runHealth.length === 0 ? <Typography>受管验证尚未启动。</Typography> : <Stack spacing={0.5}>{observedRuns.length > 0 && <><Typography>{observedRuns.length} 个受管验证作业已有输出。</Typography>{observedRuns.map((run) => <Typography key={run.runId} variant="caption" color="text.secondary">{run.sessionId} · {run.jobId} · {run.lastOutputAt ? `最后输出 ${run.lastOutputAt}` : "已观察到输出"}</Typography>)}</>}{silentRuns.length > 0 && <><Typography>{silentRuns.length} 个受管验证作业尚未写出输出；这不会关闭 Session 准入。</Typography>{silentRuns.map((run) => <Typography key={run.runId} variant="caption" color="text.secondary">{run.sessionId} · {run.jobId} · 自 {run.startedAt} 等待输出</Typography>)}</>}{unavailableRuns.length > 0 && <><Typography>{unavailableRuns.length} 个受管验证作业的日志状态暂不可读；这不会关闭 Session 准入。</Typography>{unavailableRuns.map((run) => <Typography key={run.runId} variant="caption" color="text.secondary">{run.sessionId} · {run.jobId}</Typography>)}</>}</Stack>}</HubPanel>
    <HubPanel title="验证历史记录">{historyError && !history && <Typography role="alert" color="warning.main">{historyError}</Typography>}{!history && historyLoading && <CircularProgress aria-label="加载验证历史" size={24} />}{history && <ValidationHistory history={history} loading={historyLoading} onLoadMore={() => setHistoryLimit((limit) => Math.min(200, limit + 50))} />}</HubPanel>
    <Grid container spacing={2}><Grid size={{ xs: 12, md: 7 }}><HubPanel title="Cargo 实时通道"><ArtifactLifecycleSummary lifecycle={visibleValidation.artifactLifecycle} /><ValidationLaneTable jobs={visibleValidation.currentCargoTargets} /></HubPanel></Grid><Grid size={{ xs: 12, md: 5 }}><HubPanel title="临时验证副本"><Typography variant="caption" color="text.secondary">构建后由服务清理，不形成 worktree。</Typography><BoundedTable rows={visibleValidation.validationCopies} rowKey={(row) => value(row, "job_id")} columns={[{ key: "id", label: "副本", render: (row) => value(row, "job_id") }, { key: "session", label: "Session", render: (row) => value(row, "session_id") }, { key: "status", label: "状态", render: (row) => value(row, "status") }, { key: "time", label: "创建时间", render: (row) => value(row, "created_at") }]} /></HubPanel></Grid></Grid>
  </Stack>;
}
