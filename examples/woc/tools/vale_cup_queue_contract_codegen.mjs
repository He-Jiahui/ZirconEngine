import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const QUEUE_SOURCE_PATH = 'src/sim/social/vale_cup.ts';
const CONTENT_SOURCE_PATH = 'src/sim/content/vale_cup.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'vale_cup_queue_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'vale_cup_queue_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const queueSource = sourceBlob(QUEUE_SOURCE_PATH);
  const contentSource = sourceBlob(CONTENT_SOURCE_PATH);
  const nations = [...contentSource.matchAll(/id: '([a-z]+)', primary:/g)].map((match) => match[1]);
  invariant(nations.length === 8, 'Vale Cup nation catalog no longer has eight source entries');
  invariant(new Set(nations).size === nations.length, 'Vale Cup nation catalog contains duplicate ids');

  for (const needle of [
    'export const VC_BRACKETS: readonly VcBracket[] = [1, 2, 3, 4, 5];',
    'function normalizeRole(role: SportRole | string | undefined, bracket: VcBracket): SportRole {',
    "if (bracket <= 2) return 'allrounder';",
    'return SPORT_ROLES.includes(role as SportRole) ? (role as SportRole) : \'allrounder\';',
    'export function vcupPackTeams(',
    'if (an + u.pids.length <= size) {',
    '} else if (bn + u.pids.length <= size) {',
    'return an === size && bn === size ? { a, b } : null;',
    'function matchmakeValeCup(ctx: SimContext): void {',
    'if (earliest < bestTick) {',
  ]) {
    invariant(queueSource.includes(needle), 'Vale Cup queue source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/vale_cup_queue_contract_codegen.mjs',
    source_blobs: {
      [QUEUE_SOURCE_PATH]: sha256(queueSource),
      [CONTENT_SOURCE_PATH]: sha256(contentSource),
    },
    brackets: [1, 2, 3, 4, 5],
    nations,
    role_codes: {
      allrounder: 0,
      keeper: 1,
      sweeper: 2,
      striker: 3,
    },
    selection: {
      first_fit_team_order: ['A', 'B'],
      cross_bracket_key: 'oldest_joined_at_tick',
      equal_tick_tiebreak: 'lower_bracket',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'Vale Cup queue JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Vale Cup queue Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' Vale Cup queue contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  lines.push('pub bracketMin(required: bool): int { return required ? 1 : 0; }\n');
  lines.push('pub bracketMax(required: bool): int { return required ? 5 : 0; }\n');
  lines.push('pub nationCount(required: bool): int { return required ? ' + document.nations.length + ' : 0; }\n');
  for (const [name, value] of Object.entries(document.role_codes)) {
    lines.push('pub role' + titleCase(name) + '(required: bool): int { return required ? ' + value + ' : 0; }\n');
  }
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
