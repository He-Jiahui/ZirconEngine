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
const valeCup = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'vale_cup_payload.rs'),
  'utf8',
);
const input = readFileSync(
  join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

assert.equal(payloads.schema_version, 51);
const expected = [
  [143, 'vcup_queue', 'vale_cup_queue', 4, 'u8_bracket+u8_nation+u8_role+u8_guild', 'vcupQueueJoin', ['bracket', 'nation', 'role', 'guild'], 'valeCupQueue', 'VALE_CUP_QUEUE_COMMAND_ID'],
  [144, 'vcup_leave', 'empty', 0, 'empty', 'vcupQueueLeave', [], 'valeCupLeave', 'VALE_CUP_LEAVE_COMMAND_ID'],
  [145, 'vcup_role', 'vale_cup_role', 1, 'u8_sport_role', 'vcupSetRole', ['role'], 'valeCupRole', 'VALE_CUP_ROLE_COMMAND_ID'],
  [146, 'vcup_ready', 'empty', 0, 'empty', 'vcupReady', [], 'valeCupReady', 'VALE_CUP_READY_COMMAND_ID'],
  [147, 'vcup_bet', 'vale_cup_bet', 9, 'u8_side+f64_le_amount', 'vcupBet', ['side', 'amount'], 'valeCupBet', 'VALE_CUP_BET_COMMAND_ID'],
  [148, 'vcup_practice', 'vale_cup_bracket', 1, 'u8_vc_bracket', 'vcupPracticeStart', ['bracket'], 'valeCupPractice', 'VALE_CUP_PRACTICE_COMMAND_ID'],
];

for (const [id, name, kind, length, encoding, method, fields, zrFunction, rustConstant] of expected) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  assert(entry, `missing ${name} payload contract`);
  assert.equal(entry.name, name);
  assert.equal(entry.kind, kind);
  assert.equal(entry.min_byte_length, length);
  assert.equal(entry.max_byte_length, length);
  assert.equal(entry.encoding, encoding);
  assert.deepEqual(entry.source_shape, { kind: 'client_send', method, fields });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}

for (const type of [
  'ValeCupBracket',
  'ValeCupNation',
  'ValeCupRole',
  'ValeCupSide',
  'ValeCupQueueCommandPayload',
  'ValeCupRoleCommandPayload',
  'ValeCupBetCommandPayload',
  'ValeCupPracticeCommandPayload',
]) {
  assert.match(valeCup, new RegExp(`\\b${type}\\b`));
}
for (const validator of [
  'validate_vale_cup_queue_payload',
  'validate_vale_cup_role_payload',
  'validate_vale_cup_bet_payload',
  'validate_vale_cup_practice_payload',
]) {
  assert.match(protocol, new RegExp(`\\b${validator}\\b`));
}
for (const intent of [
  'JoinValeCupQueue',
  'LeaveValeCupQueue',
  'SetValeCupRole',
  'ReadyValeCup',
  'PlaceValeCupBet',
  'StartValeCupPractice',
]) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 148);
assert.equal(coverage.totals.typed_contract_client_send_commands, 147);
assert.equal(coverage.totals.source_shape_only_commands, 9);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('Vale Cup command payload contracts are complete\n');
