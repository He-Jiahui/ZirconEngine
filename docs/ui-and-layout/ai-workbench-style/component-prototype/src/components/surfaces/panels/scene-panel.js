import { icon } from "../../../foundation/icons.js";
import { cluster, grid } from "../../../foundation/layout.js";
import { sceneTree } from "../../../foundation/data.js";
import { panelGroup } from "../../../modules/shared/module-components.js";
import { iconButton, searchInput } from "../../inputs/atoms.js";
import { treeView } from "../../data/collections.js";
import { drawerSurface } from "./drawer-surface.js";

export function scenePanel() {
  const sceneActions = cluster({ as: "span", className: "zr-topbar-group", gap: "sm", children: [iconButton("filter", "Filter"), iconButton("plus", "Add")] });
  return drawerSurface({
    className: "zr-scene-panel",
    host: "scene",
    children: [
      panelGroup("scene", [
        { label: "Scene", active: true, content: `${grid({ className: "zr-panel-toolbar", children: [searchInput("Search..."), sceneActions] })}${treeView(sceneTree)}` },
        { label: "Layers", content: layersView() }
      ])
    ]
  });
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
