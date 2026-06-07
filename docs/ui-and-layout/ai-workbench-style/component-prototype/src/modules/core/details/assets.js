import { alerts } from "../../../components/data/collections.js";
import { moduleTree, panelGroup, previewTile, settingsRows, tag } from "../../shared/module-components.js";
import { coreRightRouteOptions } from "./routes.js";

export function assetDetails() {
  return panelGroup("asset-right", [
    { label: "References", active: true, content: `${moduleTree([
      ["SM_Tree_Oak_01", "cube", true, 0],
      ["Referenced By (5)", "folder", false, 1],
      ["BP_Tree_Oak", "component", false, 2],
      ["Foliage_Oak_Set", "grid", false, 2],
      ["Level_Forest", "globe", false, 2],
      ["Depends On (12)", "folder", false, 1]
    ], coreRightRouteOptions("asset-right:references"))}` },
    { label: "Metadata", content: `${settingsRows([
      ["Name", "SM_Tree_Oak_01"],
      ["Type", "Static Mesh"],
      ["Path", "/Game/Environments/Forest"],
      ["Size", "1.24 MB"],
      ["Status", tag("Valid", "green")],
      ["Nanite", tag("Enabled", "green")]
    ])}${previewTile("asset")}` },
    { label: "Preview", content: previewTile("asset") },
    { label: "Issues", content: alerts([["warning", "1 warning"], ["error", "1 invalid collision"]]) }
  ]);
}
