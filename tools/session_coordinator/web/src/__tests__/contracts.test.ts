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
  snapshot.validation.cargoJobs = [cargoJob()];
  snapshot.git.finalizeRequests = [{ request_id: "f", session_id: "s", message: "milestone", paths: ["a.rs"], categories: { code: ["a.rs"] }, untracked: [], validation: [["cargo", "test", "-p", "crate"]], maintenance: 0, status: "previewed", commit_sha: null, error_text: null, created_at: "now", completed_at: null }];
  assert.equal(parseSnapshot(snapshot).workflows[0]?.sessionId, null);
});

test("runtime contracts accept the legacy projection during a rolling daemon upgrade", () => {
  const snapshot = validSnapshot();
  const collaboration = snapshot.collaboration as unknown as { baseline: Record<string, unknown> | null; patches: Record<string, unknown>[] };
  collaboration.baseline = { epoch_id: 1, head_commit: "h", index_tree: "t", health: "degraded", manifest_json: "{}", created_at: "now", degraded_at: "now", degraded_reason: "changed" };
  collaboration.patches = [{ patch_id: 1, session_id: "s", patch_object_hash: "h", targets: ["a"], base_hashes: {}, base_objects: {}, current_objects: null, status: "queued", error_text: null, created_at: "now", updated_at: "now", applied_at: null }];
  snapshot.validation.validationCopies = [{ job_id: "j", session_id: "s", job_root: "r", source_root: "s", target_root: "t", head_commit: "h", manifest: ["Cargo.toml"], status: "planned", created_at: "now", removed_at: null }];
  assert.equal(parseSnapshot(snapshot).collaboration.baseline?.epoch_id, 1);
});

test("runtime contracts reject missing IDs, invalid enums, and malformed arrays in every control domain", () => {
  const cases: Array<(snapshot: ReturnType<typeof validSnapshot>) => void> = [
    (snapshot) => { snapshot.failures.nodes = [{ node_id: 1, lifecycle_key: "life", artifact_path: "failure.md", kind: "failure", status: "open", created_at: "now", resolved_at: null, summary_slug: "summary", origin_plan: "origin", fixing_plan: "fixer", origin_child_dir: "01", fixing_child_dir: "02", priority: 1, imported_at: "now" }]; delete snapshot.failures.nodes[0].node_id; },
    (snapshot) => { snapshot.collaboration.leases = [{ path_key: "p", display_path: "p", session_id: "s", base_hash: null, acquired_at: "now", last_heartbeat_at: "now", expires_at: "later" }]; delete snapshot.collaboration.leases[0].path_key; },
    (snapshot) => { snapshot.collaboration.patches = [{ patch_id: 1, session_id: "s", patch_object_hash: "h", targets: ["a"], content_bytes: 12, has_current_objects: 0, status: "unknown", error_text: null, created_at: "now", updated_at: "now", applied_at: null }]; },
    (snapshot) => { snapshot.validation.cargoJobs = [{ job_id: "j", session_id: "s", lane_kind: "check", target_dir: "t", status: "leased", dry_run: 0, pid: null, command: "cargo", exit_code: null, created_at: "now", last_heartbeat_at: "now", started_at: null, finished_at: null, released_at: null }]; },
    (snapshot) => { snapshot.validation.cargoJobs = [{ ...cargoJob(), cleanup_policy: "keep-forever" }]; },
    (snapshot) => { snapshot.validation.cargoJobs = [{ ...cargoJob(), cleanup_status: "unknown" }]; },
    (snapshot) => { snapshot.validation.validationCopies = [{ job_id: "j", session_id: "s", job_root: "r", source_root: "s", target_root: "t", head_commit: "h", manifest_bytes: 12, status: "invalid", created_at: "now", removed_at: null }]; },
    (snapshot) => { snapshot.git.finalizeRequests = [{ request_id: "r", session_id: "s", message: "m", paths: [], categories: { code: "not-an-array" }, untracked: [], validation: [["cargo", "test"]], maintenance: 0, status: "previewed", commit_sha: null, error_text: null, created_at: "now", completed_at: null }]; },
  ];
  for (const mutate of cases) {
    const snapshot = validSnapshot();
    mutate(snapshot);
    assert.throws(() => parseSnapshot(snapshot));
  }
});

function cargoJob() {
  return {
    job_id: "j", session_id: "s", lane_kind: "check", target_dir: "D:/cargo-targets/pool", status: "released",
    dry_run: 0, pid: null, command: ["cargo", "check"], exit_code: 0, created_at: "now", last_heartbeat_at: "now",
    started_at: "now", finished_at: "now", released_at: "now", reuse_key: "reuse", compatibility_key: "compatibility",
    reuse_profile: "{}", reused_from_job_id: null, cleanup_policy: "retained", cleanup_status: "retained", cleanup_error: null,
  };
}

function validSnapshot() {
  return {
    projectionVersion: 1, eventCursor: 0,
    service: { status: "ok", branch: "main", mode: "read_write", baseline: "healthy", instanceId: "i", startedAt: "now", controlApiVersions: [1] },
    workflows: [] as Record<string, unknown>[], sessions: [],
    failures: { nodes: [] as Record<string, unknown>[], diagnostics: [] },
    collaboration: { baseline: null, leases: [] as Record<string, unknown>[], patches: [] as Record<string, unknown>[] },
    validation: { cargoJobs: [] as Record<string, unknown>[], validationCopies: [] as Record<string, unknown>[] },
    git: { finalizeRequests: [] as Record<string, unknown>[] }, audit: [],
  };
}
