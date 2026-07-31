import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ENCHANTING_SOURCE_PATH = 'src/sim/professions/enchanting.ts';
const ENCHANTS_CONTENT_SOURCE_PATH = 'src/sim/content/enchants.ts';
const TYPES_SOURCE_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'enchanting_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'enchanting_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const enchanting = sourceBlob(ENCHANTING_SOURCE_PATH);
  const enchants = sourceBlob(ENCHANTS_CONTENT_SOURCE_PATH);
  const types = sourceBlob(TYPES_SOURCE_PATH);
  const catalog = enchantCatalog(enchants);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/enchanting_contract_codegen.mjs',
    source_blobs: {
      [ENCHANTING_SOURCE_PATH]: sha256(enchanting),
      [ENCHANTS_CONTENT_SOURCE_PATH]: sha256(enchants),
      [TYPES_SOURCE_PATH]: sha256(types),
    },
    quality_order: orderedStrings(enchanting, 'QUALITY_ORDER'),
    disenchant_material_by_quality: stringRecord(enchanting, 'DISENCHANT_MATERIAL_BY_QUALITY'),
    material_fallback: 'arcane_dust',
    tier_levels_per_bonus: 10,
    roll_threshold: 0.5,
    skill_gain: numberConstant(enchanting, 'ENCHANTING_SKILL_GAIN'),
    enchants: catalog,
    semantics: {
      eligible_item: 'only non-poor weapon and armor definitions are disenchantable',
      eligible_copy: 'plain stacks are selected before unenchanted instances; explicit enchant markers and legacy rolled stats without masterwork are excluded',
      disenchant: 'successful disenchant removes one eligible copy before one RNG draw, grants an arcane material yield, then grants enchanting skill',
      apply: 'validates item, enchant, slot, held copy and every reagent before mutation; success preserves instance fields and adds the enchant stats into a fresh marked instance',
    },
  };
  for (const needle of [
    'instance.enchant !== undefined || (!!instance.rolled?.stats && !instance.rolled.masterwork)',
    "(def.kind === 'weapon' || def.kind === 'armor')",
    'const tierBonus = Math.floor(requiredLevelFor(def) / 10);',
    'const bonus = rng.next() < 0.5 ? 0 : 1;',
    "gainCraftSkill(meta.craftSkills, 'enchanting', ENCHANTING_SKILL_GAIN);",
    'ctx.removeEnchantableItem(itemId, 1, pid);',
    'const [consumed] = ctx.removeEnchantableItem(itemId, 1, pid);',
    'merged.enchant = enchant.id;',
  ]) {
    invariant(enchanting.includes(needle), 'enchanting source drifted: ' + needle);
  }
  invariant(types.includes('rolled?: { quality?: string; stats?: Record<string, number>; masterwork?: boolean };') &&
    types.includes('enchant?: string;'), 'item-instance shape drifted');
  invariant(catalog.length === 36, 'unexpected enchant catalog length');
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'enchanting JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'enchanting Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' enchanting contract for ' + SOURCE_COMMIT + '\n');
}

function orderedStrings(source, name) {
  const expression = new RegExp('const ' + name + '[\\s\\S]*?= \\[([\\s\\S]*?)\\];');
  const match = source.match(expression);
  invariant(match, 'enchanting source no longer exposes ' + name);
  const values = [...match[1].matchAll(/'([^']+)'/g)].map((entry) => entry[1]);
  invariant(values.join(',') === 'poor,common,uncommon,rare,epic,legendary', 'unexpected enchanting quality order');
  return values;
}

function stringRecord(source, name) {
  const expression = new RegExp('const ' + name + '[\\s\\S]*?= \\{([\\s\\S]*?)\\};');
  const match = source.match(expression);
  invariant(match, 'enchanting source no longer exposes ' + name);
  const result = Object.fromEntries([...match[1].matchAll(/(\w+): '([^']+)'/g)]
    .map((entry) => [entry[1], entry[2]]));
  invariant(Object.keys(result).length === 5, 'unexpected disenchant material map');
  return result;
}

function numberConstant(source, name) {
  const match = source.match(new RegExp('const ' + name + ' = (\\d+(?:\\.\\d+)?);'));
  invariant(match, 'enchanting source no longer exposes ' + name);
  return Number(match[1]);
}

