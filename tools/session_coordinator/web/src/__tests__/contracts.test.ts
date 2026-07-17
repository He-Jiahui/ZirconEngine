import test from "node:test";
import assert from "node:assert/strict";
import { parseControlEvent, parseSnapshot, parseWorkflowDetail } from "../api/validation";

test("runtime contracts reject incomplete snapshots", () => assert.throws(() => parseSnapshot({ eventCursor: 1 })));
test("runtime contracts parse text-only events", () => assert.equal(parseControlEvent("7", '{"type":"session.updated","payload":{},"createdAt":"now"}').id, 7));
test("runtime contracts accept the default Session goal before topology compilation", () => {
  const detail = parseWorkflowDetail({ runId: "r", sessionId: "s", workflowKey: "session:s", planPath: null, topologyHash: null, state: "registered", statusReason: null, nodes: [{ nodeId: "r:goal", nodeKey: "goal", kind: "goal", title: "Session Goal", stage: "goal", state: "pending", ownerSessionId: "s", statusReason: null, currentAttempt: null, attemptHistory: [] }], edges: [], artifacts: [], topologyVersions: [], gates: [], reviews: [], notifications: [] });
  assert.equal(detail.topologyHash, null);
  assert.equal(detail.nodes[0]?.stage, "goal");
});
test("runtime contracts reject malformed nested workflow data", () => {
  const snapshot = validSnapshot();
  snapshot.workflows = [{ runId: "r", sessionId: "s", workflowKey: "w", planPath: null, topologyHash: null, state: "not-an-enum", statusReason: null, nodeCount: 1, succeededCount: 0, failedCount: 0, updatedAt: "now" }];
  assert.throws(() => parseSnapshot(snapshot), /枚举值无效/);
});

test("runtime contracts accept persisted producer shapes", () => {
  const snapshot = validSnapshot();
  snapshot.workflows = [{ runId: "r", sessionId: null, workflowKey: "standalone", planPath: null, topologyHash: null, state: "registered", statusReason: null, nodeCount: 1, succeededCount: 0, failedCount: 0, updatedAt: "now" }];
  snapshot.validation.validationCopies = [{ job_id: "j", session_id: "s", job_root: "r", source_root: "s", target_root: "t", head_commit: "h", manifest_bytes: 42, status: "planned", created_at: "now", removed_at: null }];
  snapshot.validation.cargoJobs = [cargoLane()];
  snapshot.git.finalizeRequests = [{ request_id: "f", session_id: "s", message: "milestone", paths: ["a.rs"], categories: { code: ["a.rs"] }, untracked: [], validation: [["cargo", "test", "-p", "crate"]], maintenance: 0, status: "previewed", commit_sha: null, error_text: null, created_at: "now", completed_at: null }];
  assert.equal(parseSnapshot(snapshot).workflows[0]?.sessionId, null);
});

test("runtime contracts preserve the bounded coordinator experience projection", () => {
  const snapshot = validSnapshot();
  (snapshot as Record<string, unknown>).experience = {
    sync: { runs: 12, quietRuns: 9, visibleChanges: 3, averageDurationMs: 25 },
    blockers: [{ kind: "cargo", ownerSessionId: "session-a", laneKind: "test", status: "running", createdAt: "now" }],
  };
  assert.deepEqual(
    (parseSnapshot(snapshot) as unknown as Record<string, unknown>).experience,
    (snapshot as Record<string, unknown>).experience,
  );
});

test("runtime contracts accept only bounded same-plan continuation advice", () => {
  const snapshot = validSnapshot();
  (snapshot as Record<string, unknown>).experience = {
    sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 },
    blockers: [],
    continuations: [{
      sessionId: "waiting-owner", planPath: "docs/plans/tooling/01-workflow.md", waitKind: "validation",
      candidate: { milestone: "M1", title: "Write the remaining module documentation." },
      scopeClaimRequired: true, returnToPrimary: true,
    }],
  };
  assert.equal(parseSnapshot(snapshot).experience.continuations[0]?.candidate.milestone, "M1");

  (snapshot as unknown as { experience: { continuations: Array<{ waitKind: string }> } }).experience.continuations[0]!.waitKind = "global_drain";
  assert.throws(() => parseSnapshot(snapshot));
});

test("runtime contracts preserve the bounded validation reservation queue", () => {
  const snapshot = validSnapshot();
  snapshot.validation.cargoReservations = [{
    reservationId: "cpu-pending", sessionId: "session-a", laneScope: "cpu",
    executionMode: "warm", burstEligible: true, status: "pending", queuePosition: 2, createdAt: "now", expiresAt: "later",
  }];
  snapshot.validation.cpuBurst = { capacity: 1, active: 0, eligiblePending: 1 };
  assert.deepEqual(parseSnapshot(snapshot).validation.cargoReservations, snapshot.validation.cargoReservations);

  snapshot.validation.cargoReservations[0].queuePosition = 0;
  assert.throws(() => parseSnapshot(snapshot));
});

