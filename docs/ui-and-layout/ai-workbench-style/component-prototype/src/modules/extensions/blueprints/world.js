import { foliageEditorBlueprint } from "./world/foliage-editor.js";
import { levelStreamingBlueprint } from "./world/level-streaming.js";
import { levelVariantBlueprint } from "./world/level-variant.js";
import { prefabEditorBlueprint } from "./world/prefab-editor.js";
import { scatterEditorBlueprint } from "./world/scatter-editor.js";
import { terrainEditorBlueprint } from "./world/terrain-editor.js";
import { volumeEditorBlueprint } from "./world/volume-editor.js";
import { weatherEditorBlueprint } from "./world/weather-editor.js";

export const worldBlueprints = {
  "terrain-editor": terrainEditorBlueprint,
  "foliage-editor": foliageEditorBlueprint,
  "level-streaming": levelStreamingBlueprint,
  "level-variant": levelVariantBlueprint,
  "prefab-editor": prefabEditorBlueprint,
  "scatter-editor": scatterEditorBlueprint,
  "volume-editor": volumeEditorBlueprint,
  "weather-editor": weatherEditorBlueprint
};
