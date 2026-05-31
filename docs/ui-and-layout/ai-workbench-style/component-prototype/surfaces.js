import { icon } from "./icons.js";
import { button, checkbox, iconButton, input, numberField, radio, rangeSlider, searchInput, select, slider, tabs, toggle } from "./atoms.js";
import { alerts, listView, menu, tableView, toast, tooltip, treeView } from "./collections.js";
import { alerts as alertData, inspectorSections, listItems, menuItems, sceneTree, tableRows } from "./data.js";

export function workbenchWindow(children) {
  return `<div class="zr-window" data-surface="window">${children.join("")}</div>`;
}

function drawerSurface({ tag = "aside", className, host, kind = "drawer", children }) {
  return `<${tag} class="zr-panel ${className}" data-surface="${kind}" data-panel-host="${host}">${children.join("")}</${tag}>`;
}

function panelView(panel, key, active, content) {
  return `<div class="zr-panel-view ${active ? "is-active" : ""}" data-surface="panel-view" data-panel-view="${panel}:${key}">${content}</div>`;
}

export function topbar() {
  const left = [iconButton("menu", "Menu"), "divider", iconButton("file", "New"), iconButton("folder", "Open"), iconButton("save", "Save"), "divider", iconButton("undo", "Undo"), iconButton("redo", "Redo")];
  const tools = [iconButton("cursor", "Select", { active: true, large: true }), iconButton("move", "Move"), iconButton("rotate", "Rotate"), iconButton("scale", "Scale"), "divider", iconButton("image", "Frame"), iconButton("folder", "Package"), iconButton("grid", "Grid"), select("", { icon: "chevronDown" })];
  const right = [iconButton("play", "Play", { large: true }), iconButton("chevronDown", "Play options"), "divider", iconButton("grid", "Layout"), iconButton("sun", "Lighting"), iconButton("more", "More")];
  return `<header class="zr-topbar"><div class="zr-topbar-group">${renderGroup(left)}</div><div class="zr-topbar-tools">${renderGroup(tools)}</div><div class="zr-topbar-group">${renderGroup(right)}</div></header>`;
}

function renderGroup(items) {
  return items.map((item) => item === "divider" ? '<span class="zr-divider"></span>' : item).join("");
}

export function rail() {
  const items = [
    ["play", "Run", true],
    ["cube", "Scene"],
    ["component", "Graph"],
    ["image", "Assets"],
    ["audio", "Audio"],
    ["code", "Code"],
    ["gear", "Settings"],
    ["help", "Help"]
  ];
  return `<nav class="zr-rail">${items.map(([glyph, label, active], index) => index === 7 ? `<span></span>${iconButton(glyph, label)}` : iconButton(glyph, label, { active })).join("")}</nav>`;
}

export function scenePanel() {
  return drawerSurface({
    className: "zr-scene-panel",
    host: "scene",
    children: [
      panelTabs(["Scene", "Layers"], 0, "scene"),
      panelView("scene", "scene", true, `<div class="zr-panel-toolbar">${searchInput("Search...")}<span class="zr-topbar-group">${iconButton("filter", "Filter")}${iconButton("plus", "Add")}</span></div>${treeView(sceneTree)}`),
      panelView("scene", "layers", false, layersView())
    ]
  });
}

function panelTabs(items, active, panel) {
  return `<div class="zr-panel-tabs">${items.map((item, index) => {
    const key = item.toLowerCase().replace(/\s+/g, "-");
    return `<button class="zr-panel-tab ${index === active ? "is-active" : ""}" type="button" role="tab" aria-selected="${index === active ? "true" : "false"}" data-panel-tab="${panel}:${key}">${item}</button>`;
  }).join("")}</div>`;
}

