import { Box, Button, Chip, Grid, Stack, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import type { ControlSnapshot, ExperienceProjection, SessionProjection } from "../api/contracts";
import { controlClient } from "../api/client";
import { HubPanel } from "../theme";
import { DashboardKpi, type DashboardTone } from "../components/dashboard/DashboardKpi";
import { StageRail, type StageRailItem } from "../components/dashboard/StageRail";
import { SignalBar } from "../components/dashboard/SignalBar";
import { ActiveTaskTable, AnalyticsDurationChart, AnalyticsFlow, AnalyticsLineChart, AnalyticsSchedule, FailureDonut, ModuleShareChart, PatchStatusChart, PlanProgressChart, RecentWorkflowList, formatDuration } from "../components/dashboard/AnalyticsCharts";
import { buildAnalytics } from "../components/dashboard/analyticsModel";
import { ValidationReportSummary } from "../components/dashboard/ValidationReportSummary";
import { useOverviewReportData } from "../components/dashboard/useOverviewReportData";

export function OverviewPage({ snapshot, refreshKey = 0, onNavigate }: { snapshot: ControlSnapshot; refreshKey?: number; onNavigate?: (path: string) => void }) {
  const [continuationRows, setContinuationRows] = useState(snapshot.experience?.continuations ?? []);
  const { validationHistory, failureHistory, auditEvents, validationHistoryError } = useOverviewReportData();
  useEffect(() => {
    const controller = new AbortController();
    setContinuationRows(snapshot.experience?.continuations ?? []);
    controlClient.continuations(controller.signal).then((projection) => setContinuationRows(projection.continuations)).catch(() => undefined);
    return () => controller.abort();
  }, [refreshKey, snapshot.experience?.continuations]);
  const visibleSnapshot: ControlSnapshot = {
    ...snapshot,
    experience: {
      ...(snapshot.experience ?? { sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 }, blockers: [], continuations: [] }),
      continuations: continuationRows,
    },
  };
  const metrics = overviewMetrics(visibleSnapshot);
  const blockers = resourceBlockers(visibleSnapshot);
  const admission = admissionSummary(visibleSnapshot);
  const baseline = workspaceBaselineSummary(visibleSnapshot);
  const cleanup = cleanupDebtSummary(visibleSnapshot);
  const intervention = interventionGuidance(visibleSnapshot);
  const board = workBoard(visibleSnapshot);
  const flowHealth = validationFlowHealth(visibleSnapshot);
  const syncHealth = syncHealthSummary(visibleSnapshot);
  const continuations = continuationGuidance(visibleSnapshot);
  const continuationCoverage = continuationCoverageSummary(visibleSnapshot);
  const cpuBurst = visibleSnapshot.validation?.cpuBurst ?? { capacity: 1, active: 0, eligiblePending: 0 };
  const analytics = buildAnalytics(visibleSnapshot, { validationHistory, failureHistory, audit: auditEvents.length ? auditEvents : visibleSnapshot.audit });
  const currentCargoTargets = visibleSnapshot.validation?.currentCargoTargets ?? [];
  const cargoReservations = visibleSnapshot.validation?.cargoReservations ?? [];
  const finalizeRequests = visibleSnapshot.git?.finalizeRequests ?? [];
  const activeCount = currentCargoTargets.filter((item) => ["leased", "running"].includes(item.status)).length;
  const pendingCount = cargoReservations.filter((item) => item.status === "pending").length;
  const completedCount = currentCargoTargets.filter((item) => ["succeeded"].includes(item.status)).length;
  const stageItems: StageRailItem[] = [
    { label: "计划提交", state: metrics[0][1] ? "done" : "queued", detail: `${metrics[0][1]} 个工作流已登记` },
    { label: "验证队列", state: pendingCount ? "queued" : activeCount ? "active" : "done", detail: `${pendingCount} 项等待 · ${activeCount} 项占用通道` },
    { label: "执行验证", state: activeCount ? "active" : completedCount ? "done" : "queued", detail: `${activeCount} 运行中 · ${completedCount} 已完成` },
    { label: "Failure 审核", state: intervention.failureCount ? "blocked" : "done", detail: intervention.failureCount ? `${intervention.failureCount} 项待逐项审核` : "没有开放 Failure" },
    { label: "里程碑提交", state: finalizeRequests.some((request) => request.status === "committed") ? "done" : "queued", detail: `${finalizeRequests.length} 个提交请求` },
  ];
  const metricDetails: Record<string, string> = { "工作流": "已登记的执行流水线", "活动会话": "仍在写入或等待的 Session", Failure: intervention.failureCount ? "需要进入失败链处理" : "没有开放项", "运行验证": activeCount ? "正在占用验证通道" : "当前没有运行作业", "同步状态": syncHealth.detail, "资源阻塞": blockers.length ? "只影响验证资源" : "没有独占资源等待" };
  const metricTone = (label: string, value: string | number): DashboardTone => label === "Failure" && Number(value) > 0 ? "danger" : label === "资源阻塞" && Number(value) > 0 ? "warning" : label === "同步状态" && value === "安静" ? "success" : label === "运行验证" && Number(value) > 0 ? "info" : "neutral";
  const openPage = (path: string) => {
    if (onNavigate) { onNavigate(path); return; }
    window.history.pushState({}, "", path);
    window.dispatchEvent(new PopStateEvent("popstate"));
  };
  return <Stack spacing={2} className="dashboard-page">
    <Box className="dashboard-band">
      <Stack direction={{ xs: "column", md: "row" }} spacing={2} sx={{ alignItems: { md: "center" } }}>
        <Stack spacing={0.5} sx={{ flex: 1, minWidth: 0 }}><Typography variant="overline" color="primary.main">COORDINATOR / CONTROL CENTER</Typography><Typography variant="h4">执行看板</Typography><Typography variant="body2" color="text.secondary">把计划、验证通道、Failure 链和里程碑提交放在同一条可追踪路径上。</Typography></Stack>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={1} sx={{ alignItems: { sm: "center" }, flexWrap: "wrap" }}><Stack direction="row" spacing={1}><Chip size="small" color={visibleSnapshot.service?.mode === "read_write" ? "success" : "warning"} label={visibleSnapshot.service?.mode === "read_write" ? "写入正常" : visibleSnapshot.service?.mode ?? "模式未知"} /><Chip size="small" variant="outlined" label={`${visibleSnapshot.sessions.filter((session) => session.status === "active").length} 个活跃 Session`} /></Stack><Stack direction={{ xs: "column", sm: "row" }} spacing={0.75}><Button size="small" variant="contained" onClick={() => openPage("/ui/validation")}>进入验证队列</Button><Button size="small" variant="outlined" onClick={() => openPage("/ui/failures")}>查看失败链</Button><Button size="small" variant="outlined" onClick={() => openPage("/ui/git")}>里程碑提交</Button></Stack></Stack>
      </Stack>
    </Box>
    <ValidationReportSummary report={analytics.validationReport} loadError={validationHistoryError} />
    <HubPanel title="构建 → 验证 → 审核 → 提交"><StageRail stages={stageItems} ariaLabel="构建验证提交阶段" /></HubPanel>
    <Grid container spacing={2}><Grid size={{ xs: 12, lg: 7 }}><HubPanel title="24 小时任务趋势"><AnalyticsLineChart buckets={analytics.buckets} /><Typography variant="caption" color="text.secondary">{analytics.coverage.hasTimeSeries ? `已加载 ${analytics.coverage.historyTickets} 个验证 ticket；折线按开始、完成和失败时间聚合。` : "当前历史窗口没有足够的时间样本，图表会随验证事件进入后自动增长。"}</Typography></HubPanel></Grid><Grid size={{ xs: 12, lg: 5 }}><HubPanel title="当前 Failure 占比"><FailureDonut open={analytics.failure.open} fixed={analytics.failure.fixed} ratio={analytics.failure.ratio} loading={!analytics.failure.historyReady} /><Typography variant="caption" color="text.secondary">{!analytics.failure.historyReady ? "Failure 历史仍在采样；不会把未加载状态显示为零。" : analytics.failure.averageResolutionSeconds === null ? "尚无可计算的闭合链路耗时。" : `已闭合 ${analytics.failure.resolvedCount} 条链路，平均处理 ${formatDuration(analytics.failure.averageResolutionSeconds)}。`}</Typography></HubPanel></Grid></Grid>
    <Box className="dashboard-kpi-grid">{metrics.map(([label, value]) => <DashboardKpi key={label} label={label} value={value} detail={metricDetails[label] ?? ""} tone={metricTone(label, value)} />)}</Box>
    <Grid container spacing={2}><Grid size={{ xs: 12, md: 6 }}><HubPanel title="流程处理耗时 · Top 8"><AnalyticsDurationChart rows={analytics.durations} /></HubPanel></Grid><Grid size={{ xs: 12, md: 6 }}><HubPanel title="24 小时任务时间轴"><AnalyticsSchedule slots={analytics.schedule} /></HubPanel></Grid></Grid>
    <Grid container spacing={2}><Grid size={{ xs: 12, md: 5 }}><HubPanel title="验证任务流转"><AnalyticsFlow rows={analytics.flow} /></HubPanel></Grid><Grid size={{ xs: 12, md: 7 }}><HubPanel title="模块目录占比"><ModuleShareChart rows={analytics.modules} /><Typography variant="caption" color="text.secondary">按计划路径的前两级目录归类，Failure 和验证 ticket 会共同计入对应模块。</Typography></HubPanel></Grid></Grid>
    <Grid container spacing={2}><Grid size={{ xs: 12, md: 6 }}><HubPanel title="Failure 链补丁状态"><PatchStatusChart rows={analytics.patches} /><Typography variant="caption" color="text.secondary">展示补丁从排队、应用到已应用/需 Rebase/失败的生命周期，便于核对失败链修复是否真正落盘。</Typography></HubPanel></Grid><Grid size={{ xs: 12, md: 6 }}><HubPanel title="数据覆盖"><Stack direction="row" spacing={1} sx={{ flexWrap: "wrap" }}><Chip size="small" variant="outlined" label={`${analytics.coverage.historyTickets} 条验证历史`} /><Chip size="small" variant="outlined" label={`${analytics.coverage.historyChains} 条 Failure 链`} /><Chip size="small" variant="outlined" label={`${analytics.coverage.auditEvents} 条事件`} /><Chip size="small" color={analytics.coverage.hasTimeSeries ? "success" : "warning"} label={analytics.coverage.hasTimeSeries ? "时间序列已采样" : "等待时间样本"} /></Stack><Typography variant="caption" color="text.secondary">历史报表按最近加载窗口计算；当前快照用于补足正在运行的任务和资源占用。</Typography></HubPanel></Grid></Grid>
    <Grid container spacing={2}><Grid size={{ xs: 12, lg: 7 }}><HubPanel title="计划完成 / Todo / Failure"><PlanProgressChart rows={analytics.plans} /></HubPanel></Grid><Grid size={{ xs: 12, lg: 5 }}><HubPanel title="当前 CPU / 验证任务"><ActiveTaskTable rows={analytics.activeTasks} /></HubPanel></Grid></Grid>
    <HubPanel title={`最近流程 · ${analytics.inProgressWorkflowCount} 个处理中 · ${analytics.queuedWorkflowCount} 个已登记`}><RecentWorkflowList rows={analytics.recentWorkflows} /></HubPanel>
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, lg: 7 }}><HubPanel title="工作分布 · 只显示下一步动作"><Grid container spacing={1.25}>{board.map((lane) => <Grid key={lane.key} size={{ xs: 12, sm: 6 }}><Box className="dashboard-flow-node"><Stack direction="row" spacing={1} sx={{ alignItems: "baseline", mb: 1 }}><Typography variant="subtitle2" sx={{ flex: 1 }}>{lane.title}</Typography><Typography variant="h6">{lane.total}</Typography></Stack>{lane.cards.slice(0, 3).map((card) => <Stack key={card.id} spacing={0.2} sx={{ borderLeft: 3, borderColor: lane.key === "intervention" ? "error.main" : lane.key === "waiting" ? "warning.main" : "primary.main", pl: 1, mb: 1 }}><Typography variant="body2" sx={{ overflowWrap: "anywhere" }}>{card.title}</Typography><Typography variant="caption" color="text.secondary" sx={{ overflowWrap: "anywhere" }}>{card.detail}</Typography></Stack>)}{lane.cards.length === 0 && <Typography variant="caption" color="text.secondary">{lane.emptyText}</Typography>}{lane.overflowCount > 0 && <Typography variant="caption" color="text.secondary">另有 {lane.overflowCount} 项，转到详情页查看。</Typography>}</Box></Grid>)}</Grid></HubPanel></Grid>
      <Grid size={{ xs: 12, lg: 5 }}><HubPanel title="介入优先级"><Stack spacing={1.5}>{intervention.next === null ? <Typography>没有开放 Failure；不需要额外介入。</Typography> : <><Stack spacing={0.25}><Typography variant="caption" color="text.secondary">下一项责任计划</Typography><Typography variant="body1" sx={{ fontWeight: 700, overflowWrap: "anywhere" }}>{intervention.next.fixingPlan}</Typography><Typography variant="body2">{intervention.next.summary}</Typography></Stack><SignalBar label="开放 Failure" value={intervention.failureCount} total={Math.max(intervention.failureCount, intervention.planCount)} detail={`${intervention.planCount} 个责任计划，逐计划收敛 WIP`} tone="error.main" /></>}{intervention.waitingSessionCount > 0 || intervention.pendingReservationCount > 0 ? <Typography variant="caption" color="text.secondary">验证等待 {intervention.waitingSessionCount} 个 Session，CPU 队列 {intervention.pendingReservationCount}{intervention.nextReservation ? `；下一个 ${intervention.nextReservation.sessionId}（#${intervention.nextReservation.queuePosition} · ${intervention.nextReservation.executionMode === "warm" ? "热缓存" : "隔离突发"}）` : ""}。验证等待不阻塞代码工作。</Typography> : null}{continuationCoverage && <Typography variant="caption" color="text.secondary">{continuationCoverage}</Typography>}</Stack></HubPanel></Grid>
    </Grid>
    <Grid container spacing={2}><Grid size={{ xs: 12, md: 6 }}><HubPanel title={admission.title}><Typography>{admission.detail}</Typography><Typography variant="caption" color="text.secondary">{sessionLivenessSummary(visibleSnapshot)}</Typography></HubPanel></Grid><Grid size={{ xs: 12, md: 6 }}><HubPanel title="运行环境信号"><Stack spacing={1}><SignalBar label="验证流速" value={activeCount} total={Math.max(activeCount + pendingCount, 1)} detail={flowHealth.map(validationFlowSummary).join("；") || "没有活动验证槽；新的受管验证可立即申请。"} /><Typography variant="caption" color="text.secondary">CPU 突发 WIP：{cpuBurst.active}/{cpuBurst.capacity} · 可隔离检查 {cpuBurst.eligiblePending}</Typography><Typography variant="caption" color="text.secondary">无 target-dir 的 cargo check 或定向 cargo test --lib 会自动申请隔离突发候选；资源不足时保持热缓存 FIFO。</Typography></Stack></HubPanel></Grid></Grid>
    <Grid container spacing={2}><Grid size={{ xs: 12, md: 4 }}><HubPanel title={baseline.title}><Typography>{baseline.detail}</Typography></HubPanel></Grid><Grid size={{ xs: 12, md: 4 }}><HubPanel title={cleanup.title}><Typography>{cleanup.detail}</Typography></HubPanel></Grid><Grid size={{ xs: 12, md: 4 }}><HubPanel title="协调器同步"><Typography>{syncHealth.detail}</Typography></HubPanel></Grid></Grid>
    <HubPanel title="当前资源等待 · 仅影响独占验证">{blockers.length === 0 ? <Typography>没有独占资源等待；其他 Session 可继续运行。</Typography> : <Stack spacing={1}>{blockers.map((blocker) => <Typography key={`${blocker.kind}:${blocker.ownerSessionId}:${blocker.createdAt}`}>{resourceBlockerSummary(blocker)}</Typography>)}</Stack>}</HubPanel>
    <HubPanel title="局部等待时的续作">{continuations.length === 0 ? <Typography>没有可推荐的同计划续作；先处理介入方式的建议项或其它未冲突代码，完成后优先回到主任务。</Typography> : <Stack spacing={1}>{continuations.map((continuation) => <Stack key={continuation.sessionId} spacing={0.25} sx={{ borderLeft: 2, borderColor: "primary.main", pl: 1 }}><Typography>不要等待：{continuationWaitInstruction(continuation.waitKind)}，{continuation.kind === "unowned_failure" ? "拉取无人负责的代码 Failure" : "先做"} {continuation.milestone} · {continuation.title}</Typography>{continuation.kind === "unowned_failure" ? <Typography variant="caption" color="text.secondary">{continuation.targetPlanPath}</Typography> : null}<Typography variant="caption" color="text.secondary">{continuation.kind === "unowned_failure" ? "每个 Session 一次只拉取一个责任计划；先领取作用域，不占用其它 Session 或验证 FIFO；完成后优先回到主任务。" : "先领取作用域；完成后优先回到主任务，不扩散为跨计划 WIP。"}</Typography></Stack>)}</Stack>}</HubPanel>
  </Stack>;
}

