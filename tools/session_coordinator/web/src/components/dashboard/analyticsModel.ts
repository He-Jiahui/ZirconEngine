import type {
  AuditEvent,
  CargoLaneProjection,
  ControlSnapshot,
  FailureHistoryProjection,
  PatchProjection,
  ValidationHistoryProjection,
} from "../../api/contracts";

export interface AnalyticsBucket {
  label: string;
  hour: number;
  completed: number;
  failed: number;
  started: number;
  events: number;
}

export interface AnalyticsDurationRow {
  id: string;
  label: string;
  detail: string;
  seconds: number;
  status: "passed" | "failed" | "running" | "unknown";
}

export interface AnalyticsScheduleSlot {
  hour: number;
  label: string;
  queued: number;
  running: number;
  completed: number;
  failed: number;
}

export interface AnalyticsModuleShare {
  label: string;
  count: number;
  ratio: number;
}

export interface AnalyticsPlanProgress {
  id: string;
  label: string;
  workflowCount: number;
  nodeCount: number;
  completedCount: number;
  todoCount: number;
  failedCount: number;
  ratio: number;
  status: "complete" | "blocked" | "in_progress" | "queued";
}

export interface AnalyticsActiveTask {
  id: string;
  sessionId: string;
  lane: string;
  status: string;
  elapsedSeconds: number | null;
}

export interface AnalyticsRecentWorkflow {
  id: string;
  label: string;
  planPath: string;
  state: string;
  updatedAt: string;
  progress: number;
  failedCount: number;
}

export interface AnalyticsPatchStatus {
  label: string;
  value: number;
  tone: "info" | "warning" | "success" | "danger" | "neutral";
}

export interface AnalyticsFailureReason {
  code: string;
  count: number;
  phase: string | null;
}

export interface AnalyticsValidationReport {
  loaded: boolean;
  total: number | null;
  terminal: number | null;
  successRate: number | null;
  backlog: number | null;
  statuses: {
    queued: number | null;
    materializing: number | null;
    running: number | null;
    passed: number | null;
    failed: number | null;
    snapshotStale: number | null;
  };
  last24Hours: {
    started: number;
    passed: number;
    failed: number;
    successRate: number | null;
  };
  sampleSize: number;
  sampleTruncated: boolean;
  eventDetailsTruncated: number;
  newestUpdatedAt: string | null;
  oldestCreatedAt: string | null;
  unclassifiedFailures: number;
  failureReasons: AnalyticsFailureReason[];
}

export interface AnalyticsModel {
  buckets: AnalyticsBucket[];
  schedule: AnalyticsScheduleSlot[];
  durations: AnalyticsDurationRow[];
  failure: { open: number; fixed: number; ratio: number; averageResolutionSeconds: number | null; resolvedCount: number; historyReady: boolean };
  flow: Array<{ label: string; value: number; tone: "info" | "warning" | "success" | "danger" | "neutral" }>;
  modules: AnalyticsModuleShare[];
  plans: AnalyticsPlanProgress[];
  activeTasks: AnalyticsActiveTask[];
  recentWorkflows: AnalyticsRecentWorkflow[];
  inProgressWorkflowCount: number;
  queuedWorkflowCount: number;
  patches: AnalyticsPatchStatus[];
  validationReport: AnalyticsValidationReport;
  coverage: { historyTickets: number; historyChains: number; auditEvents: number; hasTimeSeries: boolean };
}

export interface AnalyticsSources {
  validationHistory?: ValidationHistoryProjection | null;
  failureHistory?: FailureHistoryProjection | null;
  audit?: AuditEvent[];
  now?: Date;
}

const HOUR_MS = 60 * 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

