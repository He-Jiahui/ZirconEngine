import { actionStack, listRows, moduleTree, panel } from "../../shared/module-components.js";

export function assetLeft() {
  return [
    panel("Filters", `${listRows(["All Assets", "Recently Modified", "Checked Out", "Missing References", "Validation Issues"], 0, ["12,347", "142", "8", "23", "19"])}${actionStack(["Import Assets", "Reimport Assets", "Import From Path"])}`),
    panel("Folder Tree", moduleTree([
      ["Nightingale", "folder", false, 0],
      ["Content", "folder", false, 1],
      ["Characters", "folder", false, 2],
      ["Environments", "folder", false, 2],
      ["Forest", "folder", true, 3],
      ["Materials", "folder", false, 2],
      ["VFX", "folder", false, 2]
    ]))
  ];
}