export function overviewMetrics(snapshot: ControlSnapshot) {
  const projection = experience(snapshot);
  return [["工作流", snapshot.workflows.length], ["活动会话", snapshot.sessions.filter((item) => item.status === "active").length], ["Failure", interventionGuidance(snapshot).failureCount], ["运行验证", snapshot.validation.currentCargoTargets.filter((item) => item.status === "running").length], ["同步状态", syncHealthSummary(snapshot).headline], ["资源阻塞", resourceBlockers(snapshot).length]] as const;
}

export interface SyncHealthSummary {
  headline: string;
  detail: string;
}

export function syncHealthSummary(snapshot: ControlSnapshot): SyncHealthSummary {
  const trend = experience(snapshot).sync;
  const trendDetail = `24 小时趋势：${trend.quietRuns}/${trend.runs} 安静同步，${trend.visibleChanges} 项可见变更。`;
  const latest = snapshot.codexSessions?.lastRun;
  if (latest === null || latest === undefined) {
    return { headline: "未采样", detail: `尚无最近一次同步记录；${trendDetail}` };
  }
  if (latest.status !== "succeeded" || latest.diagnosticCount > 0 || latest.unavailableCount > 0) {
    return {
      headline: "需关注",
      detail: `最近一次同步为 ${latest.status}：扫描 ${latest.scannedCount} 项，诊断 ${latest.diagnosticCount}、不可用 ${latest.unavailableCount}；${trendDetail}`,
    };
  }
  if (latest.changedCount === 0) {
    return {
      headline: "安静",
      detail: `最近一次安静同步：扫描 ${latest.scannedCount} 项，用时 ${latest.durationMs}ms；${trendDetail}`,
    };
  }
  return {
    headline: `+${latest.changedCount}`,
    detail: `最近一次同步有 ${latest.changedCount} 项可见变更：扫描 ${latest.scannedCount} 项，用时 ${latest.durationMs}ms；${trendDetail}`,
  };
}

