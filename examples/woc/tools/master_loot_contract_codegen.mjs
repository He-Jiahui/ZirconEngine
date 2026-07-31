import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/loot_master.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'master_loot_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'master_loot_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const expectedQualityRanks = {
    poor: 0,
    common: 1,
    uncommon: 2,
    rare: 3,
    epic: 4,
    legendary: 5,
  };
  for (const [quality, rank] of Object.entries(expectedQualityRanks)) {
    invariant(source.includes(quality + ': ' + rank + ','), 'Master Loot quality rank drifted: ' + quality);
  }
  for (const needle of [
    'export function meetsMasterThreshold(',
    "return QUALITY_RANK[quality ?? 'common'] >= QUALITY_RANK[threshold];",
    'export function effectiveMasterLooter(',
    'if (!settings.enabled) return null;',
    'const looter = settings.looter === 0 ? leader : settings.looter;',
    'return members.includes(looter) ? looter : leader;',
  ]) {
    invariant(source.includes(needle), 'Master Loot source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/master_loot_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    quality_ranks: expectedQualityRanks,
    semantics: {
      missing_quality: 'common',
      disabled_master_loot: 'no_looter',
      configured_looter_zero: 'leader',
      departed_configured_looter: 'leader',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'Master Loot JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Master Loot Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' Master Loot contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  for (const [name, value] of Object.entries(document.quality_ranks)) {
    lines.push('pub quality' + titleCase(name) + '(required: bool): int { return required ? ' + value + ' : 0; }\n');
  }
  lines.push('pub missingQualityCode(required: bool): int { return required ? 6 : 0; }\n');
  return lines.join('');
}

function titleCase(value) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
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
