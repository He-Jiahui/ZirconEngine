import { renderPipelineBottom } from "../../core-module-bottoms.js";
import { renderPipelineCenter } from "../../core-module-centers.js";
import { renderPipelineDetails } from "../../core-module-details.js";
import { renderPipelineLeft } from "../../core-module-lefts.js";
import { bottomOutput } from "../../../shared/module-components.js";

export const renderPipelineCoreModule = {
  id: "render-pipeline",
  label: "Render Pipeline",
  shortLabel: "Render",
  icon: "renderer",
  status: "Frame 1234 render graph captured",
  actions: [
    ["save", "Save"],
    ["undo", "Undo"],
    ["check", "Compile Pipeline"],
    ["play", "Preview Frame"],
    ["cube", "Build Frame"]
  ],
  left: () => renderPipelineLeft(),
  center: () => renderPipelineCenter(),
  right: () => renderPipelineDetails(),
  bottom: () => bottomOutput("render-pipeline", ["Frame Capture Log", "Compile Output", "Resource Transitions", "Warnings", "Errors", "Compile Log"], renderPipelineBottom())
};