export interface AdmissionSummary {
  title: string;
  detail: string;
}

export function sessionLivenessSummary(snapshot: ControlSnapshot): string {
  const ttlSeconds = snapshot.service?.sessionTtlSeconds ?? 600;
  const duration = ttlSeconds >= 3600 && ttlSeconds % 3600 === 0
    ? `${ttlSeconds / 3600} 小时`
    : `${Math.round(ttlSeconds / 60)} 分钟`;
  return `业务 Session 活跃窗口 ${duration}；资源租约和预约 TTL 仍独立回收。`;
}

export interface CleanupDebtSummary {
  title: string;
  detail: string;
}

export interface InterventionGuidance {
  failureCount: number;
  planCount: number;
  next: { summary: string; fixingPlan: string } | null;
  waitingSessionCount: number;
  pendingReservationCount: number;
  nextReservation: { sessionId: string; queuePosition: number; executionMode: "warm" | "burst" } | null;
}

export interface ContinuationGuidance {
  sessionId: string;
  waitKind: "validation" | "lease" | "external";
  kind: "same_plan" | "unowned_failure";
  targetPlanPath: string;
  milestone: string;
  title: string;
  returnToPrimary: boolean;
}

export function continuationGuidance(snapshot: ControlSnapshot): ContinuationGuidance[] {
  return (experience(snapshot).continuations ?? []).map((continuation) => ({
    sessionId: continuation.sessionId,
    waitKind: continuation.waitKind,
    kind: continuation.candidate.kind ?? "same_plan",
    targetPlanPath: continuation.candidate.planPath ?? continuation.planPath,
    milestone: continuation.candidate.milestone,
    title: continuation.candidate.title,
    returnToPrimary: continuation.returnToPrimary,
  }));
}

