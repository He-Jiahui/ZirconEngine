import { Grid, Stack, Typography } from "@mui/material";
import type { ControlSnapshot, ExperienceProjection, SessionProjection } from "../api/contracts";
import { HubPanel } from "../theme";

export function OverviewPage({ snapshot }: { snapshot: ControlSnapshot }) {
  const metrics = overviewMetrics(snapshot);
  const blockers = resourceBlockers(snapshot);
  const admission = admissionSummary(snapshot);
  const cleanup = cleanupDebtSummary(snapshot);
  const intervention = interventionGuidance(snapshot);
  const board = workBoard(snapshot);
  const flowHealth = validationFlowHealth(snapshot);
  const syncHealth = syncHealthSummary(snapshot);
  const continuations = continuationGuidance(snapshot);
  const cpuBurst = snapshot.validation?.cpuBurst ?? { capacity: 1, active: 0, eligiblePending: 0 };
  return <Stack spacing={2}>
    <Grid container spacing={2}>{metrics.map(([label, value]) => <Grid key={label} size={{ xs: 12, sm: 6, lg: 3 }}><HubPanel title={label}><Typography variant="h4">{value}</Typography></HubPanel></Grid>)}</Grid>
    <Grid container spacing={2}>{board.map((lane) => <Grid key={lane.key} size={{ xs: 12, md: 6, lg: 3 }}><HubPanel title={`${lane.title} · ${lane.total}`}><Stack spacing={1}>
      {lane.cards.length === 0 ? <Typography color="text.secondary">{lane.emptyText}</Typography> : lane.cards.map((card) => <Stack key={card.id} spacing={0.25} sx={{ borderLeft: 2, borderColor: "primary.main", pl: 1 }}><Typography variant="body2">{card.title}</Typography><Typography variant="caption" color="text.secondary">{card.detail}</Typography></Stack>)}
      {lane.overflowCount > 0 ? <Typography variant="caption" color="text.secondary">另有 {lane.overflowCount} 项，转到详情页查看。</Typography> : null}
    </Stack></HubPanel></Grid>)}</Grid>
    <HubPanel title="介入方式">
      {intervention.next === null ? <Typography>没有开放 Failure；不需要额外介入。</Typography> : <Stack spacing={0.5}>
        <Typography>{intervention.failureCount} 个 Failure 归属 {intervention.planCount} 个责任计划；一次只拉取一个责任计划，避免跨模块 WIP。</Typography>
        <Typography>建议先处理：{intervention.next.summary}</Typography>
        <Typography variant="caption" color="text.secondary">{intervention.next.fixingPlan}</Typography>
      </Stack>}
    </HubPanel>
    <HubPanel title={admission.title}>
      <Typography>{admission.detail}</Typography>
      <Typography variant="caption" color="text.secondary">{sessionLivenessSummary(snapshot)}</Typography>
    </HubPanel>
    <HubPanel title={cleanup.title}>
      <Typography>{cleanup.detail}</Typography>
    </HubPanel>
    <HubPanel title="协调器同步">
      <Typography>{syncHealth.detail}</Typography>
    </HubPanel>
    <HubPanel title="验证流速 · 仅影响独占资源">
      {flowHealth.length === 0 ? <Typography>没有活动验证槽；新的受管验证可立即申请。</Typography> : <Stack spacing={0.5}>
        {flowHealth.map((lane) => <Typography key={lane.laneScope}>{validationFlowSummary(lane)}</Typography>)}
      </Stack>}
      <Typography>CPU 突发 WIP：{cpuBurst.active}/{cpuBurst.capacity} · 可隔离检查 {cpuBurst.eligiblePending}</Typography>
      <Typography variant="caption" color="text.secondary">热缓存队列与隔离突发只限制验证资源；不会关闭 Session 准入或暂停文件工作。</Typography>
    </HubPanel>
    <HubPanel title="当前资源等待 · 仅影响独占验证">
      {blockers.length === 0 ? <Typography>没有独占资源等待；其他 Session 可继续运行。</Typography> : <Stack spacing={1}>{blockers.map((blocker) => <Typography key={`${blocker.kind}:${blocker.ownerSessionId}:${blocker.createdAt}`}>{resourceBlockerSummary(blocker)}</Typography>)}</Stack>}
    </HubPanel>
    <HubPanel title="验证等待时的续作">
      {continuations.length === 0 ? <Typography>没有可推荐的同计划续作；验证结束后直接回到主任务。</Typography> : <Stack spacing={1}>{continuations.map((continuation) => <Stack key={continuation.sessionId} spacing={0.25} sx={{ borderLeft: 2, borderColor: "primary.main", pl: 1 }}>
        <Typography>不要等待：{continuation.waitKind === "validation" ? "验证通道" : "文件作用域"}局部等待期间，先做 {continuation.milestone} · {continuation.title}</Typography>
        <Typography variant="caption" color="text.secondary">先领取作用域；完成后优先回到主任务，不扩散为跨计划 WIP。</Typography>
      </Stack>)}</Stack>}
    </HubPanel>
  </Stack>;
}

export function overviewMetrics(snapshot: ControlSnapshot) {
  const projection = experience(snapshot);
  return [["工作流", snapshot.workflows.length], ["活动会话", snapshot.sessions.filter((item) => item.status === "active").length], ["Failure", snapshot.failures.nodes.length], ["运行验证", snapshot.validation.currentCargoTargets.filter((item) => item.status === "running").length], ["同步状态", syncHealthSummary(snapshot).headline], ["资源阻塞", resourceBlockers(snapshot).length]] as const;
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
  const minutes = Math.round(ttlSeconds / 60);
  return `业务 Session 活跃窗口 ${minutes} 分钟；资源租约和预约 TTL 仍独立回收。`;
}

export interface CleanupDebtSummary {
  title: string;
  detail: string;
}

export interface InterventionGuidance {
  failureCount: number;
  planCount: number;
  next: { summary: string; fixingPlan: string } | null;
}

export interface ContinuationGuidance {
  sessionId: string;
  waitKind: "validation" | "lease";
  milestone: string;
  title: string;
  returnToPrimary: boolean;
}

export function continuationGuidance(snapshot: ControlSnapshot): ContinuationGuidance[] {
  return (experience(snapshot).continuations ?? []).map((continuation) => ({
    sessionId: continuation.sessionId,
    waitKind: continuation.waitKind,
    milestone: continuation.candidate.milestone,
    title: continuation.candidate.title,
    returnToPrimary: continuation.returnToPrimary,
  }));
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
  return [
    sessionLane("ready", "可继续", "当前没有可继续的业务 Session。", snapshot.sessions, ["active", "registered"]),
    sessionLane("waiting", "等待资源", "没有等待独占资源的 Session。", snapshot.sessions, ["waiting_lease", "waiting_validation", "finalizing"]),
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
  const cards = snapshot.failures.nodes
    .filter((failure) => failure.status === "open")
    .map((failure) => ({
      id: `failure:${failure.node_id}`,
      title: failure.summary_slug,
      detail: failure.fixing_plan || failure.origin_plan || failure.artifact_path,
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