test("runtime contracts expose only a bounded process-observation conclusion for Cargo lanes", () => {
  const snapshot = validSnapshot();
  snapshot.validation.cargoJobs = [{ ...cargoLane(), status: "running", process_observation: "observed" }];
  snapshot.validation.currentCargoTargets = [{ ...cargoLane(), status: "running", process_observation: "observed" }];
  assert.equal(parseSnapshot(snapshot).validation.cargoJobs[0]?.process_observation, "observed");

  const legacy = validSnapshot();
  legacy.validation.cargoJobs = [{ ...cargoLane(), status: "running" }];
  assert.equal(parseSnapshot(legacy).validation.cargoJobs[0]?.process_observation, "awaiting_observation");

  snapshot.validation.cargoJobs[0].process_observation = "raw_pid";
  assert.throws(() => parseSnapshot(snapshot), /process_observation/);
});

test("runtime contracts accept the legacy projection during a rolling daemon upgrade", () => {
  const snapshot = validSnapshot();
  const collaboration = snapshot.collaboration as unknown as { baseline: Record<string, unknown> | null; patches: Record<string, unknown>[] };
  collaboration.baseline = { epoch_id: 1, head_commit: "h", index_tree: "t", health: "degraded", manifest_json: "{}", created_at: "now", degraded_at: "now", degraded_reason: "changed" };
  collaboration.patches = [{ patch_id: 1, session_id: "s", patch_object_hash: "h", targets: ["a"], base_hashes: {}, base_objects: {}, current_objects: null, status: "queued", error_text: null, created_at: "now", updated_at: "now", applied_at: null }];
  snapshot.validation.validationCopies = [{ job_id: "j", session_id: "s", job_root: "r", source_root: "s", target_root: "t", head_commit: "h", manifest: ["Cargo.toml"], status: "planned", created_at: "now", removed_at: null }];
  snapshot.validation.cargoReservations = [{ reservationId: "legacy", sessionId: "s", laneScope: "cpu", status: "pending", queuePosition: 1, createdAt: "now", expiresAt: "later" }];
  delete (snapshot.validation as { cpuBurst?: unknown }).cpuBurst;
  delete (snapshot.service as { sessionTtlSeconds?: unknown }).sessionTtlSeconds;
  assert.equal(parseSnapshot(snapshot).collaboration.baseline?.epoch_id, 1);
  assert.deepEqual(parseSnapshot(validSnapshot()).codexSessions.rows, []);
  assert.deepEqual(parseSnapshot(snapshot).validation.cpuBurst, { capacity: 1, active: 0, eligiblePending: 0 });
  assert.deepEqual(parseSnapshot(snapshot).validation.cargoReservations[0], {
    reservationId: "legacy", sessionId: "s", laneScope: "cpu", executionMode: "warm", burstEligible: false,
    status: "pending", queuePosition: 1, createdAt: "now", expiresAt: "later",
  });
  assert.equal(parseSnapshot(snapshot).service.sessionTtlSeconds, 600);
});

test("Codex Session contracts accept only bounded exact text projections", () => {
  const snapshot = validSnapshot();
  (snapshot as Record<string, unknown>).codexSessions = validCodexProjection();
  const parsed = parseSnapshot(snapshot);
  assert.equal(parsed.codexSessions.rows[0]?.threadId, "thread-12345678901234567890");

  for (const mutate of [
    (row: Record<string, unknown>) => { row.state = "running"; },
    (row: Record<string, unknown>) => { row.sourceLocation = "private-path"; },
    (row: Record<string, unknown>) => { row.lastEvent = "raw_message"; },
    (row: Record<string, unknown>) => { row.boundSessionId = ""; },
    (row: Record<string, unknown>) => { row.diagnosticCode = "x".repeat(161); },
    (row: Record<string, unknown>) => { row.rawRollout = { prompt: "secret" }; },
  ]) {
    const candidate = validSnapshot();
    const codex = validCodexProjection();
    mutate(codex.rows[0] as Record<string, unknown>);
    (candidate as Record<string, unknown>).codexSessions = codex;
    assert.throws(() => parseSnapshot(candidate));
  }

  const oversized = validSnapshot();
  const codex = validCodexProjection();
  codex.rows = Array.from({ length: 1001 }, () => ({ ...codex.rows[0] }));
  (oversized as Record<string, unknown>).codexSessions = codex;
  assert.throws(() => parseSnapshot(oversized), /1000/);
});

