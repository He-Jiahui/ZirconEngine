import { layoutKindFor } from "./layout-kind.js";
import { animationRecipe } from "./recipes/animation.js";
import { dataRecipe } from "./recipes/data.js";
import { defaultRecipe } from "./recipes/default.js";
import { diagnosticsRecipe } from "./recipes/diagnostics.js";
import { gameplayRecipe } from "./recipes/gameplay.js";
import { onlineRecipe } from "./recipes/online.js";
import { productionRecipe } from "./recipes/production.js";
import { renderingRecipe } from "./recipes/rendering.js";
import { runtimeRecipe } from "./recipes/runtime.js";
import { simulationRecipe } from "./recipes/simulation.js";
import { uiRecipe } from "./recipes/ui.js";
import { vfxRecipe } from "./recipes/vfx.js";
import { worldRecipe } from "./recipes/world.js";

export const recipeByKind = {
  world: worldRecipe,
  rendering: renderingRecipe,
  animation: animationRecipe,
  ui: uiRecipe,
  production: productionRecipe,
  diagnostics: diagnosticsRecipe,
  online: onlineRecipe,
  simulation: simulationRecipe,
  data: dataRecipe,
  gameplay: gameplayRecipe,
  runtime: runtimeRecipe,
  vfx: vfxRecipe,
  default: defaultRecipe
};

export function recipeFor(source, category) {
  const key = source.toLowerCase();
  const kind = layoutKindFor(key, category);
  return { kind, ...recipeByKind[kind] };
}