export function buildAnalytics(snapshot: ControlSnapshot, sources: AnalyticsSources = {}): AnalyticsModel {
  const now = sources.now ?? new Date();
  const validation = snapshot.validation ?? {} as ControlSnapshot["validation"];
  const workflows = snapshot.workflows ?? [];
  const sessions = snapshot.sessions ?? [];
  const failures = snapshot.failures ?? { nodes: [], diagnostics: [] };
  const validationHistory = sources.validationHistory ?? null;
  const failureHistory = sources.failureHistory ?? null;
  const audit = sources.audit ?? snapshot.audit ?? [];
  const tickets = validationHistory?.tickets ?? [];
  const buckets = buildBuckets(tickets, audit, now);
  const validationReport = buildValidationReport(validationHistory, buckets);
  const schedule = buildSchedule({ ...snapshot, validation } as ControlSnapshot, tickets, now);
  const durations = buildDurations({ ...snapshot, validation } as ControlSnapshot, tickets, now);
  const failure = buildFailure({ ...snapshot, failures } as ControlSnapshot, failureHistory, now);
  const plans = buildPlans({ ...snapshot, workflows } as ControlSnapshot, tickets);
  const modules = buildModules({ ...snapshot, workflows, sessions, failures } as ControlSnapshot, tickets);
  const activeTasks = buildActiveTasks(validation.currentCargoTargets ?? [], now);
  const patches = buildPatchStatus(snapshot.collaboration?.patches ?? []);
  const recentWorkflows = workflows
    .slice()
    .sort((left, right) => timestamp(right.updatedAt) - timestamp(left.updatedAt))
    .slice(0, 8)
    .map((workflow) => ({
      id: workflow.runId,
      label: workflow.workflowKey || workflow.runId,
      planPath: workflow.planPath ?? "未关联计划",
      state: workflow.state,
      updatedAt: workflow.updatedAt,
      progress: workflow.nodeCount > 0 ? Math.round((workflow.succeededCount / workflow.nodeCount) * 100) : 0,
      failedCount: workflow.failedCount,
    }));
  return {
    buckets,
    schedule,
    durations,
    failure,
    flow: [
      { label: "排队", value: validationHistory?.statusCounts.queued ?? (validation.cargoReservations ?? []).filter((row) => row.status === "pending").length, tone: "warning" },
      { label: "物化", value: validationHistory?.statusCounts.materializing ?? (validation.validationCopies ?? []).filter((row) => row.status === "materialized").length, tone: "info" },
      { label: "运行", value: validationHistory?.statusCounts.running ?? activeTasks.length, tone: "info" },
      { label: "通过", value: validationHistory?.statusCounts.passed ?? (validation.currentCargoTargets ?? []).filter((row) => row.status === "succeeded").length, tone: "success" },
      { label: "失败", value: validationHistory?.statusCounts.failed ?? (validation.currentCargoTargets ?? []).filter((row) => row.status === "failed").length, tone: "danger" },
    ],
    modules,
    plans,
    activeTasks,
    recentWorkflows,
    inProgressWorkflowCount: workflows.filter((workflow) => ["active", "running", "in_progress", "resolving_failure", "finalizing", "waiting_validation", "waiting_lease"].includes(workflow.state.toLowerCase())).length,
    queuedWorkflowCount: workflows.filter((workflow) => ["registered", "queued", "pending"].includes(workflow.state.toLowerCase())).length,
    patches,
    validationReport,
    coverage: {
      historyTickets: validationHistory?.tickets.length ?? 0,
      historyChains: failureHistory?.chains.length ?? 0,
      auditEvents: audit.length,
      hasTimeSeries: buckets.some((bucket) => bucket.completed + bucket.failed + bucket.started > 0),
    },
  };
}

