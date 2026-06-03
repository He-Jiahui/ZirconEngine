import { icon } from "./icons.js";
import { cluster, grid, stack } from "./layout.js";
import { defaultModuleId, moduleRail, moduleTabs, moduleToolbar } from "./modules.js";
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

export function topbar(activeModuleId = defaultModuleId) {
  const left = [iconButton("menu", "Menu"), "divider", iconButton("file", "New"), iconButton("folder", "Open"), iconButton("save", "Save"), "divider", iconButton("undo", "Undo"), iconButton("redo", "Redo")];
  const tools = [moduleTabs(activeModuleId), moduleToolbar(activeModuleId)];
  const right = [iconButton("play", "Play", { large: true }), iconButton("chevronDown", "Play options"), "divider", iconButton("grid", "Layout"), iconButton("sun", "Lighting"), iconButton("more", "More")];
  return `<header class="zr-topbar">${toolbarGroup(left)}${toolbarGroup(tools, "zr-topbar-tools")}${toolbarGroup(right)}</header>`;
}

function renderGroup(items) {
  return items.map((item) => item === "divider" ? '<span class="zr-divider"></span>' : item).join("");
}

function toolbarGroup(items, className = "zr-topbar-group") {
  return cluster({ className, gap: "sm", children: renderGroup(items) });
}

export function rail(activeModuleId = defaultModuleId) {
  return moduleRail(activeModuleId);
}

