import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const arena = gitShow('src/sim/social/arena.ts');
const start = functionBlock(arena, 'export function startArenaMatch(');
const ready = functionBlock(arena, 'export function readyArenaFighter(');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src');
const projectionPath = resolve(wocSourceRoot, 'social', 'arena_state.zr');
const projection = readFileSync(projectionPath, 'utf8');

for (const needle of [
  'function cloneAbilityCharges(',
  'abilityCharges: cloneAbilityCharges(e.abilityCharges),',
]) {
  invariant(arena.includes(needle), `missing current arena return-pool behavior: ${needle}`);
}

for (const needle of [
  'e.abilityCharges = undefined;',
  'delete e.queuedOnSwingCostMultiplier;',
  'if (e.leap !== undefined) e.leap = null;',
]) {
  invariant(ready.includes(needle), `missing current arena reset behavior: ${needle}`);
}

for (const needle of [
  '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a',
  'pub var chargePoolPresent:',
  'pub var queuedSwingCostMultiplierPresent:',
  'pub var leapActive:',
  'clearArenaCombatCarryover(',
  'arenaCurrentHeadCombatResetTest(): int',
]) {
  invariant(projection.includes(needle), `arena projection omitted current-head reset field: ${needle}`);
}

for (const needle of [
  'clearArenaCombatCarryover(state, a1);',
  'clearArenaCombatCarryover(state, b1);',
  'if (a2 != 0) { clearArenaCombatCarryover(state, a2); }',
  'if (b2 != 0) { clearArenaCombatCarryover(state, b2); }',
]) {
  invariant(startMatchBlock(projection).includes(needle), `arena projection does not clear: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("social/arena_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['social/arena_state_test_main.zr', 'social/m6_scenario_matrix.zr']),
  `arena_state escaped the M6 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M6 arena current-head reset: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function functionBlock(source, declaration) {
  const start = source.indexOf(declaration);
  invariant(start >= 0, `missing source function: ${declaration}`);
  return blockFrom(source, start, declaration);
}

function startMatchBlock(source) {
  const start = source.indexOf('startMatch(\n');
  invariant(start >= 0, 'missing WOC startMatch projection');
  return blockFrom(source, start, 'startMatch');
}

function blockFrom(source, start, label) {
  const typedVoidBody = source.indexOf('): void {', start);
  const open = typedVoidBody >= 0
    ? typedVoidBody + '): void '.length
    : source.indexOf('{', start);
  invariant(open >= 0, `missing function body: ${label}`);
  let depth = 0;
  let quote = '';
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let index = open; index < source.length; index += 1) {
    const character = source[index];
    const next = source[index + 1] ?? '';
    if (lineComment) {
      if (character === '\n') lineComment = false;
      continue;
    }
    if (blockComment) {
      if (character === '*' && next === '/') {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === quote) quote = '';
      continue;
    }
    if (character === '/' && next === '/') {
      lineComment = true;
      index += 1;
      continue;
    }
    if (character === '/' && next === '*') {
      blockComment = true;
      index += 1;
      continue;
    }
    if (character === "'" || character === '"' || character === '`') {
      quote = character;
      continue;
    }
    if (character === '{') depth += 1;
    if (character === '}') {
      depth -= 1;
      if (depth === 0) return source.slice(start, index + 1);
    }
  }
  throw new Error(`unterminated function: ${label}`);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
