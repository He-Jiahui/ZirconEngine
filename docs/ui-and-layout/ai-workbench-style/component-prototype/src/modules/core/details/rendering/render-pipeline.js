import { checkbox, searchInput, select } from "../../../../components/inputs/atoms.js";
import { actionButton, compactStats, moduleTable, moduleTree, panelGroup, settingsRows, tag } from "../../../shared/module-components.js";
import { coreRightRouteOptions } from "../routes.js";

export function renderPipelineDetails() {
  return panelGroup("render-right", [
    { label: "Passes", active: true, content: `${searchInput("Search...")}${moduleTree([
      ["Frame 1234", "renderer", true, 0],
      ["Setup", "folder", false, 1],
      ["GBuffer", "folder", false, 1],
      ["1 GBuffer Pass", "renderer", false, 2],
      ["Lighting", "folder", false, 1],
      ["2 Lighting Pass", "sun", false, 2],
      ["3 SSR Pass", "renderer", false, 2],
      ["5 Post Process Pass", "renderer", true, 2],
      ["Output", "folder", false, 1],
      ["7 UI Composite Pass", "image", false, 2]
    ], coreRightRouteOptions("render-right:passes"))}${settingsRows([
      ["Pass", tag("Post Process Pass (#5)", "purple")],
      ["Pass Type", select("Render Pass")],
      ["Enabled", checkbox("", true)],
      ["SceneColor", select("R11G11B10_FLOAT")],
      ["AO", select("R8_UNORM")],
      ["PostColor", select("R11G11B10_FLOAT")]
    ])}` },
    { label: "Resources", content: moduleTable(["Resource", "Format", "State"], [
      { cells: ["SceneColor", "R11G11B10_FLOAT", tag("Read", "cyan")] },
      { cells: ["PostColor", "R11G11B10_FLOAT", tag("Write", "orange")], selected: true },
      { cells: ["Depth", "D32_FLOAT", tag("Read", "cyan")] }
    ], "1fr 1.2fr 0.8fr", coreRightRouteOptions("render-right:resources")) },
    { label: "Frame Stages", content: `${compactStats([["GPU", "0.45 ms"], ["CPU", "0.08 ms"], ["Draws", "42"], ["Bandwidth", "1.28 GB"]])}${actionButton("View in Profiler", "target", coreRightRouteOptions("render-right:frame-stages"))}` }
  ]);
}
