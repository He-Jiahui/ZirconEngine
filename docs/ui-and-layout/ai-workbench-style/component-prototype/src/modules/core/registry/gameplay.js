export { gameplayAbilityCoreModule } from "./gameplay/ability.js";
export { gameplayEffectCoreModule } from "./gameplay/effect.js";
export { gameplayTagsCoreModule } from "./gameplay/tags.js";

import { gameplayAbilityCoreModule } from "./gameplay/ability.js";
import { gameplayEffectCoreModule } from "./gameplay/effect.js";
import { gameplayTagsCoreModule } from "./gameplay/tags.js";

export const gameplayCoreModules = [
  gameplayEffectCoreModule,
  gameplayAbilityCoreModule,
  gameplayTagsCoreModule
];
