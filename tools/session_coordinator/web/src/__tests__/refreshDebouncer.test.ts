import test from "node:test";
import assert from "node:assert/strict";
import { createResyncDebouncer } from "../state/refreshDebouncer";

const delay = (milliseconds: number) => new Promise((resolve) => setTimeout(resolve, milliseconds));

test("coalesces burst events into one snapshot resync", async () => {
  let resyncCount = 0;
  const debouncer = createResyncDebouncer(() => { resyncCount += 1; }, 1);

  debouncer.schedule();
  debouncer.schedule();
  debouncer.schedule();

  assert.equal(resyncCount, 0);
  await delay(10);
  assert.equal(resyncCount, 1);
});

test("immediate resync cancels the queued event resync", async () => {
  let resyncCount = 0;
  const debouncer = createResyncDebouncer(() => { resyncCount += 1; }, 1000);

  debouncer.schedule();
  debouncer.flush();

  assert.equal(resyncCount, 1);
  await delay(10);
  assert.equal(resyncCount, 1);
});
