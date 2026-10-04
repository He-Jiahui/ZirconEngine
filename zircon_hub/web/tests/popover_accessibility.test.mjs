import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceRoot = new URL("../src/", import.meta.url);

async function source(path) {
  return readFile(new URL(path, sourceRoot), "utf8");
}

test("topbar popover triggers expose an explicit expanded-controls relationship", async () => {
  const topBar = await source("components/shell/TopBar.tsx");

  assert.match(topBar, /aria-haspopup="dialog"/g);
  assert.match(topBar, /aria-label=\{engineLabel\}/);
  assert.match(topBar, /aria-label=\{userName\}/);
  assert.match(topBar, /aria-expanded=\{Boolean\(engineAnchor\)\}/);
  assert.match(topBar, /aria-expanded=\{Boolean\(userAnchor\)\}/);
  assert.match(topBar, /aria-controls=\{sourceEnginePopoverId\}/);
  assert.match(topBar, /aria-controls=\{userMenuPopoverId\}/);
  assert.match(topBar, /id=\{sourceEnginePopoverId\}/);
  assert.match(topBar, /id=\{userMenuPopoverId\}/);
});

test("shared HubPopover publishes a named dialog surface", async () => {
  const popover = await source("components/overlays/HubPopover.tsx");

  assert.match(popover, /id\?: string/);
  assert.match(popover, /ariaLabel\?: string/);
  assert.match(popover, /role: "dialog"/);
  assert.match(popover, /"aria-label": ariaLabel/);
});
