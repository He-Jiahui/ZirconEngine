import { animationBlueprints } from "./blueprints/animation.js";
import { dataBlueprints } from "./blueprints/data.js";
import { diagnosticsBlueprints } from "./blueprints/diagnostics.js";
import { gameplayBlueprints } from "./blueprints/gameplay.js";
import { multiplayerBlueprints } from "./blueprints/multiplayer.js";
import { productionBlueprints } from "./blueprints/production.js";
import { renderingBlueprints } from "./blueprints/rendering.js";
import { simulationBlueprints } from "./blueprints/simulation.js";
import { uiBlueprints } from "./blueprints/ui.js";
import { worldBlueprints } from "./blueprints/world.js";

export const extensionBlueprints = {
  ...animationBlueprints,
  ...dataBlueprints,
  ...diagnosticsBlueprints,
  ...gameplayBlueprints,
  ...multiplayerBlueprints,
  ...productionBlueprints,
  ...renderingBlueprints,
  ...simulationBlueprints,
  ...uiBlueprints,
  ...worldBlueprints
};
