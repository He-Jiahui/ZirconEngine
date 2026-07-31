import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CRAFTING_PATH = 'src/sim/professions/crafting.ts';
const WHEEL_PATH = 'src/sim/professions/wheel.ts';
const PROFESSIONS_PATH = 'src/sim/content/professions.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'crafting_transaction_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'crafting_transaction_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before crafting transaction contracts');
  const crafting = sourceBlob(CRAFTING_PATH);
  const wheel = sourceBlob(WHEEL_PATH);
  const professions = sourceBlob(PROFESSIONS_PATH);
  const craftingText = crafting.toString('utf8');
  const wheelText = wheel.toString('utf8');
  const professionsText = professions.toString('utf8');
  const gateOrder = [
    'requiresHubStation',
    'recipe.comboRequirement',
    'isRecipeKnown(meta, recipe)',
    'hasRecipeMaterials(ctx, recipe, pid)',
    'withinCraftThrottle(meta, ctx.time)',
  ];
  const resolverStart = craftingText.indexOf('export function resolveCraftForRecipe');
  invariant(resolverStart >= 0, 'resolveCraftForRecipe is missing');
  const resolver = craftingText.slice(resolverStart);
  let prior = -1;
  for (const gate of gateOrder) {
    const index = resolver.indexOf(gate, prior + 1);
    invariant(index >= 0 && index > prior, `craft transaction gate ${gate} is absent or reordered`);
    prior = index;
  }
  invariant(craftingText.includes('const procRoll = ctx.rng.next();') &&
    craftingText.includes('procRoll < procChance') &&
    craftingText.includes('ctx.removeItem(reagent.itemId, required.count, pid);'),
  'craft transaction success/RNG ownership drifted');
  invariant(craftingText.includes('const afterSelfSigned') &&
    craftingText.includes('Math.max(1, Math.floor(afterSelfSigned * multiplier))'),
  'self-signed and specialization material discount composition drifted');
  invariant(craftingText.includes("reason: 'already_known'") &&
    craftingText.includes("reason: 'wrong_source'") &&
    craftingText.includes("if (!recipe.acquisition || recipe.acquisition.length === 0) return true;"),
  'recipe acquisition/known-state contract drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/crafting_transaction_contract_codegen.mjs',
    source_blobs: {
      [CRAFTING_PATH]: sha256(crafting),
      [WHEEL_PATH]: sha256(wheel),
      [PROFESSIONS_PATH]: sha256(professions),
    },
    constants: {
      craft_skill_gain: literalNumber(craftingText, 'CRAFT_SKILL_GAIN'),
      gold_sink_copper_per_budget: literalNumber(professionsText, 'CRAFT_GOLD_SINK_COPPER_PER_BUDGET'),
      throttle_window_seconds: literalNumber(professionsText, 'CRAFT_THROTTLE_WINDOW_SECONDS'),
      throttle_max_per_window: literalNumber(professionsText, 'CRAFT_THROTTLE_MAX_PER_WINDOW'),
      tier_skill_step: literalNumber(wheelText, 'TIER_SKILL_STEP'),
      specialized_skill_threshold: perkNumber(professionsText, 'specializedSkillThreshold'),
      material_discount_pct: perkNumber(professionsText, 'materialDiscountPct'),
    },
    denial_order: ['not_at_hub', 'combo_requirement_unmet', 'recipe_not_learned', 'insufficient_materials', 'throttled'],
    success_semantics: {
      known_recipe: 'recipes with absent or empty acquisition remain known without a learned-recipe entry; acquisition rejects unknown, already-known, then invalid sources',
      materials: 'self-signed reduction floors at one before specialized percentage discount floors at one',
      rng: 'one unconditional proc draw happens after consumption on every successful craft only',
      gold: 'the fee is ceil(itemLevelBudget * copperPerBudget) and clamps copper at zero without denying',
      progression: 'skill requirement is not an admission gate; success awards a tier/ceiling-scaled gain',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'crafting transaction JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'crafting transaction Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} crafting transaction contract for ${SOURCE_COMMIT}\n`);
}

function literalNumber(source, name) {
  const match = source.match(new RegExp(`export const ${name} = ([0-9.]+);`)) ||
    source.match(new RegExp(`const ${name} = ([0-9.]+);`));
  invariant(match, `${name} is missing or no longer a literal`);
  return Number(match[1]);
}
function perkNumber(source, name) {
  const match = source.match(new RegExp(`${name}: ([0-9.]+)`));
  invariant(match, `${name} is missing from the current perk threshold contract`);
  return Number(match[1]);
}
function renderZr(document) {
  const c = document.constants;
  const multiplier = 1 - c.material_discount_pct;
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub craftSkillGain(required: bool): float { return required ? ${c.craft_skill_gain} : 0.0; }\n` +
    `pub goldSinkCopperPerBudget(required: bool): float { return required ? ${c.gold_sink_copper_per_budget} : 0.0; }\n` +
    `pub throttleWindowSeconds(required: bool): float { return required ? ${c.throttle_window_seconds} : 0.0; }\n` +
    `pub throttleMaxPerWindow(required: bool): int { return required ? ${c.throttle_max_per_window} : 0; }\n` +
    `pub tierSkillStep(required: bool): int { return required ? ${c.tier_skill_step} : 0; }\n` +
    `pub specializedSkillThreshold(required: bool): int { return required ? ${c.specialized_skill_threshold} : 0; }\n` +
    `pub specializedMaterialMultiplier(required: bool): float { return required ? ${multiplier} : 1.0; }\n`;
}
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:crafting-transaction-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:crafting-transaction-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
