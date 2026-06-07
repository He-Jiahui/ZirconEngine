import { assetBottom } from "../core-module-bottoms.js";
import { assetCenter } from "../core-module-centers.js";
import { assetDetails } from "../core-module-details.js";
import { assetLeft } from "../core-module-lefts.js";
import { bottomOutput } from "../../shared/module-components.js";

export const assetBrowserCoreModule = {
  id: "asset-browser",
  label: "Asset Browser",
  shortLabel: "Assets",
  icon: "image",
  status: "SM_Tree_Oak_01 selected",
  actions: [
    ["save", "Save All"],
    ["folder", "Import"],
    ["history", "Reimport"],
    ["check", "Validate"],
    ["cube", "Build"]
  ],
  left: () => assetLeft(),
  center: () => assetCenter(),
  right: () => assetDetails(),
  bottom: () => bottomOutput("asset-browser", ["Queue", "Output", "Validation", "Cook", "Package"], assetBottom())
};

export const assetCoreModules = [assetBrowserCoreModule];
