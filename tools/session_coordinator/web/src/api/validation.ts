import type { ApiEnvelope, CodexSessionsProjection, ContinuationProjection, ControlEvent, ControlSnapshot, FailureHistoryProjection, FailureProjection, GitProjection, JsonObject, LogRange, ValidationHistoryProjection, ValidationProjection, WorkflowDetail } from "./contracts";

function object(value: unknown, label: string): JsonObject {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} 必须是对象`);
  return value as JsonObject;
}

function string(value: unknown, label: string): string {
  if (typeof value !== "string") throw new Error(`${label} 必须是字符串`);
  return value;
}

function number(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`${label} 必须是有限数字`);
  return value;
}

function array(value: unknown, label: string): unknown[] {
  if (!Array.isArray(value)) throw new Error(`${label} 必须是数组`);
  return value;
}

function nullableString(value: unknown, label: string): string | null {
  if (value === null) return null;
  return string(value, label);
}

function stringArray(value: unknown, label: string): string[] {
  return array(value, label).map((item, index) => string(item, `${label}[${index}]`));
}

function stringArrayMap(value: unknown, label: string): Record<string, string[]> {
  const parsed = object(value, label);
  for (const [key, paths] of Object.entries(parsed)) stringArray(paths, `${label}.${key}`);
  return parsed as unknown as Record<string, string[]>;
}

function nullableInteger(value: unknown, label: string): number | null {
  if (value === null) return null;
  return integer(value, label);
}

function flag(value: unknown, label: string): number {
  const parsed = integer(value, label);
  if (parsed !== 0 && parsed !== 1) throw new Error(`${label} 必须是 0 或 1`);
  return parsed;
}

function integer(value: unknown, label: string): number {
  const parsed = number(value, label);
  if (!Number.isSafeInteger(parsed)) throw new Error(`${label} 必须是安全整数`);
  return parsed;
}

function enumeration(value: unknown, allowed: readonly string[], label: string): string {
  const parsed = string(value, label);
  if (!allowed.includes(parsed)) throw new Error(`${label} 的枚举值无效`);
  return parsed;
}

function exactKeys(value: JsonObject, expected: readonly string[], label: string): void {
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  if (actual.length !== wanted.length || actual.some((key, index) => key !== wanted[index]))
    throw new Error(`${label} 字段集合无效`);
}

function boundedString(value: unknown, label: string, limit: number): string {
  const parsed = string(value, label);
  if (!parsed || parsed.length > limit) throw new Error(`${label} 长度无效`);
  return parsed;
}

function nonnegativeInteger(value: unknown, label: string): number {
  const parsed = integer(value, label);
  if (parsed < 0) throw new Error(`${label} 不得为负数`);
  return parsed;
}

const workflowStates = ["registered", "active", "waiting_dependency", "waiting_lease", "resolving_failure", "waiting_validation", "waiting_review", "finalizing", "succeeded", "failed", "cancelled", "stale", "archived"];
const sessionStates = ["registered", "active", "waiting_lease", "resolving_failure", "waiting_validation", "finalizing", "completed", "stale", "archived", "cancelled"];
const nodeStates = ["pending", "ready", "running", "waiting_external", "succeeded", "failed", "cancelled", "skipped"];
const nodeKinds = ["goal", "milestone", "slice", "validation", "review", "commit", "notification", "closeout"];
const stages = ["goal", "preflight", "implementation", "validation", "review", "commit", "notification"];
const validationTicketStatuses = ["queued", "materializing", "running", "passed", "failed", "snapshot_stale"];

export function parseEnvelope<T>(value: unknown, parseData: (input: unknown) => T): ApiEnvelope<T> {
  const root = object(value, "响应");
  const meta = object(root.meta, "响应元数据");
  if (number(meta.apiVersion, "API 版本") !== 1) throw new Error("不支持的控制 API 版本");
  string(meta.correlationId, "关联标识");
  if (root.ok !== true) {
    const error = object(root.error, "错误");
    throw new Error(string(error.message, "错误消息"));
  }
  return value as ApiEnvelope<T> & { data: T };
}

export function parseSnapshot(value: unknown): ControlSnapshot {
  const root = object(value, "快照");
  number(root.projectionVersion, "投影版本");
  number(root.eventCursor, "事件游标");
  const service = object(root.service, "服务状态");
  for (const key of ["status", "branch", "mode", "baseline", "instanceId", "startedAt"])
    string(service[key], `服务状态.${key}`);
  if (service.sessionTtlSeconds === undefined) service.sessionTtlSeconds = 600;
  const sessionTtlSeconds = nonnegativeInteger(service.sessionTtlSeconds, "服务状态.sessionTtlSeconds");
  if (sessionTtlSeconds < 60 || sessionTtlSeconds > 86_400)
    throw new Error("服务状态.sessionTtlSeconds 必须在 60 到 86400 秒之间");
  array(service.controlApiVersions, "控制 API 版本");
  if (service.supervision !== undefined) {
    const supervision = object(service.supervision, "服务监督状态");
    string(supervision.state, "服务监督状态.state");
    if (typeof supervision.busy !== "boolean") throw new Error("服务监督状态.busy 必须是布尔值");
    array(supervision.blockers, "服务监督状态.blockers").forEach((item, index) => object(item, `服务监督状态.blockers[${index}]`));
  }
  array(root.workflows, "workflows").forEach((item, index) => validateWorkflowSummary(item, `workflows[${index}]`));
  array(root.sessions, "sessions").forEach((item, index) => validateSession(item, `sessions[${index}]`));
  if (root.codexSessions === undefined) root.codexSessions = emptyCodexSessions();
  parseCodexSessions(root.codexSessions);
  if (root.experience === undefined) root.experience = emptyExperience();
  validateExperience(root.experience, "协作体验");
  array(root.audit, "audit").forEach((item, index) => validateAudit(item, `audit[${index}]`));
  parseFailureProjection(root.failures);
  const collaboration = object(root.collaboration, "协作投影");
  if (collaboration.baseline !== null) validateBaseline(collaboration.baseline, "协作基线");
  array(collaboration.leases, "协作.leases").forEach((item, index) => validateLease(item, `协作.leases[${index}]`));
  array(collaboration.patches, "协作.patches").forEach((item, index) => validatePatch(item, `协作.patches[${index}]`));
  parseValidationProjection(root.validation);
  parseGitProjection(root.git);
  return value as unknown as ControlSnapshot;
}

export function parseCodexSessions(value: unknown): CodexSessionsProjection {
  validateCodexSessions(value, "Codex Sessions");
  return value as CodexSessionsProjection;
}

export function parseFailureProjection(value: unknown): FailureProjection {
  const failures = object(value, "失败投影");
  array(failures.nodes, "失败节点").forEach((item, index) => validateFailureNode(item, `失败节点[${index}]`));
  array(failures.diagnostics, "失败诊断").forEach((item, index) => {
    const diagnostic = object(item, `失败诊断[${index}]`);
    integer(diagnostic.diagnosticId, "诊断标识"); string(diagnostic.code, "诊断代码"); string(diagnostic.message, "诊断消息");
    array(diagnostic.paths, "诊断路径").forEach((path) => string(path, "诊断路径")); string(diagnostic.createdAt, "诊断时间");
  });
  return failures as unknown as FailureProjection;
}

export function parseFailureHistory(value: unknown): FailureHistoryProjection {
  const projection = object(value, "Failure 历史");
  exactKeys(projection, ["chains", "statusCounts", "truncated"], "Failure 历史");
  const chains = array(projection.chains, "Failure 历史.chains");
  if (chains.length > 200) throw new Error("Failure 历史.chains 超过 200 行上限");
  chains.forEach((value, index) => {
    const chain = object(value, `Failure 历史.chains[${index}]`);
    exactKeys(chain, ["lifecycleKey", "summarySlug", "status", "priority", "originPlan", "fixingPlan", "artifactPath", "createdAt", "resolvedAt", "events"], `Failure 历史.chains[${index}]`);
    boundedString(chain.lifecycleKey, `Failure 历史.chains[${index}].lifecycleKey`, 1000);
    boundedString(chain.summarySlug, `Failure 历史.chains[${index}].summarySlug`, 500);
    enumeration(chain.status, ["open", "fixed"], `Failure 历史.chains[${index}].status`);
    nonnegativeInteger(chain.priority, `Failure 历史.chains[${index}].priority`);
    for (const key of ["originPlan", "fixingPlan", "artifactPath"])
      boundedString(chain[key], `Failure 历史.chains[${index}].${key}`, 1000);
    boundedString(chain.createdAt, `Failure 历史.chains[${index}].createdAt`, 64);
    if (chain.resolvedAt !== null) boundedString(chain.resolvedAt, `Failure 历史.chains[${index}].resolvedAt`, 64);
    const events = array(chain.events, `Failure 历史.chains[${index}].events`);
    if (events.length < 1 || events.length > 2) throw new Error(`Failure 历史.chains[${index}].events 数量无效`);
    events.forEach((eventValue, eventIndex) => {
      const event = object(eventValue, `Failure 历史.chains[${index}].events[${eventIndex}]`);
      exactKeys(event, ["kind", "createdAt", "artifactPath"], `Failure 历史.chains[${index}].events[${eventIndex}]`);
      enumeration(event.kind, ["added", "fixed"], `Failure 历史.chains[${index}].events[${eventIndex}].kind`);
      boundedString(event.createdAt, `Failure 历史.chains[${index}].events[${eventIndex}].createdAt`, 64);
      boundedString(event.artifactPath, `Failure 历史.chains[${index}].events[${eventIndex}].artifactPath`, 1000);
    });
  });
  const counts = object(projection.statusCounts, "Failure 历史.statusCounts");
  exactKeys(counts, ["open", "fixed"], "Failure 历史.statusCounts");
  nonnegativeInteger(counts.open, "Failure 历史.statusCounts.open");
  nonnegativeInteger(counts.fixed, "Failure 历史.statusCounts.fixed");
  if (typeof projection.truncated !== "boolean") throw new Error("Failure 历史.truncated 必须是布尔值");
  return projection as unknown as FailureHistoryProjection;
}

export function parseValidationHistory(value: unknown): ValidationHistoryProjection {
  const projection = object(value, "验证历史");
  exactKeys(projection, ["tickets", "statusCounts", "truncated"], "验证历史");
  const tickets = array(projection.tickets, "验证历史.tickets");
  if (tickets.length > 200) throw new Error("验证历史.tickets 超过 200 行上限");
  tickets.forEach((value, index) => {
    const ticket = object(value, `验证历史.tickets[${index}]`);
    exactKeys(ticket, ["ticketId", "sessionId", "planPath", "status", "sourceManifestHash", "command", "commandTruncated", "createdAt", "updatedAt", "events", "eventsTruncated"], `验证历史.tickets[${index}]`);
    boundedString(ticket.ticketId, `验证历史.tickets[${index}].ticketId`, 160);
    boundedString(ticket.sessionId, `验证历史.tickets[${index}].sessionId`, 160);
    boundedString(ticket.planPath, `验证历史.tickets[${index}].planPath`, 1000);
    enumeration(ticket.status, validationTicketStatuses, `验证历史.tickets[${index}].status`);
    const manifestHash = boundedString(ticket.sourceManifestHash, `验证历史.tickets[${index}].sourceManifestHash`, 64);
    if (!/^[0-9a-f]{64}$/.test(manifestHash)) throw new Error(`验证历史.tickets[${index}].sourceManifestHash 必须是 SHA-256`);
    const command = array(ticket.command, `验证历史.tickets[${index}].command`);
    if (command.length > 24) throw new Error(`验证历史.tickets[${index}].command 超过 24 项上限`);
    command.forEach((argument, argumentIndex) => boundedString(argument, `验证历史.tickets[${index}].command[${argumentIndex}]`, 160));
    if (typeof ticket.commandTruncated !== "boolean" || typeof ticket.eventsTruncated !== "boolean")
      throw new Error(`验证历史.tickets[${index}] 截断标记必须是布尔值`);
    boundedString(ticket.createdAt, `验证历史.tickets[${index}].createdAt`, 64);
    boundedString(ticket.updatedAt, `验证历史.tickets[${index}].updatedAt`, 64);
    const events = array(ticket.events, `验证历史.tickets[${index}].events`);
    if (events.length > 64) throw new Error(`验证历史.tickets[${index}].events 超过 64 项上限`);
    events.forEach((eventValue, eventIndex) => {
      const event = object(eventValue, `验证历史.tickets[${index}].events[${eventIndex}]`);
      exactKeys(event, ["eventId", "type", "createdAt", "fromStatus", "toStatus", "phase", "errorCode", "jobId", "runId", "exitCode"], `验证历史.tickets[${index}].events[${eventIndex}]`);
      nonnegativeInteger(event.eventId, `验证历史.tickets[${index}].events[${eventIndex}].eventId`);
      boundedString(event.type, `验证历史.tickets[${index}].events[${eventIndex}].type`, 160);
      boundedString(event.createdAt, `验证历史.tickets[${index}].events[${eventIndex}].createdAt`, 64);
      for (const key of ["fromStatus", "toStatus"])
        if (event[key] !== null) enumeration(event[key], validationTicketStatuses, `验证历史.tickets[${index}].events[${eventIndex}].${key}`);
      for (const key of ["phase", "errorCode", "jobId", "runId"])
        if (event[key] !== null) boundedString(event[key], `验证历史.tickets[${index}].events[${eventIndex}].${key}`, 160);
      nullableInteger(event.exitCode, `验证历史.tickets[${index}].events[${eventIndex}].exitCode`);
    });
  });
  const counts = object(projection.statusCounts, "验证历史.statusCounts");
  exactKeys(counts, validationTicketStatuses, "验证历史.statusCounts");
  for (const status of validationTicketStatuses) nonnegativeInteger(counts[status], `验证历史.statusCounts.${status}`);
  if (typeof projection.truncated !== "boolean") throw new Error("验证历史.truncated 必须是布尔值");
  return projection as unknown as ValidationHistoryProjection;
}

export function parseValidationProjection(value: unknown): ValidationProjection {
  const validation = object(value, "验证投影");
  if (validation.cargoReservations === undefined) validation.cargoReservations = [];
  if (validation.cpuBurst === undefined) validation.cpuBurst = emptyCpuBurst();
  array(validation.cargoJobs, "验证.cargoJobs").forEach((item, index) => {
    applyLegacyCargoLaneDefaults(item, `验证.cargoJobs[${index}]`);
    validateCargoLane(item, `验证.cargoJobs[${index}]`);
  });
  array(validation.currentCargoTargets, "验证.currentCargoTargets").forEach((item, index) => {
    applyLegacyCargoLaneDefaults(item, `验证.currentCargoTargets[${index}]`);
    validateCargoLane(item, `验证.currentCargoTargets[${index}]`);
  });
  const cargoReservations = array(validation.cargoReservations, "验证.cargoReservations");
  if (cargoReservations.length > 20) throw new Error("验证.cargoReservations 超过 20 行上限");
  cargoReservations.forEach((item, index) => {
    applyLegacyCargoReservationDefaults(item, `验证.cargoReservations[${index}]`);
    validateCargoReservation(item, `验证.cargoReservations[${index}]`);
  });
  validateCpuBurst(validation.cpuBurst, "验证.cpuBurst");
  validateArtifactLifecycle(validation.artifactLifecycle, "验证.artifactLifecycle");
  array(validation.validationCopies, "验证.validationCopies").forEach((item, index) => validateValidationCopy(item, `验证.validationCopies[${index}]`));
  return validation as unknown as ValidationProjection;
}

export function parseContinuationProjection(value: unknown): ContinuationProjection {
  const projection = object(value, "续作投影");
  exactKeys(projection, ["continuations"], "续作投影");
  validateExperience({ sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 }, blockers: [], continuations: projection.continuations }, "续作投影");
  return projection as unknown as ContinuationProjection;
}

export function parseGitProjection(value: unknown): GitProjection {
  const git = object(value, "Git 投影");
  array(git.finalizeRequests, "Git.finalizeRequests").forEach((item, index) => validateFinalizeRequest(item, `Git.finalizeRequests[${index}]`));
  return git as unknown as GitProjection;
}

function emptyCodexSessions() {
  return {
    rows: [], total: 0, truncated: false,
    stateCounts: { active: 0, idle: 0, archived: 0, unavailable: 0 },
    sourceCounts: { active: 0, archived: 0, missing: 0 },
    queueDepth: 0, lastSuccessfulAt: null, lastTerminalCode: null, lastRun: null,
  };
}

function emptyExperience() {
  return {
    sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 },
    blockers: [],
    continuations: [],
  };
}

function emptyCpuBurst() {
  return { capacity: 1, active: 0, eligiblePending: 0 };
}

function validateExperience(value: unknown, label: string): void {
  const experience = object(value, label);
  if (experience.continuations === undefined) experience.continuations = [];
  exactKeys(
    experience,
    experience.intervention === undefined
      ? ["sync", "blockers", "continuations"]
      : ["sync", "blockers", "continuations", "intervention"],
    label,
  );
  const sync = object(experience.sync, `${label}.sync`);
  exactKeys(sync, ["runs", "quietRuns", "visibleChanges", "averageDurationMs"], `${label}.sync`);
  for (const key of ["runs", "quietRuns", "visibleChanges", "averageDurationMs"])
    nonnegativeInteger(sync[key], `${label}.sync.${key}`);
  const blockers = array(experience.blockers, `${label}.blockers`);
  if (blockers.length > 20) throw new Error(`${label}.blockers 超过 20 行上限`);
  blockers.forEach((value, index) => {
    const blocker = object(value, `${label}.blockers[${index}]`);
    exactKeys(blocker, ["kind", "ownerSessionId", "laneKind", "status", "createdAt"], `${label}.blockers[${index}]`);
    enumeration(blocker.kind, ["cargo"], `${label}.blockers[${index}].kind`);
    boundedString(blocker.ownerSessionId, `${label}.blockers[${index}].ownerSessionId`, 160);
    enumeration(blocker.laneKind, ["check", "test", "workspace", "gpu"], `${label}.blockers[${index}].laneKind`);
    enumeration(blocker.status, ["leased", "running"], `${label}.blockers[${index}].status`);
    boundedString(blocker.createdAt, `${label}.blockers[${index}].createdAt`, 64);
  });
  const continuations = array(experience.continuations, `${label}.continuations`);
  if (continuations.length > 20) throw new Error(`${label}.continuations 超过 20 行上限`);
  continuations.forEach((value, index) => {
    const continuation = object(value, `${label}.continuations[${index}]`);
    exactKeys(continuation, ["sessionId", "planPath", "waitKind", "candidate", "scopeClaimRequired", "returnToPrimary"], `${label}.continuations[${index}]`);
    boundedString(continuation.sessionId, `${label}.continuations[${index}].sessionId`, 160);
    boundedString(continuation.planPath, `${label}.continuations[${index}].planPath`, 500);
    enumeration(continuation.waitKind, ["validation", "lease", "external"], `${label}.continuations[${index}].waitKind`);
    const candidate = object(continuation.candidate, `${label}.continuations[${index}].candidate`);
    const candidateHasRouting = candidate.kind !== undefined || candidate.planPath !== undefined;
    exactKeys(
      candidate,
      candidateHasRouting ? ["kind", "planPath", "milestone", "title"] : ["milestone", "title"],
      `${label}.continuations[${index}].candidate`,
    );
    if (candidateHasRouting) {
      enumeration(candidate.kind, ["same_plan", "unowned_failure"], `${label}.continuations[${index}].candidate.kind`);
      boundedString(candidate.planPath, `${label}.continuations[${index}].candidate.planPath`, 500);
    }
    boundedString(candidate.milestone, `${label}.continuations[${index}].candidate.milestone`, 32);
    boundedString(candidate.title, `${label}.continuations[${index}].candidate.title`, 500);
    if (typeof continuation.scopeClaimRequired !== "boolean") throw new Error(`${label}.continuations[${index}].scopeClaimRequired 必须是布尔值`);
    if (typeof continuation.returnToPrimary !== "boolean") throw new Error(`${label}.continuations[${index}].returnToPrimary 必须是布尔值`);
  });
  if (experience.intervention !== undefined) validateIntervention(experience.intervention, `${label}.intervention`);
}

function validateIntervention(value: unknown, label: string): void {
  const intervention = object(value, label);
  exactKeys(intervention, ["openFailureCount", "responsiblePlanCount", "mode", "maxConcurrentPlans", "suggestedNext", "validation"], label);
  nonnegativeInteger(intervention.openFailureCount, `${label}.openFailureCount`);
  nonnegativeInteger(intervention.responsiblePlanCount, `${label}.responsiblePlanCount`);
  enumeration(intervention.mode, ["single_plan"], `${label}.mode`);
  if (intervention.maxConcurrentPlans !== 1) throw new Error(`${label}.maxConcurrentPlans 必须为 1`);
  if (intervention.suggestedNext !== null) {
    const next = object(intervention.suggestedNext, `${label}.suggestedNext`);
    exactKeys(next, ["kind", "planPath", "summary", "priority", "action"], `${label}.suggestedNext`);
    enumeration(next.kind, ["failure"], `${label}.suggestedNext.kind`);
    boundedString(next.planPath, `${label}.suggestedNext.planPath`, 500);
    boundedString(next.summary, `${label}.suggestedNext.summary`, 500);
    nonnegativeInteger(next.priority, `${label}.suggestedNext.priority`);
    enumeration(next.action, ["resolve_one_failure"], `${label}.suggestedNext.action`);
  }
  const validation = object(intervention.validation, `${label}.validation`);
  exactKeys(validation, ["waitingSessionCount", "pendingReservationCount", "nextReservation"], `${label}.validation`);
  nonnegativeInteger(validation.waitingSessionCount, `${label}.validation.waitingSessionCount`);
  nonnegativeInteger(validation.pendingReservationCount, `${label}.validation.pendingReservationCount`);
  if (validation.nextReservation !== null) {
    const next = object(validation.nextReservation, `${label}.validation.nextReservation`);
    exactKeys(next, ["sessionId", "queuePosition", "executionMode"], `${label}.validation.nextReservation`);
    boundedString(next.sessionId, `${label}.validation.nextReservation.sessionId`, 160);
    const queuePosition = next.queuePosition;
    nonnegativeInteger(queuePosition, `${label}.validation.nextReservation.queuePosition`);
    if (typeof queuePosition !== "number" || queuePosition < 1)
      throw new Error(`${label}.validation.nextReservation.queuePosition 必须大于零`);
    enumeration(next.executionMode, ["warm", "burst"], `${label}.validation.nextReservation.executionMode`);
  }
}

function validateCodexSessions(value: unknown, label: string): void {
  const projection = object(value, label);
  exactKeys(projection, ["rows", "total", "truncated", "stateCounts", "sourceCounts", "queueDepth", "lastSuccessfulAt", "lastTerminalCode", "lastRun"], label);
  const rows = array(projection.rows, `${label}.rows`);
  if (rows.length > 1000) throw new Error(`${label}.rows 超过 1000 行上限`);
  rows.forEach((row, index) => validateCodexSession(row, `${label}.rows[${index}]`));
  nonnegativeInteger(projection.total, `${label}.total`);
  if (typeof projection.truncated !== "boolean") throw new Error(`${label}.truncated 必须是布尔值`);
  const stateCounts = object(projection.stateCounts, `${label}.stateCounts`);
  exactKeys(stateCounts, ["active", "idle", "archived", "unavailable"], `${label}.stateCounts`);
  for (const state of ["active", "idle", "archived", "unavailable"]) nonnegativeInteger(stateCounts[state], `${label}.stateCounts.${state}`);
  const sourceCounts = object(projection.sourceCounts, `${label}.sourceCounts`);
  exactKeys(sourceCounts, ["active", "archived", "missing"], `${label}.sourceCounts`);
  for (const source of ["active", "archived", "missing"]) nonnegativeInteger(sourceCounts[source], `${label}.sourceCounts.${source}`);
  nonnegativeInteger(projection.queueDepth, `${label}.queueDepth`);
  if (projection.lastSuccessfulAt !== null) boundedString(projection.lastSuccessfulAt, `${label}.lastSuccessfulAt`, 64);
  if (projection.lastTerminalCode !== null) boundedString(projection.lastTerminalCode, `${label}.lastTerminalCode`, 160);
  if (projection.lastRun !== null) validateCodexRun(projection.lastRun, `${label}.lastRun`);
}

function validateCodexSession(value: unknown, label: string): void {
  const row = object(value, label);
  exactKeys(row, ["threadId", "sourceLocation", "state", "originator", "cliVersion", "threadSource", "lastEvent", "lastTurnId", "boundSessionId", "diagnosticCode", "firstSeenAt", "lastActivityAt", "lastSyncedAt"], label);
  boundedString(row.threadId, `${label}.threadId`, 160);
  enumeration(row.sourceLocation, ["active", "archived", "missing"], `${label}.sourceLocation`);
  enumeration(row.state, ["active", "idle", "archived", "unavailable"], `${label}.state`);
  for (const key of ["originator", "cliVersion", "threadSource"])
    if (row[key] !== null) boundedString(row[key], `${label}.${key}`, 256);
  enumeration(row.lastEvent, ["session_meta", "task_started", "task_completed", "turn_aborted", "session_start", "user_prompt_submit", "stop", "subagent_start", "subagent_stop", "unknown"], `${label}.lastEvent`);
  for (const key of ["lastTurnId", "boundSessionId"])
    if (row[key] !== null) boundedString(row[key], `${label}.${key}`, 160);
  if (row.diagnosticCode !== null) boundedString(row.diagnosticCode, `${label}.diagnosticCode`, 160);
  for (const key of ["firstSeenAt", "lastActivityAt", "lastSyncedAt"])
    boundedString(row[key], `${label}.${key}`, 64);
}

function validateCodexRun(value: unknown, label: string): void {
  const run = object(value, label);
  exactKeys(run, ["runId", "trigger", "status", "scannedCount", "changedCount", "diagnosticCount", "unavailableCount", "durationMs", "errorCode", "createdAt", "completedAt"], label);
  boundedString(run.runId, `${label}.runId`, 160);
  enumeration(run.trigger, ["startup", "periodic", "hook", "controlled"], `${label}.trigger`);
  enumeration(run.status, ["succeeded", "partial", "failed"], `${label}.status`);
  for (const key of ["scannedCount", "changedCount", "diagnosticCount", "unavailableCount", "durationMs"])
    nonnegativeInteger(run[key], `${label}.${key}`);
  if (run.errorCode !== null) boundedString(run.errorCode, `${label}.errorCode`, 160);
  boundedString(run.createdAt, `${label}.createdAt`, 64);
  if (run.completedAt !== null) boundedString(run.completedAt, `${label}.completedAt`, 64);
}

export function parseWorkflowDetail(value: unknown): WorkflowDetail {
  const root = object(value, "工作流详情");
  for (const key of ["runId", "workflowKey", "state"])
    string(root[key], `工作流.${key}`);
  nullableString(root.sessionId, "工作流.sessionId");
  nullableString(root.topologyHash, "工作流.topologyHash");
  enumeration(root.state, workflowStates, "工作流.state");
  nullableString(root.planPath, "工作流.planPath");
  nullableString(root.statusReason, "工作流.statusReason");
  array(root.nodes, "工作流节点").forEach((item, index) => validateNode(item, `nodes[${index}]`));
  array(root.edges, "工作流边").forEach((item, index) => {
    const edge = object(item, `edges[${index}]`);
    string(edge.fromNodeId, "边.fromNodeId"); string(edge.toNodeId, "边.toNodeId"); string(edge.kind, "边.kind");
  });
  array(root.artifacts, "工作流工件").forEach((item, index) => {
    const artifact = object(item, `artifacts[${index}]`);
    string(artifact.artifactId, "工件.artifactId"); nullableString(artifact.nodeId, "工件.nodeId"); nullableString(artifact.attemptId, "工件.attemptId");
    string(artifact.kind, "工件.kind"); string(artifact.displayName, "工件.displayName"); nullableString(artifact.contentHash, "工件.contentHash");
    if (artifact.byteCount !== null) integer(artifact.byteCount, "工件.byteCount"); object(artifact.metadata, "工件.metadata"); string(artifact.createdAt, "工件.createdAt");
  });
  array(root.topologyVersions, "拓扑版本").forEach((item, index) => {
    const version = object(item, `topologyVersions[${index}]`);
    string(version.topologyVersionId, "拓扑版本.id"); integer(version.versionNumber, "拓扑版本.number"); integer(version.schemaVersion, "拓扑版本.schema");
    string(version.sourceKind, "拓扑版本.source"); string(version.contentHash, "拓扑版本.contentHash"); string(version.topologyHash, "拓扑版本.topologyHash"); nullableString(version.supersedesId, "拓扑版本.supersedesId"); string(version.createdAt, "拓扑版本.createdAt");
    if (typeof version.active !== "boolean") throw new Error("拓扑版本.active 必须是布尔值");
  });
  array(root.gates, "门禁证据").forEach((item, index) => {
    const gate = object(item, `gates[${index}]`); for (const key of ["evidenceId", "topologyVersionId", "kind", "decision", "code", "inputFingerprint", "createdAt"]) string(gate[key], `门禁.${key}`);
    nullableString(gate.nodeId, "门禁.nodeId"); nullableString(gate.attemptId, "门禁.attemptId"); stringArray(gate.blockingNodeIds, "门禁.blockingNodeIds"); stringArray(gate.applicableFailureIds, "门禁.applicableFailureIds"); stringArray(gate.requiredEvidence, "门禁.requiredEvidence");
  });
  array(root.reviews, "评审证据").forEach((item, index) => {
    const review = object(item, `reviews[${index}]`); for (const key of ["reviewId", "topologyVersionId", "reviewer", "executor", "verdict", "summary", "createdAt"]) string(review[key], `评审.${key}`);
    nullableString(review.nodeId, "评审.nodeId"); nullableString(review.attemptId, "评审.attemptId"); integer(review.criticalCount, "评审.criticalCount"); integer(review.importantCount, "评审.importantCount");
  });
  array(root.notifications, "通知尝试").forEach((item, index) => {
    const notification = object(item, `notifications[${index}]`); for (const key of ["attemptId", "commitSha", "channel", "status", "attemptedAt"]) string(notification[key], `通知.${key}`);
    nullableString(notification.completedAt, "通知.completedAt"); nullableString(notification.providerErrcode, "通知.providerErrcode"); nullableString(notification.sanitizedError, "通知.sanitizedError"); if (notification.exitCode !== null) integer(notification.exitCode, "通知.exitCode"); if (typeof notification.retryAllowed !== "boolean") throw new Error("通知.retryAllowed 必须是布尔值");
  });
  return value as unknown as WorkflowDetail;
}

export function parseLogRange(value: unknown): LogRange {
  const root = object(value, "日志范围");
  array(root.events, "日志事件").forEach((item, index) => validateAudit(item, `日志事件[${index}]`));
  if (typeof root.truncated !== "boolean") throw new Error("日志截断标识必须是布尔值");
  if (root.nextBefore !== null) integer(root.nextBefore, "下一日志游标");
  return value as unknown as LogRange;
}

export function parseControlEvent(idText: string, dataText: string): ControlEvent {
  const id = Number(idText);
  if (!Number.isSafeInteger(id) || id < 1) throw new Error("事件标识无效");
  const root = object(JSON.parse(dataText), "事件");
  return {
    id,
    type: string(root.type, "事件类型"),
    payload: object(root.payload, "事件载荷"),
    createdAt: string(root.createdAt, "事件时间"),
  };
}

function validateWorkflowSummary(value: unknown, label: string): void {
  const item = object(value, label);
  for (const key of ["runId", "workflowKey", "updatedAt"]) string(item[key], `${label}.${key}`);
  nullableString(item.sessionId, `${label}.sessionId`);
  nullableString(item.topologyHash, `${label}.topologyHash`);
  nullableString(item.planPath, `${label}.planPath`); nullableString(item.statusReason, `${label}.statusReason`);
  enumeration(item.state, workflowStates, `${label}.state`);
  for (const key of ["nodeCount", "succeededCount", "failedCount"]) integer(item[key], `${label}.${key}`);
}

function validateSession(value: unknown, label: string): void {
  const item = object(value, label);
  string(item.sessionId, `${label}.sessionId`); nullableString(item.displayName, `${label}.displayName`); nullableString(item.planPath, `${label}.planPath`);
  enumeration(item.status, sessionStates, `${label}.status`); nullableString(item.statusReason, `${label}.statusReason`); nullableString(item.baseHead, `${label}.baseHead`);
  if (item.baselineEpoch !== null) integer(item.baselineEpoch, `${label}.baselineEpoch`);
  array(item.writeScope, `${label}.writeScope`).forEach((path) => string(path, `${label}.writeScope`));
  string(item.updatedAt, `${label}.updatedAt`); string(item.lastHeartbeatAt, `${label}.lastHeartbeatAt`);
}

function validateAudit(value: unknown, label: string): void {
  const item = object(value, label);
  integer(item.eventId, `${label}.eventId`); nullableString(item.sessionId, `${label}.sessionId`); string(item.type, `${label}.type`); object(item.payload, `${label}.payload`); string(item.createdAt, `${label}.createdAt`);
}

function validateNode(value: unknown, label: string): void {
  const node = object(value, label);
  for (const key of ["nodeId", "nodeKey", "title"]) string(node[key], `${label}.${key}`);
  enumeration(node.kind, nodeKinds, `${label}.kind`); enumeration(node.stage, stages, `${label}.stage`); enumeration(node.state, nodeStates, `${label}.state`);
  nullableString(node.ownerSessionId, `${label}.ownerSessionId`); nullableString(node.statusReason, `${label}.statusReason`);
  if (node.currentAttempt !== null) validateAttempt(node.currentAttempt, `${label}.currentAttempt`);
  array(node.attemptHistory, `${label}.attemptHistory`).forEach((attempt, index) => validateAttempt(attempt, `${label}.attemptHistory[${index}]`));
  if (node.commitEligibility !== undefined && node.commitEligibility !== null) {
    const eligibility = object(node.commitEligibility, `${label}.commitEligibility`);
    if (typeof eligibility.eligible !== "boolean") throw new Error(`${label}.commitEligibility.eligible 必须是布尔值`);
    string(eligibility.code, `${label}.commitEligibility.code`);
    stringArray(eligibility.missing, `${label}.commitEligibility.missing`);
    stringArray(eligibility.rejected, `${label}.commitEligibility.rejected`);
    if (typeof eligibility.fingerprintConsistent !== "boolean") throw new Error(`${label}.commitEligibility.fingerprintConsistent 必须是布尔值`);
    if (typeof eligibility.independentReviewAccepted !== "boolean") throw new Error(`${label}.commitEligibility.independentReviewAccepted 必须是布尔值`);
  }
}

function validateAttempt(value: unknown, label: string): void {
  const attempt = object(value, label);
  string(attempt.attemptId, `${label}.attemptId`); integer(attempt.attemptNumber, `${label}.attemptNumber`); enumeration(attempt.state, nodeStates, `${label}.state`);
  if (typeof attempt.accepted !== "boolean") throw new Error(`${label}.accepted 必须是布尔值`);
  if (attempt.evidence === undefined) throw new Error(`${label}.evidence 缺失`);
  nullableString(attempt.startedAt, `${label}.startedAt`); nullableString(attempt.completedAt, `${label}.completedAt`);
}

function validateFailureNode(value: unknown, label: string): void {
  const node = object(value, label);
  integer(node.node_id, `${label}.node_id`);
  for (const key of ["lifecycle_key", "artifact_path", "created_at", "summary_slug", "origin_plan", "fixing_plan", "origin_child_dir", "fixing_child_dir", "imported_at"])
    string(node[key], `${label}.${key}`);
  enumeration(node.kind, ["failure", "fixed"], `${label}.kind`);
  enumeration(node.status, ["open", "fixed"], `${label}.status`);
  nullableString(node.resolved_at, `${label}.resolved_at`);
  integer(node.priority, `${label}.priority`);
}

function validateBaseline(value: unknown, label: string): void {
  const baseline = object(value, label);
  integer(baseline.epoch_id, `${label}.epoch_id`);
  for (const key of ["head_commit", "index_tree", "created_at"])
    string(baseline[key], `${label}.${key}`);
  if (baseline.manifest_bytes !== undefined) integer(baseline.manifest_bytes, `${label}.manifest_bytes`);
  else string(baseline.manifest_json, `${label}.manifest_json`);
  enumeration(baseline.health, ["healthy", "degraded"], `${label}.health`);
  nullableString(baseline.degraded_at, `${label}.degraded_at`);
  nullableString(baseline.degraded_reason, `${label}.degraded_reason`);
}

function validateLease(value: unknown, label: string): void {
  const lease = object(value, label);
  for (const key of ["path_key", "display_path", "session_id", "acquired_at", "last_heartbeat_at", "expires_at"])
    string(lease[key], `${label}.${key}`);
  nullableString(lease.base_hash, `${label}.base_hash`);
}

function validatePatch(value: unknown, label: string): void {
  const patch = object(value, label);
  integer(patch.patch_id, `${label}.patch_id`);
  for (const key of ["session_id", "patch_object_hash", "created_at", "updated_at"])
    string(patch[key], `${label}.${key}`);
  stringArray(patch.targets, `${label}.targets`);
  if (patch.content_bytes !== undefined) {
    integer(patch.content_bytes, `${label}.content_bytes`);
    flag(patch.has_current_objects, `${label}.has_current_objects`);
  } else {
    object(patch.base_hashes, `${label}.base_hashes`);
    object(patch.base_objects, `${label}.base_objects`);
    if (patch.current_objects !== null) object(patch.current_objects, `${label}.current_objects`);
  }
  enumeration(patch.status, ["queued", "applying", "applied", "needs_rebase", "failed", "cancelled"], `${label}.status`);
  nullableString(patch.error_text, `${label}.error_text`);
  nullableString(patch.applied_at, `${label}.applied_at`);
}

function validateCargoLane(value: unknown, label: string): void {
  const job = object(value, label);
  for (const key of ["job_id", "session_id", "created_at"])
    string(job[key], `${label}.${key}`);
  enumeration(job.lane_kind, ["check", "test", "workspace", "gpu"], `${label}.lane_kind`);
  enumeration(job.status, ["leased", "running", "succeeded", "failed", "released", "orphaned"], `${label}.status`);
  for (const key of ["started_at", "finished_at", "released_at"])
    nullableString(job[key], `${label}.${key}`);
  enumeration(job.cleanup_policy, ["retained", "delete_on_release"], `${label}.cleanup_policy`);
  enumeration(job.cleanup_status, ["retained", "pending", "deleted", "failed"], `${label}.cleanup_status`);
  enumeration(job.process_observation, ["not_applicable", "awaiting_observation", "observed", "reconciling"], `${label}.process_observation`);
}

function applyLegacyCargoLaneDefaults(value: unknown, label: string): void {
  const job = object(value, label);
  if (job.process_observation === undefined)
    job.process_observation = job.status === "running" ? "awaiting_observation" : "not_applicable";
}

function validateCargoReservation(value: unknown, label: string): void {
  const reservation = object(value, label);
  exactKeys(
    reservation,
    ["reservationId", "sessionId", "laneScope", "executionMode", "burstEligible", "status", "queuePosition", "createdAt", "expiresAt"],
    label,
  );
  for (const key of ["reservationId", "sessionId", "createdAt", "expiresAt"])
    boundedString(reservation[key], `${label}.${key}`, 160);
  enumeration(reservation.laneScope, ["cpu", "gpu"], `${label}.laneScope`);
  enumeration(reservation.executionMode, ["warm", "burst"], `${label}.executionMode`);
  if (typeof reservation.burstEligible !== "boolean") throw new Error(`${label}.burstEligible 必须是布尔值`);
  enumeration(reservation.status, ["pending", "leased", "running"], `${label}.status`);
  const position = nonnegativeInteger(reservation.queuePosition, `${label}.queuePosition`);
  if (position < 1) throw new Error(`${label}.queuePosition 必须从 1 开始`);
}

function applyLegacyCargoReservationDefaults(value: unknown, label: string): void {
  const reservation = object(value, label);
  if (reservation.executionMode === undefined) reservation.executionMode = "warm";
  if (reservation.burstEligible === undefined) reservation.burstEligible = false;
}

function validateCpuBurst(value: unknown, label: string): void {
  const burst = object(value, label);
  exactKeys(burst, ["capacity", "active", "eligiblePending"], label);
  if (burst.capacity !== 1) throw new Error(`${label}.capacity 必须为 1`);
  if (burst.active !== 0 && burst.active !== 1) throw new Error(`${label}.active 必须为 0 或 1`);
  nonnegativeInteger(burst.eligiblePending, `${label}.eligiblePending`);
}

function validateValidationCopy(value: unknown, label: string): void {
  const copy = object(value, label);
  for (const key of ["job_id", "session_id", "job_root", "source_root", "target_root", "head_commit", "created_at"])
    string(copy[key], `${label}.${key}`);
  if (copy.manifest_bytes !== undefined) integer(copy.manifest_bytes, `${label}.manifest_bytes`);
  else stringArray(copy.manifest, `${label}.manifest`);
  enumeration(copy.status, ["planned", "materialized", "running", "cleanup_pending", "removed", "failed"], `${label}.status`);
  nullableString(copy.removed_at, `${label}.removed_at`);
}

function validateArtifactLifecycle(value: unknown, label: string): void {
  const lifecycle = object(value, label);
  exactKeys(lifecycle, ["reusablePools", "ephemeralTargets", "pendingCleanup", "failedCleanup"], label);
  for (const key of ["reusablePools", "ephemeralTargets", "pendingCleanup", "failedCleanup"])
    nonnegativeInteger(lifecycle[key], `${label}.${key}`);
}

function validateFinalizeRequest(value: unknown, label: string): void {
  const request = object(value, label);
  for (const key of ["request_id", "session_id", "message", "created_at"])
    string(request[key], `${label}.${key}`);
  for (const key of ["paths", "untracked"])
    stringArray(request[key], `${label}.${key}`);
  stringArrayMap(request.categories, `${label}.categories`);
  array(request.validation, `${label}.validation`).forEach((command, index) => stringArray(command, `${label}.validation[${index}]`));
  flag(request.maintenance, `${label}.maintenance`);
  enumeration(request.status, ["previewed", "finalizing", "committed", "failed"], `${label}.status`);
  nullableString(request.commit_sha, `${label}.commit_sha`);
  nullableString(request.error_text, `${label}.error_text`);
  nullableString(request.completed_at, `${label}.completed_at`);
}
