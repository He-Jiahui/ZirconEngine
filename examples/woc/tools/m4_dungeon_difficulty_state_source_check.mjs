import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const sim = gitShow('src/sim/sim.ts');
const types = gitShow('src/sim/types.ts');
const setDifficulty = methodBlock(sim, '  setDungeonDifficulty(difficulty: DungeonDifficulty');
const resolveDifficulty = methodBlock(sim, '  private dungeonDifficultyForPid(pid: number)');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const projection = readFileSync(
  resolve(wocSourceRoot, 'src', 'world', 'dungeon_difficulty_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocSourceRoot, 'src', 'world', 'dungeon_difficulty_state_test_main.zr'),
  'utf8',
);
const lifecycle = readFileSync(resolve(wocSourceRoot, 'src', 'main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(
  resolve(wocSourceRoot, 'woc_m4_dungeon_difficulty_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  "export type DungeonDifficulty = 'normal' | 'heroic';",
  'export function isDungeonDifficulty(value: unknown): value is DungeonDifficulty {',
  "return value === 'normal' || value === 'heroic';",
]) {
  invariant(types.includes(needle), `missing pinned dungeon-difficulty type: ${needle}`);
}

for (const needle of [
  'if (!isDungeonDifficulty(difficulty)) return;',
  'if (party && party.leader !== r.meta.entityId)',
  "this.error(r.meta.entityId, 'You are not the party leader.');",
  "if (difficulty === 'normal') delete r.meta.dungeonDifficulty;",
  'else r.meta.dungeonDifficulty = difficulty;',
  "if (difficulty === 'normal') delete party.dungeonDifficulty;",
  'else party.dungeonDifficulty = difficulty;',
]) {
  invariant(setDifficulty.includes(needle), `missing pinned setter behavior: ${needle}`);
}

for (const needle of [
  'if (party) return party.dungeonDifficulty ?? \'normal\';',
  'return this.players.get(pid)?.dungeonDifficulty ?? \'normal\';',
]) {
  invariant(resolveDifficulty.includes(needle), `missing pinned resolver behavior: ${needle}`);
}

for (const needle of [
  'pub class DungeonDifficultyState',
  'pub isDifficulty(value: uint): bool',
  'pub effectiveDifficulty(state: DungeonDifficultyState): uint',
  'pub setDifficulty(',
  'if (state.partyActive && !isPartyLeader)',
  'state.personalHeroic = heroic;',
  'if (state.partyActive) {',
  'state.partyHeroic = heroic;',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `dungeon-difficulty projection omitted: ${needle}`);
}

for (const needle of [
  '%import("world/dungeon_difficulty_state")',
  'dungeonDifficulty.contractTest()',
]) {
  invariant(testMain.includes(needle), `missing M4 dungeon-difficulty test entry behavior: ${needle}`);
  invariant(lifecycle.includes(needle), `main lifecycle omitted dungeon-difficulty self-test: ${needle}`);
}
invariant(
  testProject.name === 'woc_m4_dungeon_difficulty_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-dungeon-difficulty-state-tests' &&
    testProject.entry === 'world/dungeon_difficulty_state_test_main',
  'M4 dungeon-difficulty test project contract drifted',
);

process.stdout.write(`checked M4 dungeon-difficulty source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function methodBlock(source, declaration) {
  const start = source.indexOf(declaration);
  invariant(start >= 0, `missing source method: ${declaration}`);
  const open = source.indexOf('{', start);
  invariant(open >= 0, `missing source method body: ${declaration}`);
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
  throw new Error(`unterminated source method: ${declaration}`);
}
