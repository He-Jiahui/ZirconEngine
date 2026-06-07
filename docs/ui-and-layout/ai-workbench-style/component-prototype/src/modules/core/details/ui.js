import { checkbox, input, searchInput, select, slider } from "../../../components/inputs/atoms.js";
import { moduleTable, moduleTree, panelGroup, settingsRows, tag } from "../../shared/module-components.js";
import { coreRightRouteOptions } from "./routes.js";

export function hudDetails() {
  return panelGroup("hud-right", [
    { label: "Widget Hierarchy", active: true, content: `${searchInput("Search hierarchy...")}${moduleTree([
      ["Gameplay_HUD (Screen)", "image", true, 0],
      ["Canvas Panel", "columns", false, 1],
      ["TopBar", "folder", false, 1],
      ["TeamScore_Left", "component", false, 2],
      ["MatchTimer", "component", false, 2],
      ["Minimap", "image", false, 1],
      ["WeaponPanel", "component", true, 1],
      ["Weapon_Icon", "image", false, 2],
      ["Ammo_Clip", "component", false, 2],
      ["AbilityBar", "folder", false, 1]
    ], coreRightRouteOptions("hud-right:widget-hierarchy"))}` },
    { label: "Inspector", content: `${settingsRows([
      ["Widget", tag("WeaponPanel", "cyan")],
      ["Is Variable", checkbox("", true)],
      ["Visible", checkbox("", true)],
      ["Opacity", select("100%")],
      ["Render Layer", input("", { value: "0" })],
      ["Tooltip", input("Enter text...")]
    ])}${slider("Scale", 62, "1.00")}` },
    { label: "Bindings", content: moduleTable(["Property", "Binding", "Status"], [
      { cells: ["Ammo_Clip", "GetCurrentAmmo", tag("OK", "green")] },
      { cells: ["Ammo_Reserve", "GetReserveAmmo", tag("Missing", "orange")], selected: true },
      { cells: ["HealthBar", "GetHealthRatio", tag("OK", "green")] }
    ], "1fr 1.3fr 88px", coreRightRouteOptions("hud-right:bindings")) }
  ]);
}