export function viewport() {
  return `<section class="zr-viewport is-raster"><img class="zr-viewport-reference" alt="" src="assets/workbench-viewport-reference.png" /><div class="zr-scene-wall"></div><div class="zr-scene-art"></div><span class="zr-scene-light l1"></span><span class="zr-scene-light l2"></span><span class="zr-scene-light l3"></span><div class="zr-scene-door"></div><div class="zr-crate"></div><span class="zr-axis-mini left"></span><span class="zr-axis-mini right"></span><div class="zr-viewport-tools"><span class="zr-viewport-cluster">${select("Perspective")}${select("Lit", { icon: "sun" })}</span><span class="zr-viewport-cluster">${iconButton("target", "Target")}${iconButton("grid", "Snap", { active: true })}${iconButton("snap", "Snap")}${iconButton("snap", "Magnet")}${iconButton("folder", "Local")}${select("10°")}${select("0.25")}${iconButton("scale", "Fullscreen")}</span></div></section>`;
}

export function inspector() {
  return drawerSurface({
    className: "zr-inspector",
    host: "inspector",
    kind: "window",
    children: [
      panelTabs(["Inspector", "History"], 0, "inspector"),
      panelView("inspector", "inspector", true, `<div class="zr-inspector-body"><div class="zr-object-header">${icon("cube")}<span>Props</span>${checkbox("Static", false)}${icon("more")}</div><div class="zr-form-row"><span>Tag</span>${select("Untagged")}<span>Layer</span>${select("Default")}</div>${inspectorSections.map(section).join("")}${button("Add Component", { icon: "plus" })}</div>`),
      panelView("inspector", "history", false, historyView())
    ]
  });
}

function section(sectionData) {
  const sectionClass = sectionData.title.toLowerCase().replace(/\s+/g, "-");
  const vectors = sectionData.fields?.map((row) => `<div class="zr-vector-row"><span>${row.label}</span><span>${row.link ? icon("link") : "X"}</span>${row.values.map((value, index) => `${index > 0 ? `<span>${["Y", "Z"][index - 1]}</span>` : ""}<span class="zr-value-box">${value}</span>`).join("")}</div>`).join("") ?? "";
  const resources = sectionData.rows?.map((row) => `<div class="zr-resource-row ${row.count ? "has-count" : "is-single-resource"}"><span>${row.label}</span><span>${row.count ?? ""}</span>${select(row.value, row.swatch ? { swatch: true } : { icon: row.icon })}</div>`).join("") ?? "";
  const nested = sectionData.nested?.map(([label, value]) => `<div class="zr-resource-row is-nested-resource"><span>${label}</span><span></span>${value === "check" ? checkbox("", true) : value ? select(value) : "<span></span>"}</div>`).join("") ?? "";
  return `<section class="zr-section is-${sectionClass}"><div class="zr-section-title">${icon(sectionData.icon)}<span>${sectionData.title}</span>${checkbox("", sectionData.checked)}${icon("chevronUp")}</div>${vectors}${resources}${nested}</section>`;
}

