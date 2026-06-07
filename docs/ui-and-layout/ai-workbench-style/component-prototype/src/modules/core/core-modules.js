import { aiPerceptionCoreModule, behaviorTreeCoreModule } from "./registry/ai.js";
import { assetCoreModules } from "./registry/assets.js";
import { gameplayCoreModules } from "./registry/gameplay.js";
import { indexCoreModules } from "./registry/index.js";
import { hudCoreModules } from "./registry/ui.js";
import { materialCoreModule, renderPipelineCoreModule, vfxCoreModule } from "./registry/rendering.js";

export const coreModules = [
  ...indexCoreModules,
  ...gameplayCoreModules,
  aiPerceptionCoreModule,
  materialCoreModule,
  behaviorTreeCoreModule,
  renderPipelineCoreModule,
  ...assetCoreModules,
  vfxCoreModule,
  ...hudCoreModules
];