export function buildValidationReport(
  history: ValidationHistoryProjection | null,
  buckets: AnalyticsBucket[],
): AnalyticsValidationReport {
  const last24Hours = buckets.reduce(
    (total, bucket) => ({
      started: total.started + bucket.started,
      passed: total.passed + bucket.completed,
      failed: total.failed + bucket.failed,
      successRate: null,
    }),
    { started: 0, passed: 0, failed: 0, successRate: null as number | null },
  );
  const recentTerminal = last24Hours.passed + last24Hours.failed;
  last24Hours.successRate = recentTerminal > 0 ? last24Hours.passed / recentTerminal : null;
  if (history === null) {
    return {
      loaded: false,
      total: null,
      terminal: null,
      successRate: null,
      backlog: null,
      statuses: { queued: null, materializing: null, running: null, passed: null, failed: null, snapshotStale: null },
      last24Hours,
      sampleSize: 0,
      sampleTruncated: false,
      eventDetailsTruncated: 0,
      newestUpdatedAt: null,
      oldestCreatedAt: null,
      unclassifiedFailures: 0,
      failureReasons: [],
    };
  }

  const counts = history.statusCounts;
  const terminal = counts.passed + counts.failed;
  const failureCodes = new Map<string, { tickets: Set<string>; phases: Map<string, number> }>();
  let unclassifiedFailures = 0;
  for (const ticket of history.tickets) {
    const ticketCodes = new Set<string>();
    for (const event of ticket.events) {
      const code = event.errorCode?.trim();
      if (!code || ticketCodes.has(code)) continue;
      ticketCodes.add(code);
      const entry = failureCodes.get(code) ?? { tickets: new Set<string>(), phases: new Map<string, number>() };
      entry.tickets.add(ticket.ticketId);
      if (event.phase) entry.phases.set(event.phase, (entry.phases.get(event.phase) ?? 0) + 1);
      failureCodes.set(code, entry);
    }
    if (ticket.status === "failed" && ticketCodes.size === 0) unclassifiedFailures += 1;
  }
  const failureReasons = [...failureCodes.entries()]
    .map(([code, entry]) => ({
      code,
      count: entry.tickets.size,
      phase: [...entry.phases.entries()].sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))[0]?.[0] ?? null,
    }))
    .sort((left, right) => right.count - left.count || left.code.localeCompare(right.code))
    .slice(0, 6);
  const updatedTimes = history.tickets.map((ticket) => timestamp(ticket.updatedAt)).filter((value) => value > 0);
  const createdTimes = history.tickets.map((ticket) => timestamp(ticket.createdAt)).filter((value) => value > 0);

  return {
    loaded: true,
    total: Object.values(counts).reduce((sum, value) => sum + value, 0),
    terminal,
    successRate: terminal > 0 ? counts.passed / terminal : null,
    backlog: counts.queued + counts.materializing + counts.running,
    statuses: {
      queued: counts.queued,
      materializing: counts.materializing,
      running: counts.running,
      passed: counts.passed,
      failed: counts.failed,
      snapshotStale: counts.snapshot_stale,
    },
    last24Hours,
    sampleSize: history.tickets.length,
    sampleTruncated: history.truncated,
    eventDetailsTruncated: history.tickets.filter((ticket) => ticket.eventsTruncated).length,
    newestUpdatedAt: updatedTimes.length ? new Date(Math.max(...updatedTimes)).toISOString() : null,
    oldestCreatedAt: createdTimes.length ? new Date(Math.min(...createdTimes)).toISOString() : null,
    unclassifiedFailures,
    failureReasons,
  };
}

export function buildBuckets(tickets: ValidationHistoryProjection["tickets"], audit: AuditEvent[], now: Date): AnalyticsBucket[] {
  const current = now.getTime();
  const start = current - 23 * HOUR_MS;
  const buckets = Array.from({ length: 24 }, (_, hour) => ({
    label: `${String((now.getHours() - (23 - hour) + 24) % 24).padStart(2, "0")}时`, hour, completed: 0, failed: 0, started: 0, events: 0,
  }));
  for (const ticket of tickets) {
    const created = timestamp(ticket.createdAt);
    const updated = timestamp(ticket.updatedAt);
    if (created >= start && created <= current) buckets[bucketIndex(created, start)]!.started += 1;
    if (updated >= start && updated <= current) {
      const bucket = buckets[bucketIndex(updated, start)]!;
      if (ticket.status === "passed") bucket.completed += 1;
      if (ticket.status === "failed") bucket.failed += 1;
    }
  }
  for (const event of audit) {
    const created = timestamp(event.createdAt);
    if (created < start || created > current) continue;
    buckets[bucketIndex(created, start)]!.events += 1;
  }
  return buckets;
}