function enchantCatalog(source) {
  const start = source.indexOf('export const ENCHANTS: Record<string, EnchantDef> = {');
  invariant(start >= 0, 'enchant content no longer exposes ENCHANTS');
  const catalog = [];
  const entry = /^  (\w+): \{/gm;
  let match;
  while ((match = entry.exec(source)) !== null) {
    const end = closingBrace(source, source.indexOf('{', match.index));
    const body = source.slice(match.index, end + 1);
    const id = requiredString(body, 'id');
    const itemSlot = requiredString(body, 'itemSlot');
    const reagents = requiredReagents(body);
    const stats = requiredStats(body);
    invariant(match[1] === id, 'enchant key/id drifted: ' + match[1]);
    catalog.push({ id, item_slot: itemSlot, reagents, stats });
    entry.lastIndex = end + 1;
  }
  return catalog;
}

function closingBrace(source, start) {
  let depth = 0;
  let quote = '';
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (quote !== '') {
      if (character === '\\') {
        index += 1;
      } else if (character === quote) {
        quote = '';
      }
      continue;
    }
    if (character === "'" || character === '"' || character === '`') {
      quote = character;
    } else if (character === '{') {
      depth += 1;
    } else if (character === '}') {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  throw new Error('unterminated enchant definition');
}

function requiredString(body, field) {
  const match = body.match(new RegExp(field + ": '([^']+)',"));
  invariant(match, 'enchant definition missing ' + field);
  return match[1];
}

function requiredReagents(body) {
  const match = body.match(/reagents: \[([\s\S]*?)\],\n    statBonus:/);
  invariant(match, 'enchant definition missing reagents');
  const reagents = [...match[1].matchAll(/itemId: '([^']+)', count: (\d+)/g)]
    .map((entry) => ({ item_id: entry[1], count: Number(entry[2]) }));
  invariant(reagents.length > 0, 'enchant definition has no reagents');
  return reagents;
}

function requiredStats(body) {
  const match = body.match(/statBonus: \{ ([a-z]+): (\d+) \},/);
  invariant(match, 'enchant definition missing one stat bonus');
  return { [match[1]]: Number(match[2]) };
}

function renderZr(document) {
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub qualityCount(): int { return ' + document.quality_order.length + '; }',
    'pub qualityAt(index: int): string {',
  ];
  document.quality_order.forEach((quality, index) => {
    lines.push('    if (index == ' + index + ') return "' + quality + '";');
  });
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub materialForQuality(quality: string): string {');
  Object.entries(document.disenchant_material_by_quality).forEach(([quality, material]) => {
    lines.push('    if (quality == "' + quality + '") return "' + material + '";');
  });
  lines.push('    return "' + document.material_fallback + '";');
  lines.push('}');
  lines.push('pub tierLevelsPerBonus(): int { return ' + document.tier_levels_per_bonus + '; }');
  lines.push('pub rollThreshold(): float { return ' + document.roll_threshold + '; }');
  lines.push('pub skillGain(): float { return ' + document.skill_gain + '.0; }');
  lines.push('pub enchantCount(): int { return ' + document.enchants.length + '; }');
  lines.push('pub enchantIdAt(index: int): string {');
  document.enchants.forEach((enchant, index) => {
    lines.push('    if (index == ' + index + ') return "' + enchant.id + '";');
  });
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub enchantItemSlotAt(index: int): string {');
  document.enchants.forEach((enchant, index) => {
    lines.push('    if (index == ' + index + ') return "' + enchant.item_slot + '";');
  });
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub enchantReagentCount(index: int): int {');
  document.enchants.forEach((enchant, index) => {
    lines.push('    if (index == ' + index + ') return ' + enchant.reagents.length + ';');
  });
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub enchantReagentIdAt(index: int, reagentIndex: int): string {');
  document.enchants.forEach((enchant, index) => {
    enchant.reagents.forEach((reagent, reagentIndex) => {
      lines.push('    if (index == ' + index + ' && reagentIndex == ' + reagentIndex + ') return "' + reagent.item_id + '";');
    });
  });
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub enchantReagentAmountAt(index: int, reagentIndex: int): int {');
  document.enchants.forEach((enchant, index) => {
    enchant.reagents.forEach((reagent, reagentIndex) => {
      lines.push('    if (index == ' + index + ' && reagentIndex == ' + reagentIndex + ') return ' + reagent.count + ';');
    });
  });
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub enchantStatBonusAt(index: int, stat: string): int {');
  document.enchants.forEach((enchant, index) => {
    Object.entries(enchant.stats).forEach(([stat, value]) => {
      lines.push('    if (index == ' + index + ' && stat == "' + stat + '") return ' + value + ';');
    });
  });
  lines.push('    return 0;');
  lines.push('}');
  return lines.join('\n') + '\n';
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), label + ' is missing; run its generate script');
    invariant(readFileSync(path, 'utf8') === output, label + ' is stale; run its generate script');
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
