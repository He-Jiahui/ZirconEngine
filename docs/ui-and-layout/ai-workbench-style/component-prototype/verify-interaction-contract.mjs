import { readFileSync } from "node:fs";
import { inspector, popups, rail, scenePanel, showcase, statusbar, topbar, viewport, workbenchWindow } from "./surfaces.js";

const appHtml = workbenchWindow([topbar(), rail(), scenePanel(), viewport(), inspector(), showcase(), statusbar(), popups()]);
const appSource = readFileSync(new URL("./app.js", import.meta.url), "utf8");
const indexSource = readFileSync(new URL("./index.html", import.meta.url), "utf8");

const checks = [
  ["tokens layer loaded first", indexSource.indexOf("tokens.css") < indexSource.indexOf("atoms.css")],
  ["atoms layer before collections", indexSource.indexOf("atoms.css") < indexSource.indexOf("collections.css")],
  ["collections layer before surfaces", indexSource.indexOf("collections.css") < indexSource.indexOf("surfaces.css")],
  ["surfaces layer before workbench", indexSource.indexOf("surfaces.css") < indexSource.indexOf("workbench.css")],
  ["button atom", appHtml.includes("zr-button")],
  ["input atom", appHtml.includes("zr-input")],
  ["checkbox atom", appHtml.includes("zr-checkbox")],
  ["toggle atom", appHtml.includes("zr-switch")],
  ["icon button atom", appHtml.includes("zr-icon-button")],
  ["tabs atom", appHtml.includes("zr-tabs")],
  ["dropdown atom", appHtml.includes("zr-select") && appHtml.includes("data-dropdown")],
  ["list collection", appHtml.includes("zr-list")],
  ["tree view collection", appHtml.includes("zr-tree")],
  ["table view collection", appHtml.includes("zr-table")],
  ["popup collection", appHtml.includes("zr-popup-layer")],
  ["scene tab target", appHtml.includes('data-panel-tab="scene:layers"')],
  ["inspector tab target", appHtml.includes('data-panel-tab="inspector:history"')],
  ["showcase tab target", appHtml.includes('data-panel-tab="showcase:console"')],
  ["window surface", appHtml.includes('data-surface="window"')],
  ["drawer surfaces", (appHtml.match(/data-surface="drawer"/g) ?? []).length >= 2],
  ["inspector window surface", appHtml.includes('class="zr-panel zr-inspector" data-surface="window"')],
  ["panel view surfaces", (appHtml.match(/data-surface="panel-view"/g) ?? []).length >= 6],
  ["tab aria selected", appHtml.includes('role="tab" aria-selected="true"')],
  ["dropdown popup layer", appHtml.includes('id="popup-layer"')],
  ["toggle handler", appSource.includes('dataset.toggle === "switch"')],
  ["radio handler", appSource.includes("[data-radio]")],
  ["panel view handler", appSource.includes("dataset.panelView") && appSource.includes("panelTarget")],
  ["tree selection handler", appSource.includes("[data-tree-row]")],
  ["list/table selection handler", appSource.includes(".zr-list-item:not(.is-disabled), .zr-table-row:not(.zr-table-head)")],
  ["tool active handler", appSource.includes(".zr-topbar-tools .zr-icon-button")],
  ["dropdown placement handler", appSource.includes("getBoundingClientRect") && appSource.includes("popup.style.left")],
  ["aria selected update", appSource.includes('setAttribute("aria-selected", "true")')],
  ["no full workbench screenshot embed", !appHtml.includes("workbench.png")],
  ["viewport raster is isolated", appHtml.includes("workbench-viewport-reference.png")]
];

const failed = checks.filter(([, passed]) => !passed);

for (const [name, passed] of checks) {
  console.log(`${passed ? "ok" : "fail"} ${name}`);
}

if (failed.length > 0) {
  console.error(`Interaction contract failed: ${failed.map(([name]) => name).join(", ")}`);
  process.exit(1);
}