test("runtime contracts reject missing IDs, invalid enums, and malformed arrays in every control domain", () => {
  const cases: Array<(snapshot: ReturnType<typeof validSnapshot>) => void> = [
    (snapshot) => { snapshot.failures.nodes = [{ node_id: 1, lifecycle_key: "life", artifact_path: "failure.md", kind: "failure", status: "open", created_at: "now", resolved_at: null, summary_slug: "summary", origin_plan: "origin", fixing_plan: "fixer", origin_child_dir: "01", fixing_child_dir: "02", priority: 1, imported_at: "now" }]; delete snapshot.failures.nodes[0].node_id; },
    (snapshot) => { snapshot.collaboration.leases = [{ path_key: "p", display_path: "p", session_id: "s", base_hash: null, acquired_at: "now", last_heartbeat_at: "now", expires_at: "later" }]; delete snapshot.collaboration.leases[0].path_key; },
    (snapshot) => { snapshot.collaboration.patches = [{ patch_id: 1, session_id: "s", patch_object_hash: "h", targets: ["a"], content_bytes: 12, has_current_objects: 0, status: "unknown", error_text: null, created_at: "now", updated_at: "now", applied_at: null }]; },
    (snapshot) => { snapshot.validation.cargoJobs = [{ job_id: "j", session_id: "s", lane_kind: "check", status: "leased", created_at: "now", started_at: null, finished_at: null, released_at: null }]; },
    (snapshot) => { snapshot.validation.cargoJobs = [{ ...cargoLane(), cleanup_policy: "keep-forever" }]; },
    (snapshot) => { snapshot.validation.cargoJobs = [{ ...cargoLane(), cleanup_status: "unknown" }]; },
    (snapshot) => { snapshot.validation.validationCopies = [{ job_id: "j", session_id: "s", job_root: "r", source_root: "s", target_root: "t", head_commit: "h", manifest_bytes: 12, status: "invalid", created_at: "now", removed_at: null }]; },
    (snapshot) => { snapshot.git.finalizeRequests = [{ request_id: "r", session_id: "s", message: "m", paths: [], categories: { code: "not-an-array" }, untracked: [], validation: [["cargo", "test"]], maintenance: 0, status: "previewed", commit_sha: null, error_text: null, created_at: "now", completed_at: null }]; },
  ];
  for (const mutate of cases) {
    const snapshot = validSnapshot();
    mutate(snapshot);
    assert.throws(() => parseSnapshot(snapshot));
  }
});

function cargoLane() {
  return {
    job_id: "j", session_id: "s", lane_kind: "check", status: "released", created_at: "now",
    started_at: "now", finished_at: "now", released_at: "now", cleanup_policy: "retained", cleanup_status: "retained",
  };
}

function validSnapshot() {
  return {
    projectionVersion: 1, eventCursor: 0,
    service: { status: "ok", branch: "main", mode: "read_write", baseline: "healthy", instanceId: "i", startedAt: "now", sessionTtlSeconds: 3600, controlApiVersions: [1] },
    workflows: [] as Record<string, unknown>[], sessions: [],
    failures: { nodes: [] as Record<string, unknown>[], diagnostics: [] },
    collaboration: { baseline: null, leases: [] as Record<string, unknown>[], patches: [] as Record<string, unknown>[] },
    validation: { cargoJobs: [] as Record<string, unknown>[], currentCargoTargets: [] as Record<string, unknown>[], cargoReservations: [] as Record<string, unknown>[], cpuBurst: { capacity: 1, active: 0, eligiblePending: 0 }, artifactLifecycle: { reusablePools: 0, ephemeralTargets: 0, pendingCleanup: 0, failedCleanup: 0 }, validationCopies: [] as Record<string, unknown>[] },
    git: { finalizeRequests: [] as Record<string, unknown>[] }, audit: [],
  };
}

function validCodexProjection() {
  return {
    rows: [{
      threadId: "thread-12345678901234567890", sourceLocation: "active", state: "active",
      originator: "Codex Desktop", cliVersion: "0.test", threadSource: "user",
      lastEvent: "task_started", lastTurnId: "turn-one", boundSessionId: null,
      diagnosticCode: null, firstSeenAt: "2026-07-13T00:00:00Z",
      lastActivityAt: "2026-07-13T00:01:00Z", lastSyncedAt: "2026-07-13T00:01:01Z",
    }],
    total: 1, truncated: false,
    stateCounts: { active: 1, idle: 0, archived: 0, unavailable: 0 },
    sourceCounts: { active: 1, archived: 0, missing: 0 },
    queueDepth: 0, lastSuccessfulAt: "2026-07-13T00:01:01Z", lastTerminalCode: "succeeded",
    lastRun: { runId: "run-one", trigger: "hook", status: "succeeded", scannedCount: 1, changedCount: 1, diagnosticCount: 0, unavailableCount: 0, durationMs: 2, errorCode: null, createdAt: "2026-07-13T00:01:00Z", completedAt: "2026-07-13T00:01:01Z" },
  };
}
