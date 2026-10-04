import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { createWindowActionScheduler } from "../src/tauri/windowActionScheduler.ts";

const webRoot = path.resolve(import.meta.dirname, "..");
const hubRoot = path.resolve(webRoot, "..");

function readWebFile(relativePath) {
  return fs.readFileSync(path.join(webRoot, relativePath), "utf8");
}

function readHubFile(relativePath) {
  return fs.readFileSync(path.join(hubRoot, relativePath), "utf8");
}

test("frameless TopBar exposes a dedicated drag surface wired to startDragging", () => {
  const source = readWebFile("src/components/shell/TopBar.tsx");
  const dragSurface = /<Box\s+data-tauri-drag-region\s+onMouseDown=\{handleStartDragging\}/s;

  assert.match(source, /const handleStartDragging = \(event: MouseEvent<HTMLElement>\) =>/);
  assert.match(source, /if \(event\.button !== 0\)/);
  assert.match(source, /runWindowAction\("start-dragging", windowActionSchedulerRef\.current!/);
  assert.match(source, /appWindow\.startDragging\(\)/);
  assert.match(source, dragSurface);
  assert.doesNotMatch(source, /<Box\s+component="header"[^>]*data-tauri-drag-region/s);
});

test("the window capability explicitly grants only the drag command needed by the handle", () => {
  const capability = JSON.parse(readHubFile("capabilities/default.json"));

  assert.ok(capability.permissions.includes("core:window:allow-start-dragging"));
  assert.equal(
    capability.permissions.filter((permission) => permission === "core:window:allow-start-dragging").length,
    1,
  );
});

test("drag requests share the scheduler single-flight receipt and retry after settlement", async () => {
  const scheduler = createWindowActionScheduler(() => {});
  let resolveDrag;
  const drag = new Promise((resolve) => {
    resolveDrag = resolve;
  });
  let calls = 0;

  const first = scheduler.run("start-dragging", () => {
    calls += 1;
    return drag;
  });
  const duplicate = scheduler.run("start-dragging", () => {
    calls += 1;
    return drag;
  });

  assert.equal(first, duplicate);
  await Promise.resolve();
  assert.equal(calls, 1);
  resolveDrag();
  assert.equal(await first, true);
  assert.equal(scheduler.inFlightCount(), 0);
  assert.equal(await scheduler.run("start-dragging", async () => {
    calls += 1;
  }), true);
  assert.equal(calls, 2);
});
