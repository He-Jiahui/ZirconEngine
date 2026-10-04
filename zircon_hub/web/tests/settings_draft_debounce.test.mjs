import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import test from "node:test";

import { DebouncedSettingsDraft, SettingsActionScheduler } from "../src/settings/debouncedSettingsDraft.ts";

const SAMPLE_PAIRS = 21;
const BURST_UPDATES = 100;

class FakeTimer {
  #nextId = 0;
  #pending = new Map();

  schedule(callback) {
    const id = ++this.#nextId;
    this.#pending.set(id, callback);
    return id;
  }

  cancel(id) {
    this.#pending.delete(id);
  }

  flush() {
    const callbacks = [...this.#pending.values()];
    this.#pending.clear();
    for (const callback of callbacks) {
      callback();
    }
  }

  get pendingCount() {
    return this.#pending.size;
  }
}

test("settings draft debounce dispatches only the last draft in each burst", () => {
  let optimizedDispatches = 0;
  const startedAt = performance.now();

  for (let sample = 0; sample < SAMPLE_PAIRS; sample += 1) {
    const timer = new FakeTimer();
    const dispatched = [];
    const drafts = new DebouncedSettingsDraft((draft) => dispatched.push(draft), 200, timer);

    for (let update = 0; update < BURST_UPDATES; update += 1) {
      drafts.schedule({ sample, update, value: `draft-${sample}-${update}` });
    }

    assert.equal(timer.pendingCount, 1);
    timer.flush();
    assert.deepEqual(dispatched, [
      { sample, update: BURST_UPDATES - 1, value: `draft-${sample}-${BURST_UPDATES - 1}` },
    ]);
    optimizedDispatches += dispatched.length;
  }

  const legacyDispatches = SAMPLE_PAIRS * BURST_UPDATES;
  const elapsedNs = Math.max(1, Math.round((performance.now() - startedAt) * 1_000_000));
  assert.equal(optimizedDispatches, SAMPLE_PAIRS);
  assert.equal(optimizedDispatches * 100, legacyDispatches);
  console.log(
    `HUB02_SETTINGS_DRAFT_DEBOUNCE_V1 sample_pairs=${SAMPLE_PAIRS} burst_updates=${BURST_UPDATES} ` +
      `legacy_dispatches=${legacyDispatches} optimized_dispatches=${optimizedDispatches} ` +
      `dispatch_reduction_pct=99.000 quiet_window_ms=200 elapsed_ns=${elapsedNs}`,
  );
});

test("settings draft debounce cancellation prevents stale state publication", () => {
  const timer = new FakeTimer();
  const dispatched = [];
  const drafts = new DebouncedSettingsDraft((draft) => dispatched.push(draft), 200, timer);

  drafts.schedule({ value: "stale-before-save" });
  drafts.cancel();
  timer.flush();
  assert.deepEqual(dispatched, []);

  drafts.schedule({ value: "current" });
  timer.flush();
  assert.deepEqual(dispatched, [{ value: "current" }]);
});

test("settings barriers wait for an in-flight draft and prevent post-save stale updates", async () => {
  const timer = new FakeTimer();
  const updateFinished = deferred();
  const calls = [];
  const scheduler = new SettingsActionScheduler(
    async (draft) => {
      calls.push(`update:${draft.value}:start`);
      await updateFinished.promise;
      calls.push(`update:${draft.value}:finish`);
    },
    200,
    timer,
  );

  scheduler.schedule({ value: "current" });
  timer.flush();
  await Promise.resolve();
  const save = scheduler.runBarrier(async () => {
    calls.push("save");
  });

  assert.deepEqual(calls, ["update:current:start"]);
  updateFinished.resolve();
  await save;
  assert.deepEqual(calls, ["update:current:start", "update:current:finish", "save"]);
  timer.flush();
  assert.deepEqual(calls, ["update:current:start", "update:current:finish", "save"]);
});

test("SettingsPage cancels pending draft publication at explicit workflow boundaries", () => {
  const source = readFileSync(new URL("../src/pages/SettingsPage.tsx", import.meta.url), "utf8");

  assert.match(source, /useDebouncedSettingsDraft/);
  assert.match(source, /runSettingsBarrier\(\(\) => onAction\(HUB_ACTION\.saveSettings/s);
  assert.match(source, /runSettingsBarrier\(\(\) => onAction\(HUB_ACTION\.browseSettingsFolder/s);
  assert.match(source, /runSettingsBarrier\(\(\) => onAction\(HUB_ACTION\.discardSettingsDraft/s);
  assert.match(source, /runSettingsBarrier\(\(\) => onAction\(HUB_ACTION\.restoreDefaultSettings/s);
});

function deferred() {
  let resolve;
  const promise = new Promise((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}
