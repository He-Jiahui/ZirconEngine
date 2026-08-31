import test from "node:test";
import assert from "node:assert/strict";
import { validationRunProgress } from "../components/validation/validationRunModel";

test("running validation reports elapsed execution stage and output health", () => {
  const rows = validationRunProgress([
    {
      job_id: "job-a", session_id: "session-a", lane_kind: "test", status: "running",
      created_at: "2026-08-24T00:00:00Z", started_at: "2026-08-24T00:01:00Z", finished_at: null,
      released_at: null, cleanup_policy: "retained", cleanup_status: "retained", process_observation: "observed",
    },
  ], [{
    runId: "run-a", jobId: "job-a", sessionId: "session-a", startedAt: "2026-08-24T00:01:00Z",
    outputState: "output_observed", lastOutputAt: "2026-08-24T00:02:00Z",
  }], new Date("2026-08-24T00:03:05Z"));

  assert.deepEqual(rows[0], {
    jobId: "job-a", sessionId: "session-a", lane: "test", state: "running", elapsed: "2m 5s",
    stepIndex: 3, stepCount: 4, stepLabel: "执行验证命令", outputLabel: "已有实时输出",
  });
});

test("terminal validation reports its completed pipeline stage", () => {
  const rows = validationRunProgress([
    {
      job_id: "job-b", session_id: "session-b", lane_kind: "check", status: "succeeded",
      created_at: "2026-08-24T00:00:00Z", started_at: "2026-08-24T00:00:05Z", finished_at: "2026-08-24T00:01:05Z",
      released_at: null, cleanup_policy: "retained", cleanup_status: "retained", process_observation: "observed",
    },
  ], [], new Date("2026-08-24T00:03:00Z"));

  assert.equal(rows[0]?.stepIndex, 4);
  assert.equal(rows[0]?.stepLabel, "验证完成");
  assert.equal(rows[0]?.elapsed, "1m 0s");
});
