import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const TOOLS_SOURCE_PATH = 'src/sim/professions/tools.ts';
const WHEEL_SOURCE_PATH = 'src/sim/professions/wheel.ts';
const PROFESSIONS_CONTENT_SOURCE_PATH = 'src/sim/content/professions.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'tool_effect_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'tool_effect_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const tools = sourceBlob(TOOLS_SOURCE_PATH);
  const wheel = sourceBlob(WHEEL_SOURCE_PATH);
  const professions = sourceBlob(PROFESSIONS_CONTENT_SOURCE_PATH);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/tool_effect_contract_codegen.mjs',
    source_blobs: {
      [TOOLS_SOURCE_PATH]: sha256(tools),
      [WHEEL_SOURCE_PATH]: sha256(wheel),
      [PROFESSIONS_CONTENT_SOURCE_PATH]: sha256(professions),
    },
    effects: toolEffects(professions),
    rarity_order: orderedStrings(tools, 'RARITY_ORDER'),
    consumption_chance_floor: numberConstant(tools, 'CONSUMPTION_CHANCE_FLOOR'),
    consumption_chance_step: numberConstant(tools, 'CONSUMPTION_CHANCE_STEP'),
    recharge_base_materials: integerConstant(tools, 'RECHARGE_BASE_MATERIALS'),
    recharge_base_ticks: productConstant(tools, 'RECHARGE_BASE_TICKS'),
    original_crafter_discount: numberConstant(tools, 'ORIGINAL_CRAFTER_DISCOUNT'),
    specialized_skill_threshold: perkValue(professions, 'specializedSkillThreshold'),
    recharge_discount_pct: perkValue(professions, 'rechargeDiscountPct'),
    semantics: {
      gate: 'a tool covers its own tier and every lower node or monster-material tier',
      use: 'prompt effects need confirmation; an accepted use applies the effect then always consumes one RNG draw, including at zero durability',
      consumption: 'equal or lower tool rarity gap consumes at 100 percent; higher rarity gaps decrease by the fixed step but never below the floor',
      recharge: 'the original crafter pays the base half-rate, then a specialized original crafter multiplies that rate by one minus the recharge discount',
    },
  };
  for (const needle of [
    'return toolTier >= targetTier;',
    "if (slot.confirmMode === 'prompt' && !confirmed)",
    'const rolled = rng.chance(effectConsumptionChance(toolRarity, targetRarity));',
    'const isOriginal = isOriginalCrafter(slot, rechargerId);',
    'discount *= rechargeDiscountMultiplier(rechargerSkills, craftId);',
  ]) {
    invariant(tools.includes(needle), 'tool effect source drifted: ' + needle);
  }
  invariant(wheel.includes('return 1 - thresholdFor(craftId).rechargeDiscountPct;'),
    'wheel recharge discount source drifted');
  invariant(document.effects.length === 3, 'unexpected tool effect catalog length');
  invariant(document.rarity_order.join(',') === 'common,uncommon,rare,epic,legendary',
    'unexpected tool rarity order');
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'tool effect JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'tool effect Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' tool effect contract for ' + SOURCE_COMMIT + '\n');
}

