export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };
export type JsonObject = { [key: string]: JsonValue };

export interface ServiceProjection {
  status: string;
  branch: string;
  mode: string;
  baseline: string;
  instanceId: string;
  startedAt: string;
  controlApiVersions: number[];
  supervision?: {
    state: string;
    busy: boolean;
    blockers: JsonObject[];
  };
}

export interface WorkflowSummary {
  runId: string;
  sessionId: string | null;
  workflowKey: string;
  planPath: string | null;
  topologyHash: string | null;
  state: string;
  statusReason: string | null;
  nodeCount: number;
  succeededCount: number;
  failedCount: number;
  updatedAt: string;
}

export interface WorkflowAttempt {
  attemptId: string;
  attemptNumber: number;
  state: string;
  accepted: boolean;
  evidence: JsonValue;
  startedAt: string | null;
  completedAt: string | null;
}

export interface WorkflowNode {
  nodeId: string;
  nodeKey: string;
  kind: string;
  title: string;
  stage: string;
  state: string;
  ownerSessionId: string | null;
  statusReason: string | null;
  currentAttempt: WorkflowAttempt | null;
  attemptHistory: WorkflowAttempt[];
  commitEligibility?: {
    eligible: boolean;
    code: string;
    missing: string[];
    rejected: string[];
    fingerprintConsistent: boolean;
    independentReviewAccepted: boolean;
  } | null;
}

export interface WorkflowDetail {
  runId: string;
  sessionId: string | null;
  workflowKey: string;
  planPath: string | null;
  topologyHash: string | null;
  state: string;
  statusReason: string | null;
  nodes: WorkflowNode[];
  edges: Array<{ fromNodeId: string; toNodeId: string; kind: string }>;
  artifacts: WorkflowArtifact[];
  topologyVersions: WorkflowTopologyVersion[];
  gates: WorkflowGateEvidence[];
  reviews: WorkflowReviewEvidence[];
  notifications: WorkflowNotificationAttempt[];
}

export interface WorkflowTopologyVersion { topologyVersionId: string; versionNumber: number; schemaVersion: number; sourceKind: string; contentHash: string; topologyHash: string; supersedesId: string | null; active: boolean; createdAt: string; }
export interface WorkflowGateEvidence { evidenceId: string; topologyVersionId: string; nodeId: string | null; attemptId: string | null; kind: string; decision: string; code: string; inputFingerprint: string; blockingNodeIds: string[]; applicableFailureIds: string[]; requiredEvidence: string[]; createdAt: string; }
export interface WorkflowReviewEvidence { reviewId: string; topologyVersionId: string; nodeId: string | null; attemptId: string | null; reviewer: string; executor: string; verdict: string; criticalCount: number; importantCount: number; summary: string; createdAt: string; }
export interface WorkflowNotificationAttempt { attemptId: string; commitSha: string; channel: string; status: string; attemptedAt: string; completedAt: string | null; exitCode: number | null; providerErrcode: string | null; sanitizedError: string | null; retryAllowed: boolean; }

export interface WorkflowArtifact {
  artifactId: string;
  nodeId: string | null;
  attemptId: string | null;
  kind: string;
  displayName: string;
  contentHash: string | null;
  byteCount: number | null;
  metadata: JsonObject;
  createdAt: string;
}

export interface SessionProjection {
  sessionId: string;
  displayName: string | null;
  planPath: string | null;
  status: string;
  statusReason: string | null;
  baseHead: string | null;
  baselineEpoch: number | null;
  writeScope: string[];
  updatedAt: string;
  lastHeartbeatAt: string;
}

export type CodexSessionState = "active" | "idle" | "archived" | "unavailable";
export type CodexSourceLocation = "active" | "archived" | "missing";
export type CodexLifecycleEvent = "session_meta" | "task_started" | "task_completed" | "turn_aborted" | "session_start" | "user_prompt_submit" | "stop" | "subagent_start" | "subagent_stop" | "unknown";

export interface CodexSessionProjection {
  threadId: string;
  sourceLocation: CodexSourceLocation;
  state: CodexSessionState;
  originator: string | null;
  cliVersion: string | null;
  threadSource: string | null;
  lastEvent: CodexLifecycleEvent;
  lastTurnId: string | null;
  boundSessionId: string | null;
  diagnosticCode: string | null;
  firstSeenAt: string;
  lastActivityAt: string;
  lastSyncedAt: string;
}

export interface CodexSyncRunProjection {
  runId: string;
  trigger: "startup" | "periodic" | "hook" | "controlled";
  status: "succeeded" | "partial" | "failed";
  scannedCount: number;
  changedCount: number;
  diagnosticCount: number;
  unavailableCount: number;
  durationMs: number;
  errorCode: string | null;
  createdAt: string;
  completedAt: string | null;
}

export interface CodexSessionsProjection {
  rows: CodexSessionProjection[];
  total: number;
  truncated: boolean;
  stateCounts: Record<CodexSessionState, number>;
  sourceCounts: Record<CodexSourceLocation, number>;
  queueDepth: number;
  lastSuccessfulAt: string | null;
  lastTerminalCode: string | null;
  lastRun: CodexSyncRunProjection | null;
}

export interface FailureProjection {
  nodes: FailureNode[];
  diagnostics: Array<{ diagnosticId: number; code: string; message: string; paths: string[]; createdAt: string }>;
}

