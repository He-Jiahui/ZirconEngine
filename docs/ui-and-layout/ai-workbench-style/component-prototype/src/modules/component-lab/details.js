import { panelGroup, settingsRows, tag } from "../shared/module-components.js";
import { componentCoverage, layoutCoverage } from "./data.js";

export function componentLabDetails() {
  return panelGroup("component-lab-right", [
    { label: "Inputs", active: true, content: inputsView() },
    { label: "Data Display", content: coverageView("Collection") },
    { label: "Feedback", content: coverageView("Feedback") },
    { label: "Overlays", content: coverageView("Overlay") },
    { label: "Surfaces", content: coverageView("Surface") },
    { label: "Layout", content: layoutView() },
    { label: "Native Handoff", content: nativeHandoffView() }
  ], { className: "is-module-right" });
}

function inputsView() {
  return settingsRows(componentCoverage.slice(0, 6).map(([family, path, coverage]) => [
    family,
    `${path} ${tag(coverage, "cyan")}`
  ]));
}

function coverageView(layer) {
  const rows = componentCoverage
    .filter(([, , , rowLayer]) => rowLayer === layer)
    .map(([family, path, coverage]) => [family, `${path} ${tag(coverage, "green")}`]);
  return settingsRows(rows.length ? rows : [["Coverage", tag("Tracked by component contract", "green")]]);
}

function layoutView() {
  return settingsRows(layoutCoverage.map(([name, role, alignment]) => [
    name,
    `${role} ${tag(alignment, "blue")}`
  ]));
}

function nativeHandoffView() {
  return settingsRows([
    ["Native scope", tag("component families only", "cyan")],
    ["Web scope", tag("component-lab is prototype-only", "blue")],
    ["Action IDs", tag("dotted functional paths", "green")],
    ["Layout model", tag("Taffy-ready flex/grid grammar", "green")],
    ["Promotion gate", tag("verify-native-component-contract", "orange")]
  ]);
}
