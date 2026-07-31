import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ARCHETYPE_SOURCE_PATH = 'src/sim/professions/archetype.ts';
const PROFESSIONS_SOURCE_PATH = 'src/sim/content/professions.ts';
const RECIPES_SOURCE_PATH = 'src/sim/content/recipes.ts';
const WHEEL_SOURCE_PATH = 'src/sim/professions/wheel.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'archetype_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'archetype_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const archetype = sourceBlob(ARCHETYPE_SOURCE_PATH);
  const professions = sourceBlob(PROFESSIONS_SOURCE_PATH);
  const recipes = sourceBlob(RECIPES_SOURCE_PATH);
  const wheel = sourceBlob(WHEEL_SOURCE_PATH);
  const crafts = craftRing(professions);
  const comboPairs = uniqueComboPairs(recipes);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/archetype_contract_codegen.mjs',
    source_blobs: {
      [ARCHETYPE_SOURCE_PATH]: sha256(archetype),
      [PROFESSIONS_SOURCE_PATH]: sha256(professions),
      [RECIPES_SOURCE_PATH]: sha256(recipes),
      [WHEEL_SOURCE_PATH]: sha256(wheel),
    },
    craft_ring: crafts,
    archetype_pairs: crafts.map((craft, index) => ({
      id: craft.id + '+' + crafts[(index + 1) % crafts.length].id,
      active: craft.id,
      paired: crafts[(index + 1) % crafts.length].id,
    })),
    combo_pairs: comboPairs,
    common_ceiling_tier: numberConstant(archetype, 'COMMON_CEILING_TIER'),
    rare_ceiling_tier: numberConstant(archetype, 'RARE_CEILING_TIER'),
    amends_base: requiredFormulaConstant(archetype, 'return 5 + priorSwitches * 3;', 0),
    amends_step: requiredFormulaConstant(archetype, 'return 5 + priorSwitches * 3;', 1),
    tier_skill_step: numberConstant(wheel, 'TIER_SKILL_STEP'),
    semantics: {
      identity: 'an active archetype is an ordered adjacent pair: two uncapped majors, an explicit opposite-craft hobby capped at rare, and every other craft capped at common',
      default_pair: 'a craft with a combo partner selects that adjacent partner; every other craft selects its previous ring neighbor',
      history: 'new attunements append a canonical pair without switch count; returning to a held non-current pair increments switch count and resets amends',
      direct_switch: 'the legacy switch requires floor-clamped amends progress of base plus step per prior switch, then rederives pair and hobby without mutating skills',
      bridge: 'the crafting projection receives active/paired ids plus the selected recipe craft ceiling; -1 encodes source Infinity for the existing transaction state',
    },
  };
  for (const needle of [
    'return (match ?? neighbors[0]).id;',
    'return 5 + priorSwitches * 3;',
    'if (activeArchetype === null) return RARE_CEILING_TIER;',
    "if (craftId === activeArchetype || craftId === pairedMajor) return Infinity;",
    "if ((mode === 'new' && seen) || (mode === 'return' && !seen)) return false;",
    'state.amendsProgress = 0;',
  ]) {
    invariant(archetype.includes(needle), 'archetype source drifted: ' + needle);
  }
  invariant(crafts.length === 10, 'unexpected craft ring size');
  invariant(document.archetype_pairs.length === 10, 'unexpected archetype pair count');
  invariant(comboPairs.length === 2, 'unexpected unique combo pair count');
  invariant(document.common_ceiling_tier === 0 && document.rare_ceiling_tier === 2, 'unexpected archetype ceilings');
  invariant(document.amends_base === 5 && document.amends_step === 3, 'unexpected amends formula');
  invariant(document.tier_skill_step === 25, 'unexpected craft tier step');
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'archetype JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'archetype Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' archetype contract for ' + SOURCE_COMMIT + '\n');
}

function craftRing(source) {
  const objects = topLevelObjects(arrayAfterAssignment(source, 'export const CRAFT_RING'));
  const crafts = objects.map((body) => ({ id: requiredString(body, 'id') }));
  invariant(crafts.length > 0, 'craft ring is empty');
  return crafts;
}

function uniqueComboPairs(source) {
  const block = arrayAfterAssignment(source, 'export const COMBO_RECIPES');
  const pairs = [...block.matchAll(/comboRequirement: \{ craftA: '([^']+)', craftB: '([^']+)', minTier: \d+ \}/g)]
    .map((entry) => ({ craft_a: entry[1], craft_b: entry[2] }));
  const unique = new Map();
  pairs.forEach((pair) => unique.set(pair.craft_a + '+' + pair.craft_b, pair));
  return [...unique.values()];
}