export function continuationCoverageSummary(snapshot: ControlSnapshot): string | null {
  const waiting = interventionGuidance(snapshot).waitingSessionCount;
  if (waiting === 0) return null;
  const covered = Math.min(
    waiting,
    new Set(
      continuationGuidance(snapshot)
        .filter((continuation) => continuation.waitKind === "validation")
        .map((continuation) => continuation.sessionId),
    ).size,
  );
  if (covered === waiting) return `验证等待代码续作覆盖 ${covered}/${waiting}；可继续编码。`;
  return `验证等待代码续作覆盖 ${covered}/${waiting}；另有 ${waiting - covered} 个暂未找到安全代码续作。`;
}

export interface ValidationLaneFlowHealth {
  laneScope: "cpu" | "gpu";
  activeCount: number;
  queuedCount: number;
  nextSessionId: string | null;
  oldestQueuedMinutes: number | null;
}

export function validationFlowHealth(
  snapshot: ControlSnapshot,
  now = new Date(),
): ValidationLaneFlowHealth[] {
  const reservations = (snapshot.validation?.cargoReservations ?? [])
    .filter((reservation) => reservation.executionMode !== "burst");
  const byLane = new Map<"cpu" | "gpu", typeof reservations>();
  for (const reservation of reservations) {
    const rows = byLane.get(reservation.laneScope) ?? [];
    rows.push(reservation);
    byLane.set(reservation.laneScope, rows);
  }
  return [...byLane.entries()].map(([laneScope, rows]) => {
    const pending = rows.filter((reservation) => reservation.status === "pending")
      .sort((left, right) => left.queuePosition - right.queuePosition || left.reservationId.localeCompare(right.reservationId));
    const waits = pending.map((reservation) => queueWaitMinutes(reservation.createdAt, now))
      .filter((minutes): minutes is number => minutes !== null);
    return {
      laneScope,
      activeCount: rows.length - pending.length,
      queuedCount: pending.length,
      nextSessionId: pending[0]?.sessionId ?? null,
      oldestQueuedMinutes: waits.length === 0 ? null : Math.max(...waits),
    };
  }).sort((left, right) => left.laneScope.localeCompare(right.laneScope));
}

