import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const ts = require('typescript');
const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const RECIPES_PATH = 'src/sim/content/recipes.ts';
const ITEMS_PATH = 'src/sim/content/items.ts';
const CRAFTING_PATH = 'src/sim/professions/crafting.ts';
const GOLDEN_PATH = 'tests/parity/golden/professions_craft.json';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'professions_craft_parity_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'professions_craft_parity_contract.zr');
const checkOnly = process.argv.includes('--check');
const RECIPE_IDS = ['recipe_minor_healing_potion', 'recipe_eastbrook_ritual_vestments'];

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before professions craft parity contracts');
  const blobs = Object.fromEntries([RECIPES_PATH, ITEMS_PATH, CRAFTING_PATH, GOLDEN_PATH]
    .map((path) => [path, sourceBlob(path)]));
  const recipes = parseRecipes(blobs[RECIPES_PATH].toString('utf8'));
  const items = parseItems(blobs[ITEMS_PATH].toString('utf8'));
  const selected = RECIPE_IDS.map((id) => {
    const recipe = recipes.get(id);
    invariant(recipe, `missing parity recipe ${id}`);
    const item = items.get(recipe.result_item_id);
    invariant(item, `missing crafted item ${recipe.result_item_id}`);
    return { ...recipe, item };
  });
  assertRecipe(selected[0], {
    profession_id: 'alchemy', result_item_id: 'minor_healing_potion', result_count: 1,
    reagents: [{ item_id: 'linen_scrap', count: 1 }, { item_id: 'spider_leg', count: 1 }],
    skill_req: 0, item_level_budget: 1, level: 1,
  });
  assertRecipe(selected[1], {
    profession_id: 'tailoring', result_item_id: 'eastbrook_ritual_vestments', result_count: 1,
    reagents: [{ item_id: 'linen_scrap', count: 3 }, { item_id: 'spider_leg', count: 1 }],
    skill_req: 0, item_level_budget: 9, level: 9,
  });
  invariant(selected[0].item.quality === 'common' && selected[0].item.slot === '',
    'minor healing potion def quality/slot drifted');
  invariant(selected[1].item.quality === 'uncommon' && selected[1].item.slot === 'chest' &&
    selected[1].item.stats.int === 2 && selected[1].item.stats.spi === 1,
  'ritual vestments def data drifted');
  const crafting = blobs[CRAFTING_PATH].toString('utf8');
  invariant(crafting.includes('const outputQuality = defOutputQuality(def);') &&
    crafting.includes('ctx.addItemInstance(') && crafting.includes('rolled: { masterwork: true, stats: bonusStats }'),
  'crafting result-quality or masterwork-instance semantics drifted');
  const golden = JSON.parse(blobs[GOLDEN_PATH].toString('utf8'));
  invariant(golden.scenario === 'professions_craft' && golden.draws === 3 && golden.drawDigest === '07bdddf8',
    'professions craft golden scenario/RNG digest drifted');
  const masterworkFrame = golden.frames.find((frame) => frame.label === 'craft-masterwork');
  invariant(masterworkFrame, 'professions craft golden lacks masterwork frame');
  const instance = masterworkFrame.players?.[0]?.inventory?.find((slot) =>
    slot.itemId === 'eastbrook_ritual_vestments' && slot.instance?.rolled?.masterwork === true);
  invariant(instance?.instance?.rolled?.stats?.int === 1 && instance.instance.rolled.stats.spi === 1,
    'professions craft golden masterwork baked stats drifted');
  invariant(instance.instance.signer === 'Adventurer',
    'professions craft golden masterwork signer drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/professions_craft_parity_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    scenario: { seed: golden.seed, observed_draws: golden.draws, observed_draw_digest: golden.drawDigest },
    masterwork_instance: {
      signer: instance.instance.signer,
      stats: { int: instance.instance.rolled.stats.int, spi: instance.instance.rolled.stats.spi },
    },
    recipes: selected.map((recipe) => ({
      id: recipe.id,
      profession_id: recipe.profession_id,
      result_item_id: recipe.result_item_id,
      result_count: recipe.result_count,
      reagents: recipe.reagents,
      skill_req: recipe.skill_req,
      item_level_budget: recipe.item_level_budget,
      level: recipe.level,
      output_quality: recipe.item.quality,
      output_slot: recipe.item.slot,
      output_primary_stats: recipe.item.stats,
      masterwork_capable: recipe.id === 'recipe_eastbrook_ritual_vestments',
      masterwork_bumped_tier: recipe.id === 'recipe_eastbrook_ritual_vestments' ? 2 : 0,
      masterwork_baked_stats: recipe.id === 'recipe_eastbrook_ritual_vestments' ? { int: 1, spi: 1 } : {},
    })),
    source_semantics: {
      result_quality: 'CraftResult quality is the static output-def quality, including on a masterwork proc',
      masterwork: 'the ritual vestments proc is an item instance with additive baked int/spi delta; it does not mutate the output quality',
      rng: 'the full authoritative golden observes three successful-craft draws after world construction; this fixture does not claim its world-construction stream state',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'professions craft parity JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'professions craft parity Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} professions craft parity contract for ${SOURCE_COMMIT}\n`);
}

function parseRecipes(source) {
  const array = variableArray(RECIPES_PATH, source, 'COMMON_RECIPES');
  const result = new Map();
  for (const element of array.elements) {
    const object = unwrap(element);
    if (!ts.isObjectLiteralExpression(object)) continue;
    const id = stringProperty(object, 'id');
    if (!RECIPE_IDS.includes(id)) continue;
    const reagents = arrayProperty(object, 'reagents').elements.map((entry) => {
      const reagent = unwrap(entry);
      return { item_id: stringProperty(reagent, 'itemId'), count: numberProperty(reagent, 'count') };
    });
    result.set(id, {
      id,
      profession_id: stringProperty(object, 'professionId'),
      result_item_id: stringProperty(object, 'resultItemId'),
      result_count: numberProperty(object, 'resultCount'),
      reagents,
      skill_req: numberProperty(object, 'skillReq'),
      item_level_budget: numberProperty(object, 'itemLevelBudget'),
      level: numberProperty(object, 'level'),
    });
  }
  return result;
}

function parseItems(source) {
  const object = variableObject(ITEMS_PATH, source, 'BASE_ITEMS');
  const result = new Map();
  for (const property of object.properties) {
    if (!ts.isPropertyAssignment(property) || !property.name) continue;
    const id = propertyName(property.name);
    if (id !== 'minor_healing_potion' && id !== 'eastbrook_ritual_vestments') continue;
    const item = unwrap(property.initializer);
    invariant(ts.isObjectLiteralExpression(item), `item ${id} is not a literal`);
    const stats = optionalObjectProperty(item, 'stats');
    result.set(id, {
      quality: stringProperty(item, 'quality'),
      slot: optionalStringProperty(item, 'slot') ?? '',
      stats: {
        str: stats ? optionalNumberProperty(stats, 'str') ?? 0 : 0,
        agi: stats ? optionalNumberProperty(stats, 'agi') ?? 0 : 0,
        sta: stats ? optionalNumberProperty(stats, 'sta') ?? 0 : 0,
        int: stats ? optionalNumberProperty(stats, 'int') ?? 0 : 0,
        spi: stats ? optionalNumberProperty(stats, 'spi') ?? 0 : 0,
      },
    });
  }
  return result;
}

function assertRecipe(actual, expected) {
  const projection = actual && {
    profession_id: actual.profession_id,
    result_item_id: actual.result_item_id,
    result_count: actual.result_count,
    reagents: actual.reagents,
    skill_req: actual.skill_req,
    item_level_budget: actual.item_level_budget,
    level: actual.level,
  };
  invariant(JSON.stringify(projection) === JSON.stringify(expected), `recipe ${actual?.id ?? 'missing'} drifted`);
}
function variableArray(path, source, name) { const value = variableInitializer(path, source, name); invariant(ts.isArrayLiteralExpression(value), `${name} is not a literal array`); return value; }
function variableObject(path, source, name) { const value = variableInitializer(path, source, name); invariant(ts.isObjectLiteralExpression(value), `${name} is not a literal object`); return value; }
function variableInitializer(path, source, name) { const file = ts.createSourceFile(path, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS); for (const statement of file.statements) { if (!ts.isVariableStatement(statement)) continue; for (const declaration of statement.declarationList.declarations) { if (ts.isIdentifier(declaration.name) && declaration.name.text === name && declaration.initializer) return unwrap(declaration.initializer); } } throw new Error(`missing ${name}`); }
function unwrap(expression) { let value = expression; while (ts.isAsExpression(value) || ts.isTypeAssertionExpression(value) || ts.isSatisfiesExpression(value) || ts.isParenthesizedExpression(value)) value = value.expression; return value; }
function propertyName(name) { return ts.isIdentifier(name) || ts.isStringLiteral(name) ? name.text : ''; }
function propertyValue(object, name) { invariant(ts.isObjectLiteralExpression(object), `expected object while reading ${name}`); for (const property of object.properties) { if (ts.isPropertyAssignment(property) && property.name && propertyName(property.name) === name) return property.initializer; } throw new Error(`missing literal property ${name}`); }
function optionalPropertyValue(object, name) { try { return propertyValue(object, name); } catch { return null; } }
function arrayProperty(object, name) { const value = unwrap(propertyValue(object, name)); invariant(ts.isArrayLiteralExpression(value), `${name} is not an array`); return value; }
function optionalObjectProperty(object, name) { const value = optionalPropertyValue(object, name); if (!value) return null; const unwrapped = unwrap(value); invariant(ts.isObjectLiteralExpression(unwrapped), `${name} is not an object`); return unwrapped; }
function stringProperty(object, name) { const value = unwrap(propertyValue(object, name)); invariant(ts.isStringLiteral(value), `${name} is not a string`); return value.text; }
function optionalStringProperty(object, name) { const value = optionalPropertyValue(object, name); if (!value) return null; const unwrapped = unwrap(value); invariant(ts.isStringLiteral(unwrapped), `${name} is not a string`); return unwrapped.text; }
function numberProperty(object, name) { const value = unwrap(propertyValue(object, name)); invariant(ts.isNumericLiteral(value), `${name} is not numeric`); return Number(value.text); }
function optionalNumberProperty(object, name) { const value = optionalPropertyValue(object, name); if (!value) return null; const unwrapped = unwrap(value); invariant(ts.isNumericLiteral(unwrapped), `${name} is not numeric`); return Number(unwrapped.text); }
function renderZr(document) {
  const lines = [
    `// Generated from ${SOURCE_COMMIT}; do not edit by hand.`,
    `pub recipeCount(required: bool): int { return required ? ${document.recipes.length} : 0; }`,
    `pub masterworkSigner(required: bool): string { return required ? "${document.masterwork_instance.signer}" : ""; }`,
  ];
  for (const [index, recipe] of document.recipes.entries()) {
    lines.push(`pub recipeId${index}(required: bool): string { return required ? "${recipe.id}" : ""; }`);
    lines.push(`pub recipeProfession${index}(required: bool): string { return required ? "${recipe.profession_id}" : ""; }`);
    lines.push(`pub recipeResultItem${index}(required: bool): string { return required ? "${recipe.result_item_id}" : ""; }`);
    lines.push(`pub recipeResultCount${index}(required: bool): int { return required ? ${recipe.result_count} : 0; }`);
    lines.push(`pub recipeSkillReq${index}(required: bool): int { return required ? ${recipe.skill_req} : 0; }`);
    lines.push(`pub recipeBudget${index}(required: bool): float { return required ? ${recipe.item_level_budget} : 0.0; }`);
    lines.push(`pub recipeLevel${index}(required: bool): int { return required ? ${recipe.level} : 0; }`);
    lines.push(`pub recipeQuality${index}(required: bool): string { return required ? "${recipe.output_quality}" : ""; }`);
    lines.push(`pub recipeSlot${index}(required: bool): string { return required ? "${recipe.output_slot}" : ""; }`);
    lines.push(`pub recipePrimaryStr${index}(required: bool): int { return required ? ${recipe.output_primary_stats.str} : 0; }`);
    lines.push(`pub recipePrimaryAgi${index}(required: bool): int { return required ? ${recipe.output_primary_stats.agi} : 0; }`);
    lines.push(`pub recipePrimarySta${index}(required: bool): int { return required ? ${recipe.output_primary_stats.sta} : 0; }`);
    lines.push(`pub recipePrimaryInt${index}(required: bool): int { return required ? ${recipe.output_primary_stats.int} : 0; }`);
    lines.push(`pub recipePrimarySpi${index}(required: bool): int { return required ? ${recipe.output_primary_stats.spi} : 0; }`);
    lines.push(`pub recipeMasterworkCapable${index}(required: bool): bool { return required ? ${recipe.masterwork_capable} : false; }`);
    lines.push(`pub recipeMasterworkBumpedTier${index}(required: bool): int { return required ? ${recipe.masterwork_bumped_tier} : 0; }`);
    lines.push(`pub recipeMasterworkIntBonus${index}(required: bool): int { return required ? ${recipe.masterwork_baked_stats.int ?? 0} : 0; }`);
    lines.push(`pub recipeMasterworkSpiBonus${index}(required: bool): int { return required ? ${recipe.masterwork_baked_stats.spi ?? 0} : 0; }`);
    lines.push(`pub recipeReagentCount${index}(required: bool): int { return required ? ${recipe.reagents.length} : 0; }`);
    recipe.reagents.forEach((reagent, reagentIndex) => {
      lines.push(`pub recipe${index}ReagentId${reagentIndex}(required: bool): string { return required ? "${reagent.item_id}" : ""; }`);
      lines.push(`pub recipe${index}ReagentAmount${reagentIndex}(required: bool): int { return required ? ${reagent.count} : 0; }`);
    });
  }
  return `${lines.join('\n')}\n`;
}
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:professions-craft-parity-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:professions-craft-parity-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
