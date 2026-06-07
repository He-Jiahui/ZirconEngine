import { icon } from "../../../foundation/icons.js";
import { cluster, grid, stack } from "../../../foundation/layout.js";
import { alerts as alertData, listItems, menuItems, tableRows } from "../../../foundation/data.js";
import { panelGroup } from "../../../modules/shared/module-components.js";
import { button, checkbox, iconButton, input, numberField, radio, rangeSlider, select, slider, tabs, toggle } from "../../inputs/atoms.js";
import { alerts, listView, menu, tableView, toast, tooltip } from "../../data/collections.js";
import { drawerSurface } from "./drawer-surface.js";

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
      panelGroup("showcase", [
        { label: "UI Components", active: true, content: componentsView },
        { label: "Console", content: consoleView() }
      ])
    ]
  });
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