export function queueWaitMinutes(createdAt: string, now = new Date()): number | null {
  const created = Date.parse(createdAt);
  if (!Number.isFinite(created)) return null;
  return Math.max(0, Math.floor((now.getTime() - created) / 60_000));
}

export function validationFlowSummary(lane: ValidationLaneFlowHealth): string {
  const label = lane.laneScope === "cpu" ? "CPU 热缓存" : "GPU";
  if (lane.queuedCount === 0) return `${label}：运行 ${lane.activeCount} · 没有排队`;
  const age = lane.oldestQueuedMinutes === null ? "等待时间未知" : `最久等待 ${lane.oldestQueuedMinutes} 分钟`;
  return `${label}：运行 ${lane.activeCount} · 排队 ${lane.queuedCount} · 下一个 ${lane.nextSessionId ?? "—"} · ${age}`;
}

export function interventionGuidance(snapshot: ControlSnapshot): InterventionGuidance {
  const authoritative = experience(snapshot).intervention;
  if (authoritative) {
    return {
      failureCount: authoritative.openFailureCount,
      planCount: authoritative.responsiblePlanCount,
      next: authoritative.suggestedNext ? {
        summary: authoritative.suggestedNext.summary,
        fixingPlan: authoritative.suggestedNext.planPath,
      } : null,
      waitingSessionCount: authoritative.validation.waitingSessionCount,
      pendingReservationCount: authoritative.validation.pendingReservationCount,
      nextReservation: authoritative.validation.nextReservation,
    };
  }
  const failures = [...(snapshot.failures?.nodes ?? [])]
    .filter((failure) => failure.status === "open")
    .sort((left, right) => (
      left.priority - right.priority
      || left.created_at.localeCompare(right.created_at)
      || left.node_id - right.node_id
    ));
  const fixingPlan = (failure: { fixing_plan: string; origin_plan: string; artifact_path: string }) => (
    failure.fixing_plan || failure.origin_plan || failure.artifact_path
  );
  const next = failures[0];
  return {
    failureCount: failures.length,
    planCount: new Set(failures.map(fixingPlan)).size,
    next: next ? { summary: next.summary_slug, fixingPlan: fixingPlan(next) } : null,
    waitingSessionCount: 0,
    pendingReservationCount: 0,
    nextReservation: null,
  };
}