export function buildSchedule(snapshot: ControlSnapshot, tickets: ValidationHistoryProjection["tickets"], now: Date): AnalyticsScheduleSlot[] {
  const slots = Array.from({ length: 24 }, (_, hour) => ({ hour, label: `${String(hour).padStart(2, "0")}:00`, queued: 0, running: 0, completed: 0, failed: 0 }));
  const add = (date: string, key: "queued" | "running" | "completed" | "failed") => {
    const parsed = timestamp(date);
    if (parsed < now.getTime() - DAY_MS || parsed > now.getTime()) return;
    slots[new Date(parsed).getHours()]![key] += 1;
  };
  for (const ticket of tickets) {
    add(ticket.createdAt, "queued");
    add(ticket.updatedAt, ticket.status === "passed" ? "completed" : ticket.status === "failed" ? "failed" : ticket.status === "running" ? "running" : "queued");
  }
  for (const reservation of snapshot.validation?.cargoReservations ?? []) add(reservation.createdAt, reservation.status === "running" ? "running" : "queued");
  for (const job of snapshot.validation?.currentCargoTargets ?? []) {
    add(job.created_at, job.status === "running" || job.status === "leased" ? "running" : job.status === "succeeded" ? "completed" : job.status === "failed" ? "failed" : "queued");
  }
  return slots;
}

export function buildDurations(snapshot: ControlSnapshot, tickets: ValidationHistoryProjection["tickets"], now: Date): AnalyticsDurationRow[] {
  const rows = tickets.map((ticket) => {
    const seconds = Math.max(0, Math.floor((timestamp(ticket.updatedAt || ticket.createdAt) - timestamp(ticket.createdAt)) / 1000));
    return { id: ticket.ticketId, label: shortLabel(ticket.planPath || ticket.ticketId), detail: ticket.status, seconds, status: ticket.status === "passed" ? "passed" as const : ticket.status === "failed" ? "failed" as const : ticket.status === "running" || ticket.status === "materializing" ? "running" as const : "unknown" as const };
  });
  if (rows.length) return rows.sort((left, right) => right.seconds - left.seconds).slice(0, 8);
  return (snapshot.validation?.currentCargoTargets ?? []).slice().sort((left, right) => timestamp(right.created_at) - timestamp(left.created_at)).slice(0, 8).map((job) => ({
    id: job.job_id,
    label: shortLabel(job.session_id),
    detail: job.status,
    seconds: elapsedSeconds(job.started_at ?? job.created_at, job.finished_at ?? now.toISOString(), now),
    status: job.status === "succeeded" ? "passed" as const : job.status === "failed" ? "failed" as const : job.status === "running" || job.status === "leased" ? "running" as const : "unknown" as const,
  }));
}

export function buildFailure(snapshot: ControlSnapshot, history: FailureHistoryProjection | null, now: Date) {
  const nodes = snapshot.failures?.nodes ?? [];
  const currentOpen = nodes.filter((node) => node.status === "open").length || snapshot.experience?.intervention?.openFailureCount || 0;
  const open = history?.statusCounts.open ?? currentOpen;
  const fixed = history?.statusCounts.fixed ?? nodes.filter((node) => node.status === "fixed").length;
  const resolved = (history?.chains ?? nodes.filter((node) => node.status === "fixed").map((node) => ({ createdAt: node.created_at, resolvedAt: node.resolved_at })))
    .filter((row) => row.resolvedAt)
    .map((row) => Math.max(0, timestamp(row.resolvedAt!) - timestamp(row.createdAt)) / 1000)
    .filter(Number.isFinite);
  return { open, fixed, ratio: open + fixed > 0 ? open / (open + fixed) : 0, averageResolutionSeconds: resolved.length ? Math.round(resolved.reduce((sum, value) => sum + value, 0) / resolved.length) : null, resolvedCount: resolved.length, historyReady: history !== null };
}

