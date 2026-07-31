import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const arena = gitShow('src/sim/social/arena.ts');
const data = gitShow('src/sim/data.ts');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const payloads = JSON.parse(readFileSync(resolve(wocRoot, 'contracts', 'command_payloads.json'), 'utf8'));

for (const needle of [
  'export function arenaQueueJoin(',
  'if (isArenaQueued(ctx, id))',
  'if (r.e.dead)',
  'if (r.e.pos.x > DUNGEON_X_THRESHOLD)',
  "if (fmt === '1v1')",
  "if (fmt === 'yumi3' || fmt === 'yumi5')",
  'Only the party leader may queue your team for Protect Yumi.',
  'const isFiesta = fmt === \'fiesta\';',
  'const unit: ArenaQueueUnit = { pids: unitPids, rating:',
  "arenaTeamRating(ctx, unitPids, '2v2')",
  'export function arenaQueueLeave(ctx: SimContext, pid?: number): void',
  'export function arenaDequeue(ctx: SimContext, pid: number): boolean',
]) {
  invariant(arena.includes(needle), `missing pinned arena queue behavior: ${needle}`);
}
invariant(
  data.includes('export const DUNGEON_X_THRESHOLD = 600;'),
  'arena queue admission threshold drifted',
);

const queuePayload = payloads.entries.find((entry) => entry.id === 87);
invariant(
  queuePayload?.name === 'arena_queue' &&
    queuePayload.kind === 'arena_queue_format' &&
    queuePayload.min_byte_length === 1 &&
    queuePayload.max_byte_length === 1 &&
    queuePayload.encoding === 'u8_arena_format',
  'arena-queue payload contract drifted',
);

for (const needle of [
  'pub var nextArenaQueueUnitOrder: uint;',
  'pub var entityArenaQueueFormats: container.Array<uint>;',
  'pub var entityArenaQueueUnitOrders: container.Array<uint>;',
  'pub var entityArenaQueueMemberOrders: container.Array<uint>;',
  'pub var entityArenaRatings: container.Array<int>;',
  'pub var entityArena2v2Ratings: container.Array<int>;',
  'pub var entityArenaQueueRatings: container.Array<float>;',
  'var arenaQueueCommand = payloads.arenaQueueCommandId(true);',
  'var arenaLeaveCommand = payloads.arenaLeaveCommandId(true);',
  'applyArenaQueueCommand(this, actorIndex, payloadOffset, payloadBytes);',
  'applyArenaLeaveCommand(this, actorIndex);',
  'arenaQueueFormatFromWire(wireFormat: uint): uint',
  'arenaQueuePositionForIndex(state: WorldState, actorIndex: int): int',
  'arenaQueueAdmissionRating(',
  'arenaQueuePartyMembersCanJoin(',
  'queueArenaUnitForParty(',
  'applyArenaLeaveCommand(state: WorldState, actorIndex: int): void',
  'arenaQueueStateIsValid(state: WorldState): bool',
  'writer.u16(<uint>38, 1, 1);',
  'schemaVersion != <uint>20',
  'schemaVersion != <uint>21',
  'schemaVersion != <uint>22',
  'schemaVersion != <uint>23',
  'schemaVersion != <uint>24',
  'schemaVersion != <uint>25',
  'schemaVersion != <uint>26',
  'schemaVersion != <uint>27',
  'schemaVersion != <uint>28',
  'schemaVersion != <uint>29',
  'schemaVersion != <uint>30',
  'schemaVersion != <uint>31',
  'schemaVersion != <uint>32',
  'schemaVersion != <uint>33',
  'schemaVersion != <uint>34',
  'schemaVersion != <uint>35',
  'schemaVersion != <uint>36',
  'schemaVersion != <uint>37',
  'schemaVersion != <uint>38',
  'if (schemaVersion >= <uint>21) {',
  'pub arenaQueueCommandStateTest(): int',
  'if (arenaQueueCommandStateTest() != 1)',
  'appendArenaQueueCommand(',
]) {
  invariant(state.includes(needle), `WOS21 arena-queue projection omitted: ${needle}`);
}
invariant(
main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the current WOS38 snapshot version',
);

process.stdout.write(`checked WOS21 arena-queue source projection: ${SOURCE_COMMIT.slice(0, 15)}\\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
