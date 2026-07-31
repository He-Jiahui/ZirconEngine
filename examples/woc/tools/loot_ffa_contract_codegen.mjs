import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/loot/loot_ffa.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'loot_ffa_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'loot_ffa_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const delay = literal(source, /export const LOOT_FFA_DELAY = (\d+);/, 'LOOT_FFA_DELAY');
  for (const needle of [
    'export function lootHasGoneFfa(lootFfaTimer: number): boolean {',
    'return lootFfaTimer <= 0;',
    'export function hasSharedLootRights(',
    'ffaUnlocked ||',
    'tappedById === null ||',
    'tappedById === pid ||',
    '!!tapperPartyMemberIds?.includes(pid)',
  ]) {
    invariant(source.includes(needle), 'loot FFA source drifted: ' + needle);
  }
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/loot_ffa_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    loot_ffa_delay_seconds: delay,
    semantics: {
      timer_lapse: 'timer <= 0',
      shared_loot_rights: ['ffa_unlocked', 'untapped', 'tapper', 'tapper_party_member'],
      party_boundary: 'an empty typed member array is behaviorally equivalent to the source null party for this membership query',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'loot FFA JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'loot FFA Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' loot FFA contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  return '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n' +
    'pub lootFfaDelaySeconds(): int { return ' + document.loot_ffa_delay_seconds + '; }\n';
}

function literal(source, expression, label) {
  const match = source.match(expression);
  invariant(match, 'loot FFA source no longer exposes ' + label);
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
