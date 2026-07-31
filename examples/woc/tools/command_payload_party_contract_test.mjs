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
const party = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'party_payload.rs'),
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
  [48, 'setLootMaster', 'party_loot_master', 10, 'u8_enabled+f64_le_looter+u8_threshold', 'setPartyLootMaster', ['enabled', 'looter', 'threshold'], 'partySetLootMaster', 'PARTY_SET_LOOT_MASTER_COMMAND_ID'],
  [50, 'setMarker', 'party_marker', 16, 'f64_le_entity_id+f64_le_marker_id', 'setMarker', ['id', 'marker'], 'partySetMarker', 'PARTY_SET_MARKER_COMMAND_ID'],
  [51, 'clearMarker', 'party_marker_clear', 8, 'f64_le_entity_id', 'clearMarker', ['id'], 'partyClearMarker', 'PARTY_CLEAR_MARKER_COMMAND_ID'],
  [52, 'readyrespond', 'boolean', 1, 'u8_false_or_true', 'readyCheckRespond', ['ready'], 'partyReadyRespond', 'PARTY_READY_RESPOND_COMMAND_ID'],
];

for (const [id, name, kind, length, encoding, method, fields, zrFunction, rustConstant] of expected) {
  assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
    id,
    name,
    kind,
    min_byte_length: length,
    max_byte_length: length,
    encoding,
    source_shape: { kind: 'client_send', method, fields },
  });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}

for (const type of [
  'MasterLootThreshold',
  'PartyLootMasterCommandPayload',
  'PartyMarkerCommandPayload',
  'PartyMarkerClearCommandPayload',
  'ReadyCheckRespondCommandPayload',
]) {
  assert.match(party, new RegExp(`\\b${type}\\b`));
}
for (const validator of [
  'validate_party_loot_master_payload',
  'validate_party_marker_payload',
  'validate_party_marker_clear_payload',
]) {
  assert.match(protocol, new RegExp(`\\b${validator}\\b`));
}
for (const intent of [
  'SetPartyLootMaster',
  'SetPartyMarker',
  'ClearPartyMarker',
  'RespondToReadyCheck',
]) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('party command payload contracts are complete\n');
