import test from "node:test";
import assert from "node:assert/strict";
import type { ControlSnapshot, ValidationHistoryTicket } from "../api/contracts";
import { buildAnalytics, buildBuckets, buildFailure, buildModules, buildPatchStatus, buildPlans, buildSchedule, buildValidationReport } from "../components/dashboard/analyticsModel";

const now = new Date("2026-08-30T12:00:00Z");

function ticket(id: string, status: ValidationHistoryTicket["status"], createdAt: string, updatedAt: string, planPath = "docs/plans/optimize/zircon_runtime/01-review.md"): ValidationHistoryTicket {
  return { ticketId: id, sessionId: `session-${id}`, planPath, status, sourceManifestHash: "hash", command: ["cargo", "test"], commandTruncated: false, createdAt, updatedAt, events: [], eventsTruncated: false };
}

function snapshot(overrides: Record<string, unknown> = {}): ControlSnapshot {
  return {
    workflows: [], sessions: [], failures: { nodes: [], diagnostics: [] }, validation: { cargoJobs: [], currentCargoTargets: [], cargoReservations: [], validationCopies: [], artifactLifecycle: { reusablePools: 0, ephemeralTargets: 0, pendingCleanup: 0, failedCleanup: 0 }, cpuBurst: { capacity: 1, active: 0, eligiblePending: 0 } }, collaboration: { baseline: null, leases: [], patches: [] }, git: { finalizeRequests: [] }, audit: [], experience: { sync: { runs: 0, quietRuns: 0, visibleChanges: 0, averageDurationMs: 0 }, blockers: [], continuations: [] }, ...overrides,
  } as unknown as ControlSnapshot;
}

test("analytics buckets count ticket starts and terminal outcomes in the last 24 hours", () => {
  const tickets = [ticket("passed", "passed", "2026-08-30T10:15:00Z", "2026-08-30T10:45:00Z"), ticket("failed", "failed", "2026-08-30T10:20:00Z", "2026-08-30T11:00:00Z")];
  const buckets = buildBuckets(tickets, [], now);
  const startBucket = buckets.find((bucket) => bucket.started === 2);
  const completedBucket = buckets.find((bucket) => bucket.completed === 1);
  const failedBucket = buckets.find((bucket) => bucket.failed === 1);
  assert.ok(startBucket);
  assert.ok(completedBucket);
  assert.ok(failedBucket);
  assert.equal(startBucket?.label, `${String(new Date("2026-08-30T10:15:00Z").getHours()).padStart(2, "0")}时`);
  assert.equal(completedBucket?.label, `${String(new Date("2026-08-30T10:45:00Z").getHours()).padStart(2, "0")}时`);
  assert.equal(failedBucket?.label, `${String(new Date("2026-08-30T11:00:00Z").getHours()).padStart(2, "0")}时`);
});

test("analytics schedule preserves hourly positions for queue, running, completed and failed tasks", () => {
  const data = snapshot({ validation: { cargoReservations: [{ createdAt: "2026-08-30T03:10:00+08:00", status: "pending" }], currentCargoTargets: [{ job_id: "running", session_id: "s", lane_kind: "check", status: "running", created_at: "2026-08-30T05:00:00+08:00" }, { job_id: "done", session_id: "s", lane_kind: "test", status: "succeeded", created_at: "2026-08-30T06:00:00+08:00" }, { job_id: "bad", session_id: "s", lane_kind: "test", status: "failed", created_at: "2026-08-30T07:00:00+08:00" }] } });
  const slots = buildSchedule(data, [], now);
  assert.equal(slots[3]?.queued, 1);
  assert.equal(slots[5]?.running, 1);
  assert.equal(slots[6]?.completed, 1);
  assert.equal(slots[7]?.failed, 1);
});

test("analytics failure ratio and resolution time are derived from lifecycle history", () => {
  const result = buildFailure(snapshot(), { chains: [{ lifecycleKey: "a", summarySlug: "a", status: "fixed", priority: 0, originPlan: "o", fixingPlan: "f", artifactPath: "a", createdAt: "2026-08-30T00:00:00Z", resolvedAt: "2026-08-30T01:00:00Z", events: [] }, { lifecycleKey: "b", summarySlug: "b", status: "open", priority: 0, originPlan: "o", fixingPlan: "f", artifactPath: "b", createdAt: "2026-08-30T00:00:00Z", resolvedAt: null, events: [] }], statusCounts: { open: 1, fixed: 1 }, truncated: false }, now);
  assert.equal(result.open, 1);
  assert.equal(result.fixed, 1);
  assert.equal(result.ratio, 0.5);
  assert.equal(result.averageResolutionSeconds, 3600);
  assert.equal(result.historyReady, true);
});

