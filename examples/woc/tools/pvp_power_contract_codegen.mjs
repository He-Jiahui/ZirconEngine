import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/pvp/power.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'pvp_power_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'pvp_power_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const ratingPerPct = literal(source, /export const PVP_RATING_PER_PCT = ([\d.]+);/, 'rating per percent');
  const offenseCap = literal(source, /export const PVP_OFFENSE_CAP = ([\d.]+);/, 'offense cap');
  const defenseCap = literal(source, /export const PVP_DEFENSE_CAP = ([\d.]+);/, 'defense cap');
  for (const needle of [
    'function pvpFractionFromRating(rating: number, cap: number): number {',
    'return Math.min(cap, Math.max(0, rating) / (PVP_RATING_PER_PCT * 100));',
    'export function pvpFractionsFromRatings(',
    'export function pvpDamageMultiplier(source: Entity, target: Entity): number {',
    'return (1 + offense) * (1 - defense);',
  ]) {
    invariant(source.includes(needle), 'PvP power source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/pvp_power_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    rating_per_pct: ratingPerPct,
    offense_cap: offenseCap,
    defense_cap: defenseCap,
    rating_denominator: ratingPerPct * 100,
    damage_multiplier: '(1 + clamp(offense, 0, offense_cap)) * (1 - clamp(defense, 0, defense_cap))',
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'PvP power JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'PvP power Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' PvP power contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  return '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n' +
    'pub ratingPerPct(): float { return ' + document.rating_per_pct + '.0; }\n' +
    'pub offenseCap(): float { return ' + document.offense_cap + '; }\n' +
    'pub defenseCap(): float { return ' + document.defense_cap + '; }\n' +
    'pub ratingDenominator(): float { return ' + document.rating_denominator + '.0; }\n';
}

function literal(source, expression, label) {
  const match = source.match(expression);
  invariant(match, 'PvP power source no longer exposes ' + label);
  return Number(match[1]);
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
