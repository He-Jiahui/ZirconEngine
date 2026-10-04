import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceRoot = new URL("../src/", import.meta.url);

async function source(path) {
  return readFile(new URL(path, sourceRoot), "utf8");
}

test("HubToggle requires and publishes an accessible group name", async () => {
  const toggle = await source("components/inputs/HubToggle.tsx");

  assert.match(toggle, /ariaLabel: string/);
  assert.match(toggle, /function HubToggle\(\{ value, options, onChange, ariaLabel \}: HubToggleProps\)/);
  assert.match(toggle, /<ToggleButtonGroup[\s\S]*aria-label=\{ariaLabel\}/);
});

test("project view toggles use the localized dashboard and browser titles as group names", async () => {
  const toolbar = await source("components/inputs/ProjectsToolbar.tsx");
  const browser = await source("pages/ProjectBrowserPage.tsx");

  assert.match(toolbar, /<HubToggle[\s\S]*ariaLabel=\{text\.title\}/);
  assert.match(browser, /<HubToggle[\s\S]*ariaLabel=\{text\.browserTitle\}/);
});
