import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceRoot = new URL("../src/components/data/", import.meta.url);

async function source(name) {
  return readFile(new URL(name, sourceRoot), "utf8");
}

test("shared tree exposes a keyboard-operable WAI-ARIA tree contract", async () => {
  const tree = await source("HubTreeView.tsx");

  assert.match(tree, /role=\"tree\"/);
  assert.match(tree, /role=\"treeitem\"/);
  assert.match(tree, /role=\"group\"/);
  assert.match(tree, /aria-level=/);
  assert.match(tree, /aria-expanded=/);
  assert.match(tree, /aria-selected=/);
  assert.match(tree, /tabIndex=/);
  for (const key of ["ArrowDown", "ArrowUp", "ArrowRight", "ArrowLeft", "Home", "End"]) {
    assert.match(tree, new RegExp(`case \\\"${key}\\\"`));
  }
});

test("project table rows support keyboard selection without disabling the row", async () => {
  const table = await source("ProjectTable.tsx");

  assert.match(table, /tabIndex=\{onSelect \? 0 : -1\}/);
  assert.match(table, /aria-selected=\{selected\}/);
  assert.match(table, /onKeyDown=\{\(event\) =>/);
  assert.match(table, /event\.key === \"Enter\"/);
  assert.match(table, /event\.key === \" \"/);
  assert.match(table, /onSelect\?\.\(project\)/);
});

test("static HubList rows are presentational instead of disabled pseudo-actions", async () => {
  const list = await source("HubList.tsx");

  assert.match(list, /component=\{hasSelectHandler \? \"button\" : \"div\"\}/);
  assert.match(list, /const itemDisabled = hasSelectHandler \? item\.disabled : false/);
  assert.match(list, /disabled=\{itemDisabled\}/);
  assert.match(list, /tabIndex=\{hasSelectHandler \? 0 : -1\}/);
  assert.doesNotMatch(list, /const itemDisabled = item\.disabled \|\| !hasSelectHandler/);
});
