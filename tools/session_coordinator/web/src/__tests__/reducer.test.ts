import test from "node:test";
import assert from "node:assert/strict";
import { controlReducer, initialControlState, snapshotRetryDelay } from "../state/reducer";

const event = (id: number) => ({ id, type: "test", payload: {}, createdAt: "now" });
test("event reconciliation deduplicates old ids", () => assert.equal(controlReducer({ ...initialControlState, cursor: 4 }, { type: "event", event: event(4) }).cursor, 4));
test("event reconciliation requires resync for gaps", () => assert.equal(controlReducer({ ...initialControlState, cursor: 4 }, { type: "event", event: event(6) }).needsRefresh, true));
test("event reconciliation advances contiguous ids", () => assert.equal(controlReducer({ ...initialControlState, cursor: 4 }, { type: "event", event: event(5) }).cursor, 5));

test("snapshot failures leave blocking load state and remain retryable", () => {
  const failed = controlReducer(
    { ...initialControlState, loading: true, needsRefresh: true },
    { type: "error", message: "控制服务请求失败" },
  );

  assert.equal(failed.loading, false);
  assert.equal(failed.needsRefresh, false);
  assert.equal(failed.error, "控制服务请求失败");
  assert.equal(failed.retryNonce, 1);

  const retrying = controlReducer(failed, { type: "resync" });
  assert.equal(retrying.needsRefresh, true);
  assert.equal(retrying.error, "控制服务请求失败");

  const failedAgain = controlReducer(retrying, { type: "error", message: "控制服务请求失败" });
  assert.equal(failedAgain.retryNonce, 2);
});

test("snapshot retry uses bounded exponential backoff", () => {
  assert.deepEqual(
    [0, 1, 2, 3, 4, 20].map(snapshotRetryDelay),
    [1_000, 2_000, 4_000, 8_000, 10_000, 10_000],
  );
});