export function scenePanel() {
  const sceneActions = cluster({ as: "span", className: "zr-topbar-group", gap: "sm", children: [iconButton("filter", "Filter"), iconButton("plus", "Add")] });
  return drawerSurface({
    className: "zr-scene-panel",
    host: "scene",
    children: [
      panelTabs(["Scene", "Layers"], 0, "scene"),
      panelView("scene", "scene", true, `${grid({ className: "zr-panel-toolbar", children: [searchInput("Search..."), sceneActions] })}${treeView(sceneTree)}`),
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
  const gridLines = [
    ...[0, 1, 2, 3, 4, 5].map((line) => `<span class="zr-viewport-grid-line is-horizontal ${line === 2 || line === 4 ? "is-major" : ""}" style="--line:${line}"></span>`),
    ...[0, 1, 2, 3, 4, 5, 6].map((line) => `<span class="zr-viewport-grid-line is-vertical ${line === 2 || line === 5 ? "is-major" : ""}" style="--line:${line}"></span>`),
  ].join("");
  return `<section class="zr-viewport">
    <div class="zr-scene-shell">
      <div class="zr-scene-ceiling"><span class="zr-scene-light l1 is-soft"></span><span class="zr-scene-light l2"></span><span class="zr-scene-light l3"></span><span class="zr-scene-light l4"></span></div>
      <div class="zr-scene-wall"><span class="zr-scene-wall-detail center-lines"></span><div class="zr-scene-door"><span></span></div><span class="zr-scene-wall-panel p1"></span><span class="zr-scene-wall-panel p2"></span><span class="zr-scene-wall-panel p3"></span><span class="zr-scene-column c-left"></span><span class="zr-scene-column c-right"></span><span class="zr-scene-beacon b1"></span><span class="zr-scene-beacon b2"></span></div>
      <span class="zr-scene-lightwash left"></span>
      <span class="zr-scene-lightwash center"></span>
      <span class="zr-scene-shadow top-bay"></span>
      <span class="zr-scene-shadow ceiling-left"></span>
      <span class="zr-scene-shadow ceiling-mid"></span>
      <span class="zr-scene-lightwash wall-right"></span>
      <span class="zr-scene-lightwash rear-walkway"></span>
      <div class="zr-scene-side left"></div>
      <div class="zr-scene-side right"></div>
      <span class="zr-scene-rack left"></span>
      <div class="zr-scene-floor"><span class="zr-floor-reflection"></span><span class="zr-floor-grate right"></span><span class="zr-floor-panel fp1"></span><span class="zr-floor-panel fp2"></span><span class="zr-floor-panel fp3"></span><span class="zr-floor-seam seam-right"></span>${gridLines}</div>
      <span class="zr-scene-lightwash lower"></span>
      <span class="zr-scene-lightwash floor"></span>
      <span class="zr-scene-lightwash floor-cool"></span>
      <span class="zr-scene-lightwash floor-right"></span>
      <span class="zr-scene-shadow left-floor"></span>
      <span class="zr-scene-shadow right-floor"></span>
      <div class="zr-scene-handrail left"></div>
      <div class="zr-scene-handrail right"></div>
      <div class="zr-scene-cargo c1"><span class="zr-cargo-inner"></span></div>
      <div class="zr-scene-cargo c2"><span class="zr-cargo-inner"></span></div>
      <div class="zr-scene-cargo c3"><span class="zr-cargo-inner"></span></div>
      <div class="zr-scene-cargo c4"><span class="zr-cargo-inner"></span></div>
      <div class="zr-crate"><span class="zr-crate-top"></span><span class="zr-selection-edge top"></span><span class="zr-selection-edge right"></span><span class="zr-selection-edge bottom"></span><span class="zr-selection-edge left"></span><span class="zr-transform-origin"></span><span class="zr-transform-axis axis-x"></span><span class="zr-transform-axis axis-y"></span><span class="zr-transform-axis axis-z"></span><span class="zr-transform-label label-x">X</span><span class="zr-transform-label label-y">Y</span></div>
      <span class="zr-axis-mini left"></span>
      <div class="zr-orientation-gizmo"><span class="axis y">Y</span><span class="axis z">Z</span><span class="axis x">X</span><span class="center"></span></div>
      <span class="zr-scene-vignette"></span>
    </div>
    <div class="zr-viewport-tools">${cluster({ as: "span", className: "zr-viewport-cluster", children: [select("Perspective"), select("Lit", { icon: "sun" })] })}${cluster({ as: "span", className: "zr-viewport-cluster", children: [iconButton("target", "Target"), iconButton("grid", "Snap", { active: true }), iconButton("snap", "Snap"), iconButton("snap", "Magnet"), iconButton("folder", "Local"), select("10°"), select("0.25"), iconButton("scale", "Fullscreen")] })}</div>
  </section>`;
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
  const vectors = sectionData.fields?.map(vectorRow).join("") ?? "";
  const resources = sectionData.rows?.map((row) => `<div class="zr-resource-row ${row.count ? "has-count" : "is-single-resource"}"><span>${row.label}</span><span>${row.count ?? ""}</span>${select(row.value, row.swatch ? { swatch: true } : { icon: row.icon })}</div>`).join("") ?? "";
  const nested = sectionData.nested?.map(([label, value]) => {
    const isDisclosure = value === "";
    const content = isDisclosure ? `${icon("chevronDown")}<span>${label}</span>` : label;
    const control = value === "check" ? checkbox("", true) : value ? select(value) : "<span></span>";
    return `<div class="zr-resource-row is-nested-resource ${isDisclosure ? "is-disclosure-row" : ""}"><span>${content}</span><span></span>${control}</div>`;
  }).join("") ?? "";
  return `<section class="zr-section is-${sectionClass}"><div class="zr-section-title">${icon(sectionData.icon)}<span>${sectionData.title}</span>${checkbox("", sectionData.checked)}${icon("chevronUp")}</div>${vectors}${resources}${nested}</section>`;
}

function vectorRow(row) {
  if (!row.link) {
    return `<div class="zr-vector-row"><span>${row.label}</span><span>X</span>${row.values.map((value, index) => `${index > 0 ? `<span>${["Y", "Z"][index - 1]}</span>` : ""}<span class="zr-value-box">${value}</span>`).join("")}</div>`;
  }

  const axes = ["X", "Y", "Z"];
  const cells = row.values.map((value, index) => {
    if (index === 0) {
      return `<span class="zr-linked-axis">${icon("link")}<span class="zr-axis-x">${axes[index]}</span></span><span class="zr-value-box">${value}</span>`;
    }
    return `<span>${axes[index]}</span><span class="zr-value-box">${value}</span>`;
  }).join("");
  return `<div class="zr-vector-row has-linked-axis"><span>${row.label}</span>${cells}</div>`;
}

export function showcase() {
  const componentsView = `${grid({ className: "zr-showcase-grid", gap: "none", children: [
    showcaseColumn("Buttons", grid({ className: "zr-control-grid", columns: 2, children: [button("Primary", { kind: "primary" }), button("Secondary", { kind: "secondary" }), button("Tertiary", { kind: "tertiary" }), button("Outline", { kind: "outline" }), button("Icon", { icon: "plus" }), button("", { icon: "trash", kind: "danger" }), button("Disabled", { disabled: true }), select("Dropdown")] })),
    showcaseColumn("Icon Buttons", [cluster({ className: "zr-icon-grid", wrap: true, children: [iconButton("plus", "Add", { large: true }), iconButton("folder", "Folder", { large: true }), iconButton("save", "Save", { large: true }), iconButton("trash", "Delete", { large: true, danger: true }), iconButton("eye", "Visible", { large: true }), iconButton("eyeOff", "Hidden", { large: true }), iconButton("lock", "Locked", { large: true }), iconButton("more", "More", { large: true })] }), colTitle("Toggle Buttons"), tabs([{ icon: "grid" }, { icon: "list" }, { icon: "columns" }], 0, "zr-segment")]),
    showcaseColumn("Inputs", stack({ className: "zr-field-stack", gap: "sm", children: [input("Text field"), input("", { value: "Focused input", focused: true }), input("Disabled input", { disabled: true }), cluster({ className: "zr-topbar-group", gap: "sm", children: [select("Dropdown"), numberField("42", { stepper: true })] })] })),
    showcaseColumn("Checkboxes & Radios", stack({ className: "zr-check-stack", gap: "sm", children: [checkbox("Checkbox", true), checkbox("Checkbox", false), radio("Radio option", true), radio("Radio option", false)] })),
    showcaseColumn("Sliders", [slider("Value", 58, "0.75"), rangeSlider("Range", 28, 78, "0.20", "0.80"), slider("Steps", 86, "3", true)]),
    showcaseColumn("Labs", [tabs(["Tab 1", "Tab 2", "Tab 3"], 0), colTitle("Segmented Control"), tabs(["Left", "Center", "Right"], 1, "zr-segment"), colTitle("Switch"), toggle("", true)])
  ] })}${stack({ className: "zr-side-stack", gap: "md", children: [
    `<div class="zr-side-list">${colTitle("List")}${listView(listItems)}</div>`,
    `<div class="zr-side-menu">${colTitle("Menu")}${menu(menuItems)}</div>`
  ] })}${grid({ className: "zr-lower-demo", gap: "md", children: [
    `<div>${colTitle("Table")}${tableView(tableRows)}</div>`,
    alerts(alertData),
    tooltip(),
    toast()
  ] })}`;

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

export function statusbar(statusText = "Ready") {
  const left = cluster({ className: "zr-status-left", gap: "lg", children: [`<span class="zr-status-item"><span class="zr-dot"></span><span class="zr-module-status-message" data-status-message>${statusText}</span></span>`, `<span class="zr-status-item">${icon("check")}No Errors</span>`, `<span class="zr-status-item">${icon("warning")}2 Warnings</span>`, `<span class="zr-status-item">${icon("info")}0 Messages</span>`] });
  const right = cluster({ className: "zr-status-right", gap: "lg", children: [select("Grid: 10 cm"), select("Snap: On"), iconButton("snap", "Snap"), iconButton("globe", "World"), iconButton("target", "Target"), select("100%")] });
  return `<footer class="zr-statusbar">${left}<span></span>${right}</footer>`;
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
  const layerActions = cluster({ as: "span", className: "zr-topbar-group", gap: "sm", children: [iconButton("eye", "Visibility"), iconButton("lock", "Lock")] });
  return `<div class="zr-alt-panel">${grid({ className: "zr-panel-toolbar", children: [searchInput("Filter layers..."), layerActions] })}<div class="zr-layer-list">${layers.map(([name, state, on]) => `<button class="zr-layer-row ${on ? "is-active" : ""}" type="button">${icon(on ? "eye" : "eyeOff")}<span>${name}</span><small>${state}</small></button>`).join("")}</div></div>`;
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

function colTitle(label) {
  return `<h3 class="zr-col-title">${label}</h3>`;
}

function showcaseColumn(title, children) {
  return stack({ className: "zr-showcase-col", gap: "sm", children: [colTitle(title), children] });
}