export function admissionSummary(snapshot: ControlSnapshot): AdmissionSummary {
  const mode = snapshot.service?.mode;
  if (mode && mode !== "read_write") {
    return {
      title: "Session 准入待恢复",
      detail: `协调器当前为 ${mode} 模式；请先恢复服务写入能力。`,
    };
  }
  const blockers = resourceBlockers(snapshot);
  if (blockers.length > 0) {
    return {
      title: "Session 准入开放",
      detail: `${blockers.length} 条独占验证通道正在使用；仅等待该通道，其他 Session 不排空、不暂停。`,
    };
  }
  return {
    title: "Session 准入开放",
    detail: "没有独占验证通道等待；其他 Session 可立即继续运行。",
  };
}

export interface WorkspaceBaselineSummary {
  title: string;
  detail: string;
}

export function workspaceBaselineSummary(snapshot: ControlSnapshot): WorkspaceBaselineSummary {
  const baseline = snapshot.collaboration?.baseline;
  if (baseline?.health === "degraded") {
    const reason = baseline.degraded_reason ?? "存在未归属的工作树改动";
    return {
      title: "共享工作树存在未归属改动",
      detail: `${reason}；这只影响全局工作树对账，Session 准入和已归属作用域提交保持开放。`,
    };
  }
  return {
    title: "共享工作树基线正常",
    detail: "全局工作树对账正常；Session 准入和已归属作用域提交保持开放。",
  };
}

