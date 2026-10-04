import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const sourceRoot = new URL("../src/", import.meta.url);

async function source(path) {
  return readFile(new URL(path, sourceRoot), "utf8");
}

test("project-row menu trigger exposes a stable menu relationship", async () => {
  const table = await source("components/data/ProjectTable.tsx");

  assert.match(table, /menuId\?: string/);
  assert.match(table, /menuTriggerPrefix\?: string/);
  assert.match(table, /menuOpenTriggerId\?: string \| null/);
  assert.match(table, /const menuTriggerId =/);
  assert.match(table, /aria-haspopup=\{onRowMenu \? "menu" : undefined\}/);
  assert.match(table, /aria-expanded=\{onRowMenu \? menuOpenTriggerId === menuTriggerId : undefined\}/);
  assert.match(table, /aria-controls=\{onRowMenu \? menuId : undefined\}/);
  assert.match(table, /menuLabel\?: string/);
});

test("HubMenu gives the MUI menu list the id and accessible name used by its trigger", async () => {
  const menu = await source("components/overlays/HubMenu.tsx");

  assert.match(menu, /id\?: string/);
  assert.match(menu, /ariaLabel\?: string/);
  assert.match(menu, /list: \{ dense: true, id, "aria-label": ariaLabel \}/);
});

test("dashboard binds both project tables to one open menu id and uses the management label", async () => {
  const dashboard = await source("pages/ProjectsDashboard.tsx");

  assert.match(dashboard, /const projectRowMenuId = "hub-project-row-menu"/);
  assert.match(dashboard, /const projectBrowserRowMenuTriggerPrefix = "hub-project-browser-row-menu-trigger"/);
  assert.match(dashboard, /const recentProjectRowMenuTriggerPrefix = "hub-recent-project-row-menu-trigger"/);
  assert.match(dashboard, /menuId=\{projectRowMenuId\}/g);
  assert.match(dashboard, /menuTriggerPrefix=\{projectBrowserRowMenuTriggerPrefix\}/);
  assert.match(dashboard, /menuTriggerPrefix=\{recentProjectRowMenuTriggerPrefix\}/);
  assert.match(dashboard, /menuOpenTriggerId=\{rowMenu\?\.triggerId \|\| null\}/);
  assert.match(dashboard, /onRowMenu=\{\(project, anchor, triggerId\) => setRowMenu\(\{ anchor, project, triggerId \}\)\}/);
  assert.match(dashboard, /menuLabel: text\.projectManagement/);
  assert.match(dashboard, /id=\{projectRowMenuId\}/);
  assert.match(dashboard, /ariaLabel=\{`\$\{text\.projectManagement\}: \$\{rowMenu\.project\.name\}`\}/);
});
