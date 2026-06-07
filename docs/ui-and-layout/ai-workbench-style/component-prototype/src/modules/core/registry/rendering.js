export { materialCoreModule } from "./rendering/material.js";
export { renderPipelineCoreModule } from "./rendering/render-pipeline.js";
export { vfxCoreModule } from "./rendering/vfx.js";

import { materialCoreModule } from "./rendering/material.js";
import { renderPipelineCoreModule } from "./rendering/render-pipeline.js";
import { vfxCoreModule } from "./rendering/vfx.js";

export const renderingCoreModules = [
  materialCoreModule,
  renderPipelineCoreModule,
  vfxCoreModule
];
