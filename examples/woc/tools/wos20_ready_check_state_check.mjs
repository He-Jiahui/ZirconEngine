import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const readyCheck = gitShow('src/sim/social/ready_check.ts');
const party = gitShow('src/sim/social/party.ts');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const payloads = JSON.parse(readFileSync(resolve(wocRoot, 'contracts', 'command_payloads.json'), 'utf8'));

for (const needle of [
  'export const READY_CHECK_SECONDS = 30;',
  'if (party.leader !== r.meta.entityId)',
  'if (ctx.readyChecks.has(party.id))',
  "responses.set(mPid, mPid === r.meta.entityId ? 'ready' : 'pending');",
  'endsAt: ctx.time + READY_CHECK_SECONDS,',
  "check.responses.set(r.meta.entityId, ready ? 'ready' : 'notready');",
  'if (ctx.time >= check.endsAt || !pending) finalizeReadyCheck(ctx, check);',
  'ctx.readyChecks.delete(check.partyId);',
]) {
  invariant(readyCheck.includes(needle), `missing pinned ready-check behavior: ${needle}`);
}

for (const needle of [
  'this.ctx.readyChecks.delete(old.id);',
  'this.ctx.readyChecks.get(party.id)?.responses.delete(pid);',
]) {
  invariant(party.includes(needle), `missing pinned party ready-check cleanup: ${needle}`);
}

const responsePayload = payloads.entries.find((entry) => entry.id === 52);
invariant(
  responsePayload?.name === 'readyrespond' &&
    responsePayload.kind === 'boolean' &&
    responsePayload.min_byte_length === 1 &&
    responsePayload.max_byte_length === 1 &&
    responsePayload.encoding === 'u8_false_or_true',
  'ready-check response payload contract drifted',
);

for (const needle of [
  'pub var entityReadyCheckPartyIds: container.Array<uint>;',
  'pub var entityReadyCheckInitiatorIds: container.Array<uint>;',
  'pub var entityReadyCheckEndsAtMicros: container.Array<uint>;',
  'pub var entityReadyCheckResponses: container.Array<uint>;',
  'var partyReadyRespondCommand = payloads.partyReadyRespondCommandId(true);',
  'applyPartyReadyRespondCommand(this, actorIndex, payloadOffset, payloadBytes);',
  'startPartyReadyCheck(state: WorldState, actorIndex: int): void',
  'applyPartyReadyRespondCommand(',
  'updatePartyReadyChecks(state);',
  'partyReadyCheckStateIsValid(state: WorldState): bool',
  'partyRemoveReadyCheckMember(state: WorldState, index: int): void',
  'writer.u16(<uint>21, 1, 1);',
  'schemaVersion != <uint>20',
  'if (schemaVersion >= <uint>20) {',
  'pub partyReadyCheckCommandStateTest(): int',
  'if (partyReadyCheckCommandStateTest() != 1)',
  'appendPartyReadyRespondCommand(',
]) {
  invariant(state.includes(needle), `WOS20 ready-check projection omitted: ${needle}`);
}
invariant(
  main.includes('\\"world_state\\":\\"WOS21\\",'),
  'package stateSchema must expose the WOS21 snapshot version',
);

process.stdout.write(`checked WOS20 ready-check source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
