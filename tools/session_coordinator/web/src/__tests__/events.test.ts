import test from "node:test";
import assert from "node:assert/strict";
import { parseControlEvent } from "../api/validation";
import { openControlEvents } from "../api/events";

test("SSE events reject missing ids", () => assert.throws(() => parseControlEvent("", "{}")));
test("SSE payload stays plain text", () => {
    const parsed = parseControlEvent("2", '{"type":"log","payload":{"message":"<img onerror=alert(1)>"},"createdAt":"now"}');
    assert.equal(parsed.payload.message, "<img onerror=alert(1)>");
});
test("SSE reports disconnect and reconnect", () => {
  const states: boolean[] = [];
  class FakeEventSource {
    static instance: FakeEventSource; onopen: (() => void) | null = null; onerror: (() => void) | null = null;
    constructor() { FakeEventSource.instance = this; }
    addEventListener() {} close() {}
  }
  globalThis.EventSource = FakeEventSource as unknown as typeof EventSource;
  openControlEvents(4, { onEvent() {}, onResync() {}, onConnection: (connected) => states.push(connected), onError() {} });
  FakeEventSource.instance.onerror?.(); FakeEventSource.instance.onopen?.();
  assert.deepEqual(states, [false, true]);
});