test("analytics groups plan nodes into completion, todo, failure and module shares", () => {
  const data = snapshot({ workflows: [{ runId: "run", sessionId: "s", workflowKey: "workflow", planPath: "docs/plans/engine/module/01.md", topologyHash: null, state: "running", statusReason: null, nodeCount: 4, succeededCount: 2, failedCount: 1, updatedAt: "2026-08-30T10:00:00Z" }] });
  const plans = buildPlans(data, []);
  const modules = buildModules(data, []);
  assert.deepEqual(plans[0] && { completed: plans[0].completedCount, todo: plans[0].todoCount, failed: plans[0].failedCount, ratio: plans[0].ratio, status: plans[0].status }, { completed: 2, todo: 2, failed: 1, ratio: 50, status: "blocked" });
  assert.equal(modules[0]?.label, "engine/module");
  const analytics = buildAnalytics(data, { now });
  assert.equal(analytics.coverage.hasTimeSeries, false);
  assert.equal(analytics.inProgressWorkflowCount, 1);
  assert.equal(analytics.queuedWorkflowCount, 0);
});

test("analytics keeps the full failure patch lifecycle visible", () => {
  const rows = buildPatchStatus([
    { patch_id: 1, session_id: "s", patch_object_hash: "a", targets: ["src/a.rs"], status: "applied", error_text: null, created_at: "2026-08-30T01:00:00+08:00", updated_at: "2026-08-30T02:00:00+08:00", applied_at: "2026-08-30T02:00:00+08:00" },
    { patch_id: 2, session_id: "s", patch_object_hash: "b", targets: ["src/b.rs"], status: "failed", error_text: "conflict", created_at: "2026-08-30T01:00:00+08:00", updated_at: "2026-08-30T02:00:00+08:00", applied_at: null },
    { patch_id: 3, session_id: "s", patch_object_hash: "c", targets: ["src/c.rs"], status: "needs_rebase", error_text: null, created_at: "2026-08-30T01:00:00+08:00", updated_at: "2026-08-30T02:00:00+08:00", applied_at: null },
  ]);
  assert.deepEqual(Object.fromEntries(rows.map((row) => [row.label, row.value])), { "排队": 0, "应用中": 0, "已应用": 1, "需 Rebase": 1, "失败": 1, "已取消": 0 });
});

test("validation report separates full-database counts from the bounded detail window", () => {
  const failed = ticket("failed", "failed", "2026-08-30T10:00:00Z", "2026-08-30T11:00:00Z");
  failed.events = [
    { eventId: 1, type: "validation.failed", createdAt: "2026-08-30T11:00:00Z", fromStatus: "running", toStatus: "failed", phase: "run", errorCode: "compile_failed", jobId: "job", runId: "run", exitCode: 1 },
    { eventId: 2, type: "validation.failure_recorded", createdAt: "2026-08-30T11:00:01Z", fromStatus: "failed", toStatus: "failed", phase: "run", errorCode: "compile_failed", jobId: "job", runId: "run", exitCode: 1 },
  ];
  failed.eventsTruncated = true;
  const history = {
    tickets: [ticket("passed", "passed", "2026-08-30T09:00:00Z", "2026-08-30T10:00:00Z"), failed],
    statusCounts: { queued: 3, materializing: 2, running: 1, passed: 30, failed: 10, snapshot_stale: 4 },
    truncated: true,
  };

  const report = buildValidationReport(history, buildBuckets(history.tickets, [], now));

  assert.equal(report.loaded, true);
  assert.equal(report.total, 50);
  assert.equal(report.terminal, 40);
  assert.equal(report.successRate, 0.75);
  assert.equal(report.backlog, 6);
  assert.deepEqual(report.last24Hours, { started: 2, passed: 1, failed: 1, successRate: 0.5 });
  assert.equal(report.sampleSize, 2);
  assert.equal(report.sampleTruncated, true);
  assert.equal(report.eventDetailsTruncated, 1);
  assert.deepEqual(report.failureReasons, [{ code: "compile_failed", count: 1, phase: "run" }]);
});

test("validation report keeps unloaded history distinct from an empty database", () => {
  const report = buildValidationReport(null, []);
  assert.equal(report.loaded, false);
  assert.equal(report.total, null);
  assert.equal(report.successRate, null);
  assert.equal(report.failureReasons.length, 0);
});