function toolEffects(source) {
  const start = source.indexOf('export const TOOL_EFFECTS: Record<ToolEffectId, ToolEffectDef> = {');
  invariant(start >= 0, 'profession content no longer exposes TOOL_EFFECTS');
  const end = source.indexOf('\n};', start);
  invariant(end >= 0, 'TOOL_EFFECTS source is unterminated');
  const block = source.slice(start, end + 3);
  const catalog = [];
  const entry = /^  (\w+): \{/gm;
  let match;
  while ((match = entry.exec(block)) !== null) {
    const entryEnd = closingBrace(block, block.indexOf('{', match.index));
    const body = block.slice(match.index, entryEnd + 1);
    const effect = {
      id: requiredString(body, 'id'),
      kind: requiredString(body, 'kind'),
      bonus: requiredInteger(body, 'bonus'),
      starting_durability: requiredInteger(body, 'startingDurability'),
      craft_id: requiredString(body, 'craftId'),
    };
    invariant(effect.id === match[1], 'tool effect key/id drifted: ' + match[1]);
    catalog.push(effect);
    entry.lastIndex = entryEnd + 1;
  }
  return catalog;
}

function closingBrace(source, start) {
  let depth = 0;
  let quote = '';
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (quote !== '') {
      if (character === '\\') index += 1;
      else if (character === quote) quote = '';
      continue;
    }
    if (character === "'" || character === '"' || character === '`') quote = character;
    else if (character === '{') depth += 1;
    else if (character === '}') {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  throw new Error('unterminated tool effect definition');
}

function requiredString(body, field) {
  const match = body.match(new RegExp(field + ": '([^']+)',"));
  invariant(match, 'tool effect missing ' + field);
  return match[1];
}

function requiredInteger(body, field) {
  const match = body.match(new RegExp(field + ': (\\d+),'));
  invariant(match, 'tool effect missing ' + field);
  return Number(match[1]);
}

function orderedStrings(source, name) {
  const expression = new RegExp('const ' + name + '[\\s\\S]*?= \\[([\\s\\S]*?)\\];');
  const match = source.match(expression);
  invariant(match, 'tool source no longer exposes ' + name);
  return [...match[1].matchAll(/'([^']+)'/g)].map((entry) => entry[1]);
}

function numberConstant(source, name) {
  const match = source.match(new RegExp('(?:export )?const ' + name + ' = (\\d+(?:\\.\\d+)?);'));
  invariant(match, 'tool source no longer exposes ' + name);
  return Number(match[1]);
}

function integerConstant(source, name) {
  const match = source.match(new RegExp('const ' + name + ' = (\\d+);'));
  invariant(match, 'tool source no longer exposes ' + name);
  return Number(match[1]);
}

function productConstant(source, name) {
  const match = source.match(new RegExp('const ' + name + ' = ([0-9][0-9\\s*]*);'));
  invariant(match, 'tool source no longer exposes ' + name + ' as a positive product');
  const factors = match[1].split('*').map((factor) => factor.trim());
  invariant(factors.every((factor) => /^\d+$/.test(factor)), name + ' product has unsupported factors');
  return factors.reduce((product, factor) => product * Number(factor), 1);
}

function perkValue(source, field) {
  const match = source.match(new RegExp(field + ': (\\d+(?:\\.\\d+)?)'));
  invariant(match, 'profession content no longer exposes ' + field);
  return Number(match[1]);
}

function renderZr(document) {
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub effectCount(): int { return ' + document.effects.length + '; }',
    'pub effectIdAt(index: int): string {',
  ];
  document.effects.forEach((effect, index) => lines.push('    if (index == ' + index + ') return "' + effect.id + '";'));
  lines.push('    return "";');
  lines.push('}');
  for (const [functionName, key, type] of [
    ['effectKindAt', 'kind', 'string'],
    ['effectBonusAt', 'bonus', 'int'],
    ['effectStartingDurabilityAt', 'starting_durability', 'int'],
    ['effectCraftIdAt', 'craft_id', 'string'],
  ]) {
    lines.push('pub ' + functionName + '(index: int): ' + type + ' {');
    document.effects.forEach((effect, index) => {
      const value = effect[key];
      lines.push('    if (index == ' + index + ') return ' +
        (type === 'string' ? '"' + value + '"' : value) + ';');
    });
    lines.push(type === 'string' ? '    return "";' : '    return 0;');
    lines.push('}');
  }
  lines.push('pub rarityIndex(rarity: string): int {');
  document.rarity_order.forEach((rarity, index) => lines.push('    if (rarity == "' + rarity + '") return ' + index + ';'));
  lines.push('    return -1;');
  lines.push('}');
  lines.push('pub consumptionChanceFloor(): float { return ' + document.consumption_chance_floor + '; }');
  lines.push('pub consumptionChanceStep(): float { return ' + document.consumption_chance_step + '; }');
  lines.push('pub rechargeBaseMaterials(): int { return ' + document.recharge_base_materials + '; }');
  lines.push('pub rechargeBaseTicks(): int { return ' + document.recharge_base_ticks + '; }');
  lines.push('pub originalCrafterDiscount(): float { return ' + document.original_crafter_discount + '; }');
  lines.push('pub specializedSkillThreshold(): float { return ' + document.specialized_skill_threshold + '.0; }');
  lines.push('pub rechargeDiscountPct(): float { return ' + document.recharge_discount_pct + '; }');
  return lines.join('\n') + '\n';
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), label + ' is missing; run its generate script');
    invariant(readFileSync(path, 'utf8') === output, label + ' is stale; run its generate script');
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
