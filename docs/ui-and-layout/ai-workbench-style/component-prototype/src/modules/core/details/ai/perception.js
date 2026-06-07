import { checkbox, input, slider, toggle } from "../../../../components/inputs/atoms.js";
import { actionButton, listRows, moduleTable, moduleTree, panelGroup, settingsRows, tag } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function perceptionDetails() {
  return panelGroup("perception-right", [
    { label: "World Overview", active: true, content: `${listRows(["AI_Guard_01", "AI_Guard_02", "AI_Guard_03"], 0, ["Sight", "Sight", "Hearing"], coreRightRouteOptions("perception-right:world-overview"))}${moduleTable(["Time", "Actor", "Sense"], [
      { cells: ["00:12.345", "Noise_Maker_BP", tag("Hearing", "purple")] },
      { cells: ["00:13.104", "Enemy_01", tag("Sight", "cyan")], selected: true },
      { cells: ["00:13.590", "Explosion_BP", tag("Hearing", "purple")] },
      { cells: ["00:14.512", "Enemy_01", tag("Sight", "cyan")] }
    ], "82px 1fr 88px", coreRightRouteOptions("perception-right:world-overview"))}` },
    { label: "Sight Details", content: `${settingsRows([
      ["Enabled", toggle("", true)],
      ["Radius", input("", { value: "2000.0" })],
      ["Lose Sight Radius", input("", { value: "2500.0" })],
      ["Age Max", input("", { value: "5.0s" })],
      ["Tick Interval", input("", { value: "0.2s" })],
      ["Detect Enemies", checkbox("", true)],
      ["Detect Neutrals", checkbox("", true)],
      ["Detect Friendlies", checkbox("", false)]
    ])}${slider("Peripheral Angle", 72, "120deg")}` },
    { label: "Filters", content: `${moduleTree([
      ["Target Tags", "folder", false, 0],
      ["Faction.Enemy", "target", true, 1],
      ["Faction.Neutral", "target", false, 1],
      ["Class Filter", "folder", false, 0],
      ["AI_Guard", "component", true, 1]
    ], coreRightRouteOptions("perception-right:filters"))}${actionButton("Add Filter", "plus", coreRightRouteOptions("perception-right:filters"))}` }
  ]);
}
