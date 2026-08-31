import test from "node:test";
import assert from "node:assert/strict";
import { validationQueueLanes } from "../components/validation/validationQueueModel";

test("validation queue separates lanes and preserves FIFO order", () => {
  const lanes = validationQueueLanes([
    { reservationId: "gpu-2", sessionId: "gpu-next", laneScope: "gpu", executionMode: "warm", burstEligible: false, status: "pending", queuePosition: 2, createdAt: "2026-08-24T00:02:00Z", expiresAt: "2026-08-24T01:02:00Z" },
    { reservationId: "cpu-run", sessionId: "cpu-active", laneScope: "cpu", executionMode: "warm", burstEligible: false, status: "running", queuePosition: 1, createdAt: "2026-08-24T00:00:00Z", expiresAt: "2026-08-24T01:00:00Z" },
    { reservationId: "gpu-1", sessionId: "gpu-active", laneScope: "gpu", executionMode: "warm", burstEligible: false, status: "running", queuePosition: 1, createdAt: "2026-08-24T00:01:00Z", expiresAt: "2026-08-24T01:01:00Z" },
  ]);

  assert.deepEqual(lanes.map((lane) => lane.scope), ["cpu", "gpu"]);
  assert.deepEqual(lanes[1]?.items.map((item) => item.reservationId), ["gpu-1", "gpu-2"]);
  assert.equal(lanes[1]?.pending, 1);
  assert.equal(lanes[0]?.running, 1);
});
