import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const payloads = JSON.parse(
  readFileSync(join(projectRoot, 'contracts', 'command_payloads.json'), 'utf8'),
);
const zr = readFileSync(
  join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr'),
  'utf8',
);
const generated = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'),
  'utf8',
);
const protocol = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'),
  'utf8',
);
const duelArena = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'duel_arena_payload.rs'),
  'utf8',
);
const input = readFileSync(
  join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

assert.equal(payloads.schema_version, 38);
const expected = [
  [69, 'duel_req', 'duel_request', 8, 'f64_le_target_id', 'duelRequest', ['id'], 'duelRequest', 'DUEL_REQUEST_COMMAND_ID'],
  [70, 'duel_accept', 'empty', 0, 'empty', 'duelAccept', [], 'duelAccept', 'DUEL_ACCEPT_COMMAND_ID'],
  [71, 'duel_decline', 'empty', 0, 'empty', 'duelDecline', [], 'duelDecline', 'DUEL_DECLINE_COMMAND_ID'],
  [87, 'arena_queue', 'arena_queue_format', 1, 'u8_arena_format', 'arenaQueueJoin', ['format'], 'arenaQueue', 'ARENA_QUEUE_COMMAND_ID'],
  [88, 'arena_leave', 'empty', 0, 'empty', 'arenaQueueLeave', [], 'arenaLeave', 'ARENA_LEAVE_COMMAND_ID'],
  [89, 'arena_augment', 'arena_augment', 4, 'u32_le_utf8', 'arenaAugmentPick', ['augment'], 'arenaAugment', 'ARENA_AUGMENT_COMMAND_ID'],
];

for (const [id, name, kind, minimum, encoding, method, fields, zrFunction, rustConstant] of expected) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  assert(entry, `missing ${name} payload contract`);
  assert.equal(entry.name, name);
  assert.equal(entry.kind, kind);
  assert.equal(entry.min_byte_length, minimum);
  assert.equal(entry.encoding, encoding);
  assert.deepEqual(entry.source_shape, { kind: 'client_send', method, fields });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}

assert.deepEqual(payloads.entries.find((entry) => entry.id === 89), {
  id: 89,
  name: 'arena_augment',
  kind: 'arena_augment',
  min_byte_length: 4,
  max_byte_length: 260,
  max_utf8_bytes: 256,
  max_utf16_code_units: 64,
  encoding: 'u32_le_utf8',
  source_shape: { kind: 'client_send', method: 'arenaAugmentPick', fields: ['augment'] },
});

for (const type of [
  'ArenaFormat',
  'DuelRequestCommandPayload',
  'ArenaQueueCommandPayload',
  'ArenaAugmentCommandPayload',
]) {
  assert.match(duelArena, new RegExp(`\\b${type}\\b`));
}
for (const validator of [
  'validate_duel_request_payload',
  'validate_arena_queue_payload',
  'validate_arena_augment_payload',
]) {
  assert.match(protocol, new RegExp(`\\b${validator}\\b`));
}
for (const intent of [
  'RequestDuel',
  'AcceptDuel',
  'DeclineDuel',
  'JoinArenaQueue',
  'LeaveArenaQueue',
  'PickArenaAugment',
]) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('duel-arena command payload contracts are complete\n');
