import { extensionBlueprints } from "../extension-blueprints.js";
import { assetsFor } from "./assets.js";
import { hydrateSettings } from "./controls.js";
import { recipeFor } from "./recipes.js";
import { extensionSources } from "./sources.js";
import { titleWord } from "./text.js";

export const extensionModuleConfigs = extensionSources.map(([source, category, glyph]) =>
  createReferenceExtensionConfig(source, category, glyph),
);

function createReferenceExtensionConfig(source, category, glyph) {
  const id = source.replace(/^ai-|-layout\.png$/g, "");
  const label = id.split("-").map(titleWord).join(" ");
  const shortLabel = label.split(" ").slice(0, 2).join(" ");
  const subject = label.replace(/\s+(Editor|Audit|Layout|Dashboard|Manager)$/i, "");
  const recipe = recipeFor(source, category);
  const blueprint = extensionBlueprints[id] ?? {};
  const primary = blueprint.primary ?? null;
  const table = blueprint.table ?? primary?.rows ?? recipe.table(subject);
  return {
    id,
    label,
    shortLabel,
    icon: glyph,
    source,
    category,
    layoutKind: recipe.kind,
    blueprint: Boolean(primary),
    primary,
    status: blueprint.status ?? `${label} reference panel selected`,
    actions: blueprint.actions ?? recipe.actions(subject, shortLabel),
    tools: blueprint.tools ?? recipe.tools(subject),
    assets: blueprint.assets ?? assetsFor(subject, category, glyph),
    metrics: blueprint.metrics ?? recipe.metrics(subject),
    detailTabs: blueprint.detailTabs ?? recipe.detailTabs,
    settings: hydrateSettings(blueprint.settings ?? recipe.settings(subject)),
    table,
    tableHeaders: primary?.headers ?? ["Item", "State", "Value"],
    tableColumns: primary?.columns ?? "1.1fr 0.9fr 0.9fr"
  };
}
