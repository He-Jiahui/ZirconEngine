import assert from "node:assert/strict";
import test from "node:test";

import {
  shouldAutoHideTask,
  shouldOpenTaskNotification,
  taskNotificationKey,
} from "../src/tauri/hubNotification.ts";

function task(overrides = {}) {
  return {
    label: "Build complete",
    detail: "Editor runtime is ready",
    tone: "success",
    running: false,
    cancellable: false,
    recovery: null,
    operation: "Build",
    progressPercent: 100,
    taskId: 41,
    queued: 0,
    ...overrides,
  };
}

test("task notification identity separates terminal operations with the same copy", () => {
  const first = taskNotificationKey(task({ taskId: 41 }));
  const same = taskNotificationKey(task({ taskId: 41 }));
  const next = taskNotificationKey(task({ taskId: 42 }));

  assert.equal(first, same);
  assert.notEqual(first, next);
});

test("task notification opening ignores unrelated state updates after acknowledgement", () => {
  const current = task({ taskId: 41 });
  const key = taskNotificationKey(current);

  assert.equal(shouldOpenTaskNotification(null, current), true);
  assert.equal(shouldOpenTaskNotification(key, task(current)), false);
  assert.equal(shouldOpenTaskNotification(key, task({ ...current, queued: 1 })), false);
  assert.equal(shouldOpenTaskNotification(key, task({ taskId: 42 })), true);
  assert.equal(shouldOpenTaskNotification(key, task({ tone: "neutral", recovery: null })), false);
});

test("running tasks stay visible while terminal notifications may auto-hide", () => {
  assert.equal(shouldAutoHideTask(task({ running: true })), false);
  assert.equal(shouldAutoHideTask(task({ running: false })), true);
});