export function cleanupDebtSummary(snapshot: ControlSnapshot): CleanupDebtSummary {
  const lifecycle = snapshot.validation?.artifactLifecycle ?? {
    reusablePools: 0,
    ephemeralTargets: 0,
    pendingCleanup: 0,
    failedCleanup: 0,
  };
  const title = lifecycle.failedCleanup > 0
    ? "构建产物回收需处理"
    : lifecycle.pendingCleanup > 0
      ? "构建产物回收排队"
      : "构建产物回收正常";
  return {
    title,
    detail: `${lifecycle.reusablePools} 个可复用池、${lifecycle.ephemeralTargets} 个临时产物；${lifecycle.pendingCleanup} 个待清理、${lifecycle.failedCleanup} 个清理失败。请在验证详情处理，Session 准入保持开放。`,
  };
}

export function resourceBlockers(snapshot: ControlSnapshot): ExperienceProjection["blockers"] {
  const targets = snapshot.validation?.currentCargoTargets ?? [];
  const live: ExperienceProjection["blockers"] = [];
  for (const target of targets) {
    if (target.status !== "running" && target.status !== "leased") continue;
    live.push({
      kind: "cargo" as const,
      ownerSessionId: target.session_id,
      laneKind: target.lane_kind,
      status: target.status,
      createdAt: target.created_at,
    });
  }
  return live.length > 0 ? live : experience(snapshot).blockers;
}

export function resourceBlockerSummary(
  blocker: ExperienceProjection["blockers"][number],
  now = new Date(),
): string {
  const state = blocker.status === "running" ? "运行中" : "已预约";
  const startedAt = Date.parse(blocker.createdAt);
  if (!Number.isFinite(startedAt)) {
    return `${blocker.laneKind} 通道由 ${blocker.ownerSessionId} ${state}（开始于 ${blocker.createdAt}）`;
  }
  const elapsedMinutes = Math.max(0, Math.floor((now.getTime() - startedAt) / 60_000));
  return `${blocker.laneKind} 通道由 ${blocker.ownerSessionId} ${state}（已运行 ${elapsedMinutes} 分钟）`;
}

const BOARD_CARD_LIMIT = 8;

export type WorkBoardLaneKey = "ready" | "waiting" | "attention" | "intervention";

export interface WorkBoardCard {
  id: string;
  title: string;
  detail: string;
}

export interface WorkBoardLane {
  key: WorkBoardLaneKey;
  title: string;
  emptyText: string;
  cards: WorkBoardCard[];
  total: number;
  overflowCount: number;
}

export function workBoard(snapshot: ControlSnapshot): WorkBoardLane[] {
  const continuationBySession = new Map(
    continuationGuidance(snapshot).map((continuation) => [continuation.sessionId, continuation]),
  );
  const queuedValidationSessions = new Set(
    (snapshot.validation?.cargoReservations ?? [])
      .filter((reservation) => reservation.status === "pending")
      .map((reservation) => reservation.sessionId),
  );
  const continuationCards = snapshot.sessions
    .filter((session) => (
      continuationBySession.has(session.sessionId)
      && ["active", "registered", "waiting_lease", "waiting_validation"].includes(session.status)
    ))
    .map((session) => continuationSessionCard(session, continuationBySession.get(session.sessionId)!));
  const ordinaryReadyCards = [
    ...snapshot.sessions
      .filter((session) => (
        ["active", "registered"].includes(session.status)
        && !continuationBySession.has(session.sessionId)
      ))
      .map(sessionCard),
  ];
  const readyCards = [...continuationCards, ...ordinaryReadyCards];
  const waitingCards = snapshot.sessions
    .filter((session) => (
      ["waiting_lease", "waiting_validation", "finalizing"].includes(session.status)
      && !continuationBySession.has(session.sessionId)
    ))
    .map((session) => waitingSessionCard(session, queuedValidationSessions.has(session.sessionId)));
  return [
    boundedLane("ready", "可继续", "当前没有可继续的业务 Session。", readyCards),
    boundedLane("waiting", "局部等待", "没有局部等待的 Session。", waitingCards),
    sessionLane("attention", "需关注", "没有需要恢复的 Session。", snapshot.sessions, ["resolving_failure", "stale"]),
    failureLane(snapshot),
  ];
}

