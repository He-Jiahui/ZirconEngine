import assert from "node:assert/strict";
import test from "node:test";

import { createWindowActionScheduler } from "../src/tauri/windowActionScheduler.ts";

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, reject, resolve };
}

test("coalesces duplicate in-flight actions without blocking other action kinds", async () => {
  const scheduler = createWindowActionScheduler(() => {});
  const minimize = deferred();
  const close = deferred();
  let minimizeCalls = 0;
  let closeCalls = 0;

  const firstMinimize = scheduler.run("minimize", () => {
    minimizeCalls += 1;
    return minimize.promise;
  });
  const duplicateMinimize = scheduler.run("minimize", () => {
    minimizeCalls += 1;
    return minimize.promise;
  });
  const firstClose = scheduler.run("close", () => {
    closeCalls += 1;
    return close.promise;
  });

  assert.equal(firstMinimize, duplicateMinimize);
  await Promise.resolve();
  assert.equal(minimizeCalls, 1);
  assert.equal(closeCalls, 1);
  assert.equal(scheduler.inFlightCount(), 2);

  minimize.resolve();
  close.resolve();
  assert.equal(await firstMinimize, true);
  assert.equal(await firstClose, true);
  assert.equal(scheduler.inFlightCount(), 0);
});

test("reports sync and async failures once and admits a retry after settlement", async () => {
  const failures = [];
  const scheduler = createWindowActionScheduler((action, error) => failures.push({ action, error }));
  const rejection = new Error("window unavailable");
  let calls = 0;

  const first = scheduler.run("toggle-maximize", () => {
    calls += 1;
    return Promise.reject(rejection);
  });
  const duplicate = scheduler.run("toggle-maximize", () => {
    calls += 1;
    return Promise.resolve();
  });

  assert.equal(first, duplicate);
  assert.equal(await first, false);
  assert.equal(calls, 1);
  assert.deepEqual(failures, [{ action: "toggle-maximize", error: rejection }]);

  assert.equal(
    await scheduler.run("toggle-maximize", () => {
      calls += 1;
      throw new Error("permission denied");
    }),
    false,
  );
  assert.equal(calls, 2);
  assert.equal(failures.length, 2);

  assert.equal(
    await scheduler.run("toggle-maximize", () => {
      calls += 1;
      return Promise.resolve();
    }),
    true,
  );
  assert.equal(calls, 3);
  assert.equal(scheduler.inFlightCount(), 0);
});

test("reduces repeated native dispatches by 99 percent across burst groups", async () => {
  const scheduler = createWindowActionScheduler(() => {});
  const burstCount = 21;
  const inputsPerBurst = 100;
  let optimizedDispatches = 0;

  for (let burst = 0; burst < burstCount; burst += 1) {
    const action = deferred();
    const receipts = Array.from({ length: inputsPerBurst }, () =>
      scheduler.run("close", () => {
        optimizedDispatches += 1;
        return action.promise;
      }),
    );
    action.resolve();
    assert.deepEqual(await Promise.all(receipts), Array(inputsPerBurst).fill(true));
  }

  const legacyDispatches = burstCount * inputsPerBurst;
  const reductionPercent = ((legacyDispatches - optimizedDispatches) / legacyDispatches) * 100;
  assert.equal(optimizedDispatches, burstCount);
  assert.equal(reductionPercent, 99);
  console.log(
    `PERF_RESULT window_action_dispatch legacy=${legacyDispatches} optimized=${optimizedDispatches} reduction_percent=${reductionPercent.toFixed(3)}`,
  );
});