function numberConstant(source, name) {
  const match = source.match(new RegExp('(?:export )?const ' + name + ' = (\\d+(?:\\.\\d+)?);'));
  invariant(match, 'source no longer exposes ' + name);
  return Number(match[1]);
}

function requiredFormulaConstant(source, formula, index) {
  invariant(source.includes(formula), 'source formula drifted: ' + formula);
  return index === 0 ? 5 : 3;
}

function requiredString(body, field) {
  const match = body.match(new RegExp(field + ": '([^']+)'"));
  invariant(match, 'ring entry missing ' + field);
  return match[1];
}

function arrayAfterAssignment(source, anchor) {
  const assignment = source.indexOf('=', source.indexOf(anchor));
  invariant(assignment >= 0, 'source no longer exposes ' + anchor);
  return balancedBlock(source, source.indexOf('[', assignment), '[', ']');
}

function balancedBlock(source, start, open, close) {
  invariant(start >= 0, 'source block start missing');
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
    else if (character === open) depth += 1;
    else if (character === close) {
      depth -= 1;
      if (depth === 0) return source.slice(start + 1, index);
    }
  }
  throw new Error('unterminated source block');
}

function topLevelObjects(block) {
  const result = [];
  for (let index = 0; index < block.length; index += 1) {
    if (block[index] !== '{') continue;
    let depth = 0;
    let quote = '';
    let end = index;
    for (; end < block.length; end += 1) {
      const character = block[end];
      if (quote !== '') {
        if (character === '\\') end += 1;
        else if (character === quote) quote = '';
        continue;
      }
      if (character === "'" || character === '"' || character === '`') quote = character;
      else if (character === '{') depth += 1;
      else if (character === '}') {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    result.push(block.slice(index, end + 1));
    index = end;
  }
  return result;
}

function renderZr(document) {
  const crafts = document.craft_ring.map((craft) => craft.id);
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub craftCount(): int { return ' + crafts.length + '; }',
    'pub craftAt(index: int): string {',
  ];
  crafts.forEach((craft, index) => lines.push('    if (index == ' + index + ') return "' + craft + '";'));
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub craftIndex(id: string): int {');
  crafts.forEach((craft, index) => lines.push('    if (id == "' + craft + '") return ' + index + ';'));
  lines.push('    return -1;');
  lines.push('}');
  lines.push('pub previousCraft(id: string): string {');
  crafts.forEach((craft, index) => lines.push('    if (id == "' + craft + '") return "' + crafts[(index - 1 + crafts.length) % crafts.length] + '";'));
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub nextCraft(id: string): string {');
  crafts.forEach((craft, index) => lines.push('    if (id == "' + craft + '") return "' + crafts[(index + 1) % crafts.length] + '";'));
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub oppositeCraft(id: string): string {');
  crafts.forEach((craft, index) => lines.push('    if (id == "' + craft + '") return "' + crafts[(index + crafts.length / 2) % crafts.length] + '";'));
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub archetypePairCount(): int { return ' + document.archetype_pairs.length + '; }');
  for (const [name, key] of [['Id', 'id'], ['Active', 'active'], ['Paired', 'paired']]) {
    lines.push('pub archetypePair' + name + 'At(index: int): string {');
    document.archetype_pairs.forEach((pair, index) => lines.push('    if (index == ' + index + ') return "' + pair[key] + '";'));
    lines.push('    return "";');
    lines.push('}');
  }
  lines.push('pub comboPartner(craftId: string): string {');
  document.combo_pairs.forEach((pair) => {
    lines.push('    if (craftId == "' + pair.craft_a + '") return "' + pair.craft_b + '";');
    lines.push('    if (craftId == "' + pair.craft_b + '") return "' + pair.craft_a + '";');
  });
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub commonCeilingTier(): int { return ' + document.common_ceiling_tier + '; }');
  lines.push('pub rareCeilingTier(): int { return ' + document.rare_ceiling_tier + '; }');
  lines.push('pub amendsBase(): float { return ' + document.amends_base + '.0; }');
  lines.push('pub amendsStep(): float { return ' + document.amends_step + '.0; }');
  lines.push('pub tierSkillStep(): float { return ' + document.tier_skill_step + '.0; }');
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

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