export interface CollaborationProjection {
  baseline: BaselineEpoch | null;
  leases: LeaseProjection[];
  patches: PatchProjection[];
}

export interface ValidationProjection {
  cargoJobs: CargoJobProjection[];
  validationCopies: ValidationCopyProjection[];
}

export interface GitProjection { finalizeRequests: FinalizeRequestProjection[] }

export interface FailureNode extends JsonObject {
  node_id: number;
  lifecycle_key: string;
  artifact_path: string;
  kind: "failure" | "fixed";
  status: "open" | "fixed";
  created_at: string;
  resolved_at: string | null;
  summary_slug: string;
  origin_plan: string;
  fixing_plan: string;
  origin_child_dir: string;
  fixing_child_dir: string;
  priority: number;
  imported_at: string;
}

export interface BaselineEpoch extends JsonObject {
  epoch_id: number;
  head_commit: string;
  index_tree: string;
  health: "healthy" | "degraded";
  created_at: string;
  degraded_at: string | null;
  degraded_reason: string | null;
}

export interface LeaseProjection extends JsonObject {
  path_key: string;
  display_path: string;
  session_id: string;
  base_hash: string | null;
  acquired_at: string;
  last_heartbeat_at: string;
  expires_at: string;
}

export interface PatchProjection extends JsonObject {
  patch_id: number;
  session_id: string;
  patch_object_hash: string;
  targets: string[];
  status: "queued" | "applying" | "applied" | "needs_rebase" | "failed" | "cancelled";
  error_text: string | null;
  created_at: string;
  updated_at: string;
  applied_at: string | null;
}

export interface CargoJobProjection extends JsonObject {
  job_id: string;
  session_id: string;
  lane_kind: "check" | "test" | "workspace" | "gpu";
  target_dir: string;
  status: "leased" | "running" | "succeeded" | "failed" | "released" | "orphaned";
  dry_run: number;
  pid: number | null;
  command: string[];
  exit_code: number | null;
  created_at: string;
  last_heartbeat_at: string;
  started_at: string | null;
  finished_at: string | null;
  released_at: string | null;
  reuse_key: string | null;
  compatibility_key: string | null;
  reuse_profile: string | null;
  reused_from_job_id: string | null;
  cleanup_policy: "retained" | "delete_on_release";
  cleanup_status: "retained" | "pending" | "deleted" | "failed";
  cleanup_error: string | null;
}

export interface ValidationCopyProjection extends JsonObject {
  job_id: string;
  session_id: string;
  job_root: string;
  source_root: string;
  target_root: string;
  head_commit: string;
  status: "planned" | "materialized" | "running" | "cleanup_pending" | "removed" | "failed";
  created_at: string;
  removed_at: string | null;
}

export interface FinalizeRequestProjection extends JsonObject {
  request_id: string;
  session_id: string;
  message: string;
  paths: string[];
  categories: Record<string, string[]>;
  untracked: string[];
  validation: string[][];
  maintenance: number;
  status: "previewed" | "finalizing" | "committed" | "failed";
  commit_sha: string | null;
  error_text: string | null;
  created_at: string;
  completed_at: string | null;
}

export interface AuditEvent {
  eventId: number;
  sessionId: string | null;
  type: string;
  payload: JsonObject;
  createdAt: string;
}

export interface LogRange {
  events: AuditEvent[];
  truncated: boolean;
  nextBefore: number | null;
}

export interface ControlSnapshot {
  projectionVersion: number;
  eventCursor: number;
  service: ServiceProjection;
  workflows: WorkflowSummary[];
  sessions: SessionProjection[];
  codexSessions: CodexSessionsProjection;
  failures: FailureProjection;
  collaboration: CollaborationProjection;
  validation: ValidationProjection;
  git: GitProjection;
  audit: AuditEvent[];
}

export interface ControlEvent {
  id: number;
  type: string;
  payload: JsonObject;
  createdAt: string;
}

export interface ApiEnvelope<T> {
  ok: boolean;
  data?: T;
  error?: { code: string; message: string; retryable: boolean; details: JsonObject };
  meta: { apiVersion: number; correlationId: string };
}

export interface ControlAuthSession {
  actor: string;
  role: "observer" | "operator" | "committer" | "maintainer";
  boundSessionId: string | null;
  mutationEnabled?: boolean;
  elevatedUntil?: string | null;
}

export interface ActionSpecProjection {
  kind: string;
  title: string;
  risk: "green" | "yellow" | "red";
  requiredRole: ControlAuthSession["role"];
  enabled: boolean;
  sessionBound: boolean;
  previewOnly: boolean;
  warnings: string[];
}

export interface ActionCatalog { actions: ActionSpecProjection[] }

export interface ActionRecord {
  actionId: string;
  kind: string;
  risk: "green" | "yellow" | "red";
  requiredRole: ControlAuthSession["role"];
  actor: string;
  boundSessionId: string | null;
  parameters: JsonObject;
  impact: string[];
  warnings: string[];
  stateFingerprint: string;
  status: "previewed" | "executing" | "succeeded" | "failed" | "cancelled" | "expired" | "state_changed" | "denied";
  createdAt: string;
  expiresAt: string;
  reason: string | null;
  result: JsonObject | null;
  errorCode: string | null;
  confirmationPhrase?: string | null;
}

export interface ActionActivityResponse {
  actions: ActionRecord[];
  truncated: boolean;
  limit: number;
}