export function buildPlans(snapshot: ControlSnapshot, tickets: ValidationHistoryProjection["tickets"]): AnalyticsPlanProgress[] {
  const plans = new Map<string, AnalyticsPlanProgress>();
  const ensure = (path: string) => {
    const id = path || "未关联计划";
    const current = plans.get(id);
    if (current) return current;
    const next: AnalyticsPlanProgress = { id, label: shortLabel(id), workflowCount: 0, nodeCount: 0, completedCount: 0, todoCount: 0, failedCount: 0, ratio: 0, status: "queued" };
    plans.set(id, next);
    return next;
  };
  for (const workflow of snapshot.workflows ?? []) {
    const row = ensure(workflow.planPath ?? "未关联计划");
    row.workflowCount += 1;
    row.nodeCount += workflow.nodeCount;
    row.completedCount += workflow.succeededCount;
    row.failedCount += workflow.failedCount;
  }
  for (const ticket of tickets) ensure(ticket.planPath);
  for (const row of plans.values()) {
    row.todoCount = Math.max(0, row.nodeCount - row.completedCount);
    row.ratio = row.nodeCount > 0 ? Math.round((row.completedCount / row.nodeCount) * 100) : 0;
    row.status = row.failedCount > 0 ? "blocked" : row.nodeCount > 0 && row.todoCount === 0 ? "complete" : row.nodeCount > 0 ? "in_progress" : "queued";
  }
  return [...plans.values()].sort((left, right) => right.failedCount - left.failedCount || left.ratio - right.ratio).slice(0, 10);
}

export function buildModules(snapshot: ControlSnapshot, tickets: ValidationHistoryProjection["tickets"]): AnalyticsModuleShare[] {
  const counts = new Map<string, number>();
  const add = (path: string | null | undefined) => {
    if (!path) return;
    const clean = path.replaceAll("\\", "/").split("/").filter(Boolean);
    const label = clean[0] === "docs" && clean[1] === "plans" ? clean.slice(2, 4).join("/") || "docs/plans" : clean.slice(0, 2).join("/") || "未分类";
    counts.set(label, (counts.get(label) ?? 0) + 1);
  };
  (snapshot.workflows ?? []).forEach((row) => add(row.planPath));
  (snapshot.sessions ?? []).forEach((row) => add(row.planPath));
  (snapshot.failures?.nodes ?? []).forEach((row) => { add(row.origin_plan); add(row.fixing_plan); });
  tickets.forEach((row) => add(row.planPath));
  const total = [...counts.values()].reduce((sum, value) => sum + value, 0);
  return [...counts.entries()].sort(([, left], [, right]) => right - left).slice(0, 10).map(([label, count]) => ({ label, count, ratio: total ? count / total : 0 }));
}

export function buildActiveTasks(jobs: CargoLaneProjection[], now: Date): AnalyticsActiveTask[] {
  return jobs.filter((job) => job.status === "running" || job.status === "leased").map((job) => ({ id: job.job_id, sessionId: job.session_id, lane: job.lane_kind, status: job.status, elapsedSeconds: elapsedSeconds(job.started_at ?? job.created_at, null, now) }));
}

export function buildPatchStatus(patches: PatchProjection[]): AnalyticsPatchStatus[] {
  const definitions: Array<[PatchProjection["status"], string, AnalyticsPatchStatus["tone"]]> = [
    ["queued", "排队", "warning"],
    ["applying", "应用中", "info"],
    ["applied", "已应用", "success"],
    ["needs_rebase", "需 Rebase", "warning"],
    ["failed", "失败", "danger"],
    ["cancelled", "已取消", "neutral"],
  ];
  return definitions.map(([status, label, tone]) => ({ label, value: patches.filter((patch) => patch.status === status).length, tone }));
}

function bucketIndex(value: number, start: number): number { return Math.min(23, Math.max(0, Math.floor((value - start) / HOUR_MS))); }
function timestamp(value: string | null | undefined): number { const parsed = value ? Date.parse(value) : Number.NaN; return Number.isFinite(parsed) ? parsed : 0; }
function elapsedSeconds(start: string, end: string | null, now: Date): number { const started = timestamp(start); const finished = end ? timestamp(end) : now.getTime(); return started ? Math.max(0, Math.floor((finished - started) / 1000)) : 0; }
function shortLabel(value: string): string { const parts = value.replaceAll("\\", "/").split("/").filter(Boolean); return parts.at(-1) || value; }
