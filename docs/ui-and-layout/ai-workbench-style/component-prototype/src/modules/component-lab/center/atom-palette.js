import { cluster, grid, stack } from "../../../foundation/layout.js";
import {
  button,
  checkbox,
  iconButton,
  input,
  numberField,
  radio,
  rangeSlider,
  select,
  slider,
  tabs,
  toggle
} from "../../../components/inputs/atoms.js";
import { actionButton } from "../../shared/module-components.js";
import { componentLabRouteOptions } from "../routes.js";
import { labColumn } from "./lab-column.js";

export function atomPalette() {
  return `${grid({ className: "zr-showcase-grid", gap: "md", children: [
    labColumn("Buttons", grid({ className: "zr-control-grid", columns: 2, children: [
      button("Primary", { kind: "primary" }),
      button("Secondary", { kind: "secondary" }),
      button("Tertiary", { kind: "tertiary" }),
      button("Outline", { kind: "outline" }),
      button("Icon", { icon: "plus" }),
      button("", { icon: "trash", kind: "danger" }),
      actionButton("Route Audit", "target", componentLabRouteOptions("module-bottom-component-lab:routes", "workbench.component_lab.atom")),
      actionButton("Input Audit", "check", componentLabRouteOptions("component-lab-right:inputs", "workbench.component_lab.atom"))
    ] })),
    labColumn("Icon Buttons", cluster({ className: "zr-icon-grid", wrap: true, children: [
      iconButton("plus", "Add", { large: true }),
      iconButton("folder", "Folder", { large: true }),
      iconButton("save", "Save", { large: true }),
      iconButton("trash", "Delete", { large: true, danger: true }),
      iconButton("eye", "Visible", { large: true }),
      iconButton("eyeOff", "Hidden", { large: true }),
      iconButton("lock", "Locked", { large: true }),
      iconButton("more", "More", { large: true })
    ] })),
    labColumn("Fields", stack({ className: "zr-field-stack", gap: "sm", children: [
      input("Text field"),
      input("", { value: "Focused input", focused: true }),
      input("Disabled input", { disabled: true }),
      cluster({ className: "zr-topbar-group", gap: "sm", children: [select("Dropdown"), numberField("42", { stepper: true })] })
    ] })),
    labColumn("Selection", stack({ className: "zr-check-stack", gap: "sm", children: [
      checkbox("Checkbox", true),
      checkbox("Checkbox", false),
      radio("Radio option", true),
      radio("Radio option", false),
      toggle("Toggle", true)
    ] })),
    labColumn("Sliders", [slider("Value", 58, "0.75"), rangeSlider("Range", 28, 78, "0.20", "0.80"), slider("Steps", 86, "3", true)]),
    labColumn("Tabs", [tabs(["Tab 1", "Tab 2", "Tab 3"], 0), tabs(["Left", "Center", "Right"], 1, "zr-segment")])
  ] })}`;
}