export function showcase() {
  const componentsView = `<div class="zr-showcase-grid">
    <div class="zr-showcase-col"><h3 class="zr-col-title">Buttons</h3><div class="zr-control-grid">${button("Primary", { kind: "primary" })}${button("Secondary", { kind: "secondary" })}${button("Tertiary", { kind: "tertiary" })}${button("Outline", { kind: "outline" })}${button("Icon", { icon: "plus" })}${button("", { icon: "trash", kind: "danger" })}${button("Disabled", { disabled: true })}${select("Dropdown")}</div></div>
    <div class="zr-showcase-col"><h3 class="zr-col-title">Icon Buttons</h3><div class="zr-icon-grid">${iconButton("plus", "Add", { large: true })}${iconButton("folder", "Folder", { large: true })}${iconButton("save", "Save", { large: true })}${iconButton("trash", "Delete", { large: true, danger: true })}${iconButton("eye", "Visible", { large: true })}${iconButton("eyeOff", "Hidden", { large: true })}${iconButton("lock", "Locked", { large: true })}${iconButton("more", "More", { large: true })}</div><h3 class="zr-col-title">Toggle Buttons</h3>${tabs([{ icon: "grid" }, { icon: "list" }, { icon: "columns" }], 0, "zr-segment")}</div>
    <div class="zr-showcase-col"><h3 class="zr-col-title">Inputs</h3><div class="zr-field-stack">${input("Text field")}${input("", { value: "Focused input", focused: true })}${input("Disabled input", { disabled: true })}<div class="zr-topbar-group">${select("Dropdown")}${numberField("42", { stepper: true })}</div></div></div>
    <div class="zr-showcase-col"><h3 class="zr-col-title">Checkboxes & Radios</h3><div class="zr-check-stack">${checkbox("Checkbox", true)}${checkbox("Checkbox", false)}${radio("Radio option", true)}${radio("Radio option", false)}</div></div>
    <div class="zr-showcase-col"><h3 class="zr-col-title">Sliders</h3>${slider("Value", 58, "0.75")}${rangeSlider("Range", 28, 78, "0.20", "0.80")}${slider("Steps", 86, "3", true)}</div>
    <div class="zr-showcase-col"><h3 class="zr-col-title">Labs</h3>${tabs(["Tab 1", "Tab 2", "Tab 3"], 0)}<h3 class="zr-col-title">Segmented Control</h3>${tabs(["Left", "Center", "Right"], 1, "zr-segment")}<h3 class="zr-col-title">Switch</h3>${toggle("", true)}</div>
  </div><div class="zr-side-stack"><div><h3 class="zr-col-title">List</h3>${listView(listItems)}</div><div><h3 class="zr-col-title">Menu</h3>${menu(menuItems)}</div></div><div class="zr-lower-demo"><div><h3 class="zr-col-title">Table</h3>${tableView(tableRows)}</div>${alerts(alertData)}${tooltip()}${toast()}</div>`;

  return drawerSurface({
    tag: "section",
    className: "zr-showcase",
    host: "showcase",
    children: [
      panelTabs(["UI Components", "Console"], 0, "showcase"),
      panelView("showcase", "ui-components", true, componentsView),
      panelView("showcase", "console", false, consoleView())
    ]
  });
}

export function statusbar() {
  return `<footer class="zr-statusbar"><div class="zr-status-left"><span class="zr-status-item"><span class="zr-dot"></span>Ready</span><span class="zr-status-item">${icon("check")}No Errors</span><span class="zr-status-item">${icon("warning")}2 Warnings</span><span class="zr-status-item">${icon("info")}0 Messages</span></div><span></span><div class="zr-status-right">${select("Grid: 10 cm")}${select("Snap: On")}${iconButton("snap", "Snap")}${iconButton("globe", "World")}${iconButton("target", "Target")}${select("100%")}</div></footer>`;
}

export function popups() {
  return `<div id="popup-layer" class="zr-popup-layer">${menu(menuItems)}</div>`;
}

function layersView() {
  const layers = [
    ["Environment", "Visible", true],
    ["Gameplay", "Visible", true],
    ["Audio", "Locked", false],
    ["Debug", "Hidden", false]
  ];
  return `<div class="zr-alt-panel"><div class="zr-panel-toolbar">${searchInput("Filter layers...")}<span class="zr-topbar-group">${iconButton("eye", "Visibility")}${iconButton("lock", "Lock")}</span></div><div class="zr-layer-list">${layers.map(([name, state, on]) => `<button class="zr-layer-row ${on ? "is-active" : ""}" type="button">${icon(on ? "eye" : "eyeOff")}<span>${name}</span><small>${state}</small></button>`).join("")}</div></div>`;
}

function historyView() {
  const entries = ["Selected Props", "Updated material", "Moved Box_01", "Saved scene"];
  return `<div class="zr-inspector-body zr-history-list">${entries.map((entry, index) => `<button class="zr-history-row ${index === 0 ? "is-active" : ""}" type="button">${icon(index === 0 ? "check" : "undo")}<span>${entry}</span><small>${index + 1}m</small></button>`).join("")}</div>`;
}

function consoleView() {
  const rows = [
    ["info", "UI component palette loaded"],
    ["warning", "2 layout warnings"],
    ["check", "No runtime errors"]
  ];
  return `<div class="zr-console-panel">${rows.map(([glyph, text]) => `<div class="zr-console-row">${icon(glyph)}<span>${text}</span></div>`).join("")}</div>`;
}
