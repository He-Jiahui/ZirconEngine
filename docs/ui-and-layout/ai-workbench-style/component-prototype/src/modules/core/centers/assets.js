import { cluster } from "../../../foundation/layout.js";
import { checkbox, searchInput, select } from "../../../components/inputs/atoms.js";
import { actionButton, moduleTable, panel, tag } from "../../shared/module-components.js";

export function assetCenter() {
  return `<div class="zr-module-editor-grid is-assets">
    ${panel("Content / Environments / Forest", `${cluster({ className: "zr-module-filterbar", children: [select("Type: All"), select("Status: All"), select("Tags: All"), actionButton("Add Filter", "plus"), searchInput("Search Assets")] })}${moduleTable(["", "Name", "Type", "Tags", "Size", "Status", "Modified"], [
      { cells: [checkbox("", false), "Foliage", "Folder", "-", "-", "-", "2026-05-19"] },
      { cells: [checkbox("", true), "SM_Tree_Oak_01", "Static Mesh", `${tag("Nature", "green")} ${tag("Tree", "green")}`, "1.24 MB", tag("Valid", "green"), "2026-05-18 14:32"], selected: true },
      { cells: [checkbox("", false), "SM_Rock_Cliff_01", "Static Mesh", `${tag("Rock", "purple")} ${tag("Cliff", "purple")}`, "2.15 MB", tag("Valid", "green"), "2026-05-18 14:34"] },
      { cells: [checkbox("", false), "T_Forest_Ground_01", "Texture 2D", tag("Ground", "orange"), "4.10 MB", tag("Valid", "green"), "2026-05-18 14:20"] }
    ], "36px 1.4fr 1fr 1.2fr 90px 90px 150px")}`)}
  </div>`;
}
