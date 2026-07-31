import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const FOCUS_SOURCE_PATH = 'src/sim/professions/focus.ts';
const GATHERING_SOURCE_PATH = 'src/sim/professions/gathering.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'town_focus_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'town_focus_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const focus = sourceBlob(FOCUS_SOURCE_PATH);
  const gathering = sourceBlob(GATHERING_SOURCE_PATH);
  const focusPointBudget = literal(focus, /FOCUS_POINT_BUDGET = (\d+);/, 'focus point budget');
  const pointsPerTierBonus = literal(focus, /POINTS_PER_TIER_BONUS = (\d+);/, 'points per tier bonus');
  const maxFocusTierBonus = literal(focus, /MAX_FOCUS_TIER_BONUS = (\d+);/, 'maximum tier bonus');
  const yieldBonusPerPoint = literal(
    focus,
    /FOCUS_YIELD_BONUS_PER_POINT = ([0-9.]+);/,
    'focus yield bonus per point',
  );
  const tiers = harvestTiers(gathering);
  const respecTiers = {
    time: respecTier(focus, 'time'),
    timeAndPartial: respecTier(focus, 'timeAndPartial'),
    instant: respecTier(focus, 'instant'),
  };

  for (const needle of [
    'function pointsFor(focus: FocusAllocation, componentType: string): number {',
    'return Math.max(0, focus[componentType] ?? 0);',
    'return baseYield + baseYield * points * FOCUS_YIELD_BONUS_PER_POINT;',
    'const steps = Math.min(MAX_FOCUS_TIER_BONUS, Math.floor(points / POINTS_PER_TIER_BONUS));',
    'return dx * dx + dz * dz <= zone.hub.radius * zone.hub.radius;',
    "if (!isInTown) return { ok: false, allocation: previous, reason: 'not_in_town' };",
    "return { ok: false, allocation: previous, reason: 'invalid_allocation' };",
    "return { ok: false, allocation: previous, reason: 'over_budget' };",
    'magnitude += Math.abs(pointsFor(requested, componentType) - pointsFor(previous, componentType));',
  ]) {
    invariant(focus.includes(needle), 'town focus source drifted: ' + needle);
  }
  for (const needle of [
    "export type HarvestTier = 'poor' | 'common' | 'uncommon' | 'rare' | 'epic' | 'legendary';",
    'export const HARVEST_TIERS: readonly HarvestTier[] = [',
  ]) {
    invariant(gathering.includes(needle), 'harvest tier source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/town_focus_contract_codegen.mjs',
    source_blobs: {
      [FOCUS_SOURCE_PATH]: sha256(focus),
      [GATHERING_SOURCE_PATH]: sha256(gathering),
    },
    focus_point_budget: focusPointBudget,
    points_per_tier_bonus: pointsPerTierBonus,
    max_focus_tier_bonus: maxFocusTierBonus,
    focus_yield_bonus_per_point: yieldBonusPerPoint,
    harvest_tiers: tiers,
    respec_tiers: respecTiers,
    semantics: {
      allocation: 'string-keyed positive point entries preserve request order; zero entries are omitted on a successful set',
      town_gate: 'the caller supplies town position and hub circle; the boundary is inclusive',
      invalid_request: 'not-in-town, empty key, negative point, duplicate key in the typed projection, and over-budget requests preserve the prior allocation',
      respec: 'cost is the sum of absolute per-key point changes, with the three source payment tiers',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'town focus JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'town focus Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' town focus contract for ' + SOURCE_COMMIT + '\n');
}

function harvestTiers(source) {
  const match = source.match(/export const HARVEST_TIERS: readonly HarvestTier\[\] = \[([\s\S]*?)\];/);
  invariant(match, 'harvest tier source no longer exposes HARVEST_TIERS');
  const tiers = [...match[1].matchAll(/'([^']+)'/g)].map((entry) => entry[1]);
  invariant(tiers.length === 6, 'expected six source harvest tiers');
  return tiers;
}

function respecTier(source, name) {
  const expressions = {
    time: /time: \{ durationMsPerPoint: (\d[\d_]*), coinPerPoint: (\d[\d_]*), materialsPerPoint: (\d[\d_]*) \}/,
    timeAndPartial: /timeAndPartial: \{ durationMsPerPoint: (\d[\d_]*), coinPerPoint: (\d[\d_]*), materialsPerPoint: (\d[\d_]*) \}/,
    instant: /instant: \{ durationMsPerPoint: (\d[\d_]*), coinPerPoint: (\d[\d_]*), materialsPerPoint: (\d[\d_]*) \}/,
  };
  const match = source.match(expressions[name]);
  invariant(match, 'town focus source no longer exposes respec tier ' + name);
  return {
    duration_ms_per_point: numberLiteral(match[1]),
    coin_per_point: numberLiteral(match[2]),
    materials_per_point: numberLiteral(match[3]),
  };
}

function renderZr(document) {
  let output = '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n';
  output += 'pub focusPointBudget(): int { return ' + document.focus_point_budget + '; }\n';
  output += 'pub pointsPerTierBonus(): int { return ' + document.points_per_tier_bonus + '; }\n';
  output += 'pub maxFocusTierBonus(): int { return ' + document.max_focus_tier_bonus + '; }\n';
  output += 'pub focusYieldBonusPerPoint(): float { return ' +
    floatLiteral(document.focus_yield_bonus_per_point) + '; }\n';
  output += 'pub harvestTierCount(): int { return ' + document.harvest_tiers.length + '; }\n';
  output += 'pub harvestTierAt(index: int): string {\n';
  document.harvest_tiers.forEach((tier, index) => {
    output += '    if (index == ' + index + ') return "' + tier + '";\n';
  });
  output += '    return "";\n}\n';
  const tiers = [document.respec_tiers.time, document.respec_tiers.timeAndPartial, document.respec_tiers.instant];
  output += renderTierLookup('respecDurationMsPerPoint', tiers.map((tier) => tier.duration_ms_per_point));
  output += renderTierLookup('respecCoinPerPoint', tiers.map((tier) => tier.coin_per_point));
  output += renderTierLookup('respecMaterialsPerPoint', tiers.map((tier) => tier.materials_per_point));
  return output;
}

function renderTierLookup(name, values) {
  let output = 'pub ' + name + '(tierCode: int): int {\n';
  values.forEach((value, index) => {
    output += '    if (tierCode == ' + index + ') return ' + value + ';\n';
  });
  output += '    return 0;\n}\n';
  return output;
}

function floatLiteral(value) {
  return Number.isInteger(value) ? String(value) + '.0' : String(value);
}

function literal(source, expression, label) {
  const match = source.match(expression);
  invariant(match, 'town focus source no longer exposes ' + label);
  return numberLiteral(match[1]);
}

function numberLiteral(value) {
  return Number(value.replaceAll('_', ''));
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
