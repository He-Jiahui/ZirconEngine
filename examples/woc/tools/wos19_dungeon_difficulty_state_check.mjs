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
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'dungeon_difficulty_state.zr'),
  'utf8',
);
const payloads = JSON.parse(readFileSync(resolve(wocRoot, 'contracts', 'command_payloads.json'), 'utf8'));
const protocol = readFileSync(
  resolve(wocRoot, 'native', 'crates', 'woc_protocol', 'src', 'dungeon_difficulty_payload.rs'),
  'utf8',
);

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
  "if (difficulty === 'normal') delete r.meta.dungeonDifficulty;",
  'else r.meta.dungeonDifficulty = difficulty;',
  "if (difficulty === 'normal') delete party.dungeonDifficulty;",
  'else party.dungeonDifficulty = difficulty;',
]) {
  invariant(setDifficulty.includes(needle), `missing pinned setter behavior: ${needle}`);
}

for (const needle of [
  "if (party) return party.dungeonDifficulty ?? 'normal';",
  "return this.players.get(pid)?.dungeonDifficulty ?? 'normal';",
]) {
  invariant(resolveDifficulty.includes(needle), `missing pinned resolver behavior: ${needle}`);
}

const difficultyPayload = payloads.entries.find((entry) => entry.id === 141);
invariant(
  difficultyPayload?.name === 'set_dungeon_difficulty' &&
    difficultyPayload.kind === 'dungeon_difficulty' &&
    difficultyPayload.min_byte_length === 1 &&
    difficultyPayload.max_byte_length === 1 &&
    difficultyPayload.encoding === 'u8_dungeon_difficulty_normal_heroic',
  'dungeon-difficulty command payload contract drifted',
);
for (const needle of [
  'Self::Normal => 0,',
  'Self::Heroic => 1,',
  'other => Err(ProtocolError::InvalidDungeonDifficulty(other)),',
]) {
  invariant(protocol.includes(needle), `native difficulty encoding drifted: ${needle}`);
}

for (const needle of [
  '%import("world/dungeon_difficulty_state")',
  'pub var entityDungeonDifficultyPersonalHeroic: container.Array<bool>;',
  'pub var entityPartyDungeonDifficultyHeroic: container.Array<bool>;',
  'var setDungeonDifficultyCommand = payloads.setDungeonDifficultyCommandId(true);',
  'applyDungeonDifficultyCommand(this, actorIndex, payloadOffset, payloadBytes);',
  'partySetDungeonDifficulty(state: WorldState, partyId: uint, heroic: bool): void',
  'effectiveDungeonDifficultyForIndex(state: WorldState, index: int): uint',
  'state.entityPartyDungeonDifficultyHeroic[actorIndex] =',
  'applyDungeonDifficultyCommand(',
  'if (!dungeonDifficulty.isDifficulty(requested)) {',
  'if (partyId != <uint>0 && <uint>state.entityPartyLeaderIds[actorIndex] != actorId) {',
  'state.entityDungeonDifficultyPersonalHeroic[actorIndex] = heroic;',
  'partySetDungeonDifficulty(state, partyId, heroic);',
  'writer.u16(<uint>21, 1, 1);',
  'schemaVersion != <uint>19',
  'if (schemaVersion >= <uint>19) {',
  'woc entity dungeon difficulty marker is invalid',
  'partyDifficultyStateIsValid(state: WorldState): bool',
  'pub dungeonDifficultyCommandStateTest(): int',
  'if (dungeonDifficultyCommandStateTest() != 1)',
  'appendDungeonDifficultyCommand(',
]) {
  invariant(state.includes(needle), `WOS19 dungeon-difficulty projection omitted: ${needle}`);
}
invariant(
  main.includes('\\"world_state\\":\\"WOS21\\",'),
  'package stateSchema must expose the WOS21 snapshot version',
);

for (const needle of [
  'pub isDifficulty(value: uint): bool',
  'pub effectiveDifficulty(state: DungeonDifficultyState): uint',
  'pub setDifficulty(',
]) {
  invariant(projection.includes(needle), `shared dungeon-difficulty projection omitted: ${needle}`);
}

process.stdout.write(`checked WOS19 dungeon-difficulty source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

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
