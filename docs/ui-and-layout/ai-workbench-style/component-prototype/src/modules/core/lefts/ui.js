import { searchInput } from "../../../components/inputs/atoms.js";
import { listRows, moduleTree, panel, panelGroup, segmentButtons } from "../../shared/module-components.js";

export function hudLeft() {
  return [
    panel("Widget Palette", `${searchInput("Search widgets...")}${listRows(["Text", "Image", "Button", "Progress Bar", "Slider", "Icon", "Container", "System"], 3)}`),
    panel("Responsive Presets", segmentButtons(["Phone", "Tablet", "Desktop", "Console"], 0)),
    panel("UI Assets", panelGroup("hud-assets", [
      { label: "UI Assets", active: true, content: moduleTree([
        ["HUD", "folder", false, 0],
        ["Gameplay_HUD", "image", true, 1],
        ["Vehicle_HUD", "image", false, 1],
        ["Widget Blueprints", "folder", false, 0],
        ["WBP_HealthBar", "component", false, 1],
        ["WBP_AmmoCounter", "component", false, 1],
        ["Style Resources", "folder", false, 0],
        ["Colors", "material", false, 1]
      ]) },
      { label: "Screens", content: moduleTree([
        ["HUD", "folder", true, 0],
        ["Gameplay_HUD", "image", true, 1],
        ["Vehicle_HUD", "image", false, 1]
      ]) }
    ], { className: "is-card-panel" }))
  ];
}
