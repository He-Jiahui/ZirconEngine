import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const root = new URL("../src/", import.meta.url);

test("HubSelect requires and forwards an explicit label", async () => {
  const select = await readFile(new URL("components/inputs/HubSelect.tsx", root), "utf8");

  assert.match(select, /label: string/);
  assert.match(select, /\{ value, options, label, minWidth/);
  assert.match(select, /inputProps=\{\{ "aria-label": label \}\}/);
});

test("project filter and sort labels are localized across Rust and Web DTOs", async () => {
  const [types, fallback, rustText, toolbar, browser] = await Promise.all([
    readFile(new URL("types/hub.ts", root), "utf8"),
    readFile(new URL("data/hubData.ts", root), "utf8"),
    readFile(new URL("../../src/tauri_app/view_model/ui_text.rs", import.meta.url), "utf8"),
    readFile(new URL("components/inputs/ProjectsToolbar.tsx", root), "utf8"),
    readFile(new URL("pages/ProjectBrowserPage.tsx", root), "utf8"),
  ]);

  for (const source of [types, fallback, rustText]) {
    assert.match(source, /filterLabel|filter_label/);
    assert.match(source, /sortLabel|sort_label/);
  }
  for (const source of [toolbar, browser]) {
    assert.match(source, /label=\{text\.filterLabel\}/);
    assert.match(source, /label=\{text\.sortLabel\}/);
  }
});
