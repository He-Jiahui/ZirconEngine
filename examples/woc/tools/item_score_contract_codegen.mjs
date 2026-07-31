import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/item_level.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'item_score_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'item_score_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const armorPerPoint = literal(source, /export const ARMOR_PER_POINT = ([0-9.]+);/, 'ARMOR_PER_POINT');
  const weaponDpsWeight = literal(source, /export const WEAPON_DPS_WEIGHT = ([0-9.]+);/, 'WEAPON_DPS_WEIGHT');
  for (const needle of [
    'export function primaryStatSum(item: ItemDef): number {',
    'for (const k of PRIMARY_STATS) sum += item.stats[k] ?? 0;',
    'export function itemScore(item: ItemDef): number {',
    'if (item.stats?.armor) score += item.stats.armor / ARMOR_PER_POINT;',
    'const dps = (item.weapon.min + item.weapon.max) / 2 / item.weapon.speed;',
    'score += dps * WEAPON_DPS_WEIGHT;',
    'return Math.round(score * 10) / 10;',
  ]) {
    invariant(source.includes(needle), 'item score source drifted: ' + needle);
  }
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/item_score_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    armor_per_point: armorPerPoint,
    weapon_dps_weight: weaponDpsWeight,
    semantics: {
      primary_stats: ['str', 'agi', 'sta', 'int', 'spi'],
      score: 'primary stats + armor / armorPerPoint + average weapon DPS * weaponDpsWeight',
      rounding: 'Math.round(score * 10) / 10',
      input_boundary: 'current catalog item scores are nonnegative and any present weapon has positive speed; this permits exact scalar tenths rounding without a generic JavaScript Number adapter',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'item score JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'item score Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' item score contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  return '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n' +
    'pub armorPerPoint(): float { return ' + floatLiteral(document.armor_per_point) + '; }\n' +
    'pub weaponDpsWeight(): float { return ' + floatLiteral(document.weapon_dps_weight) + '; }\n';
}

function floatLiteral(value) {
  return Number.isInteger(value) ? String(value) + '.0' : String(value);
}

function literal(source, expression, label) {
  const match = source.match(expression);
  invariant(match, 'item score source no longer exposes ' + label);
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