function sessionLane(
  key: Exclude<WorkBoardLaneKey, "intervention">,
  title: string,
  emptyText: string,
  sessions: SessionProjection[],
  statuses: readonly string[],
): WorkBoardLane {
  const cards = sessions.filter((session) => statuses.includes(session.status)).map(sessionCard);
  return boundedLane(key, title, emptyText, cards);
}

function failureLane(snapshot: ControlSnapshot): WorkBoardLane {
  if (snapshot.failures.nodes.length === 0) {
    const intervention = interventionGuidance(snapshot);
    const cards = intervention.next ? [{
      id: `failure-plan:${intervention.next.fixingPlan}`,
      title: intervention.next.fixingPlan,
      detail: `${intervention.failureCount} 个开放 Failure · ${intervention.next.summary}`,
    }] : [];
    const lane = boundedLane("intervention", "需介入", "没有未关闭的 Failure。", cards);
    return {
      ...lane,
      total: intervention.planCount,
      overflowCount: Math.max(intervention.planCount - cards.length, 0),
    };
  }
  const plans = new Map<string, { count: number; priority: number; createdAt: string; summary: string }>();
  for (const failure of snapshot.failures.nodes) {
    if (failure.status !== "open") continue;
    const planPath = failure.fixing_plan || failure.origin_plan || failure.artifact_path;
    const current = plans.get(planPath);
    if (current === undefined) {
      plans.set(planPath, {
        count: 1,
        priority: failure.priority,
        createdAt: failure.created_at,
        summary: failure.summary_slug,
      });
      continue;
    }
    current.count += 1;
    if (failure.priority < current.priority || (failure.priority === current.priority && failure.created_at < current.createdAt)) {
      current.priority = failure.priority;
      current.createdAt = failure.created_at;
      current.summary = failure.summary_slug;
    }
  }
  const cards = [...plans.entries()]
    .sort(([, left], [, right]) => left.priority - right.priority || left.createdAt.localeCompare(right.createdAt))
    .map(([planPath, group]) => ({
      id: `failure-plan:${planPath}`,
      title: planPath,
      detail: `P${group.priority} · ${group.count} 个开放 Failure · ${group.summary}`,
    }));
  return boundedLane("intervention", "需介入", "没有未关闭的 Failure。", cards);
}

function sessionCard(session: SessionProjection): WorkBoardCard {
  return {
    id: `session:${session.sessionId}`,
    title: session.displayName || session.sessionId,
    detail: session.statusReason || session.planPath || "等待新的工作项",
  };
}

function waitingSessionCard(session: SessionProjection, validationQueued: boolean): WorkBoardCard {
  const waitLabel = session.status === "waiting_validation"
    ? (validationQueued ? "验证已排队" : "外部条件等待")
    : session.status === "waiting_lease"
      ? "文件作用域等待"
      : "准备提交";
  return {
    id: `session:${session.sessionId}`,
    title: session.displayName || session.sessionId,
    detail: session.statusReason ? `${waitLabel}；${session.statusReason}` : waitLabel,
  };
}

function continuationSessionCard(
  session: SessionProjection,
  continuation: ContinuationGuidance,
): WorkBoardCard {
  const waitLabel = continuationWaitLabel(continuation.waitKind);
  return {
    id: `session:${session.sessionId}`,
    title: session.displayName || session.sessionId,
    detail: `${waitLabel}；可继续 ${continuation.milestone} · ${continuation.title}`,
  };
}

function continuationWaitLabel(waitKind: ContinuationGuidance["waitKind"]): string {
  if (waitKind === "validation") return "验证已排队";
  if (waitKind === "lease") return "文件作用域等待";
  return "外部条件等待";
}

function continuationWaitInstruction(waitKind: ContinuationGuidance["waitKind"]): string {
  if (waitKind === "validation") return "验证通道排队时";
  if (waitKind === "lease") return "文件作用域等待时";
  return "外部条件等待时";
}

function boundedLane(
  key: WorkBoardLaneKey,
  title: string,
  emptyText: string,
  cards: WorkBoardCard[],
): WorkBoardLane {
  return {
    key,
    title,
    emptyText,
    cards: cards.slice(0, BOARD_CARD_LIMIT),
    total: cards.length,
    overflowCount: Math.max(cards.length - BOARD_CARD_LIMIT, 0),
  };
}

function experience(snapshot: ControlSnapshot) {
  return snapshot.experience ?? {
    sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 },
    blockers: [],
    continuations: [],
  };
}
