import test from "node:test";
import assert from "node:assert/strict";
import { controlReducer, initialControlState } from "../state/reducer";

const event = (id: number) => ({ id, type: "test", payload: {}, createdAt: "now" });
test("event reconciliation deduplicates old ids", () => assert.equal(controlReducer({ ...initialControlState, cursor: 4 }, { type: "event", event: event(4) }).cursor, 4));
test("event reconciliation requires resync for gaps", () => assert.equal(controlReducer({ ...initialControlState, cursor: 4 }, { type: "event", event: event(6) }).needsRefresh, true));
test("event reconciliation advances contiguous ids", () => assert.equal(controlReducer({ ...initialControlState, cursor: 4 }, { type: "event", event: event(5) }).cursor, 5));
