import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const payloads = JSON.parse(readFileSync(join(projectRoot, 'contracts', 'command_payloads.json'), 'utf8'));
const zr = readFileSync(join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr'), 'utf8');
const generated = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'), 'utf8');
const protocol = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'), 'utf8');
const lootRoll = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'loot_roll_payload.rs'), 'utf8');
const input = readFileSync(join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'), 'utf8');
const coverage = JSON.parse(readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'));

assert.equal(payloads.schema_version, 38);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 14), {
  id: 14,
  name: 'lootRoll',
  kind: 'loot_roll',
  min_byte_length: 9,
  max_byte_length: 9,
  encoding: 'f64_le_roll_id+u8_need_greed_pass',
  source_shape: { kind: 'client_send', method: 'submitLootRoll', fields: ['rollId', 'choice'] },
});
assert.match(zr, /pub lootRollCommandId\(required: bool\): uint/);
assert.match(generated, /pub const LOOT_ROLL_COMMAND_ID: u16 = 14;/);
assert.match(generated, /LootRoll/);
assert.match(lootRoll, /LootRollPayload/);
assert.match(lootRoll, /InvalidLootRollChoice/);
assert.match(protocol, /CommandPayloadKind::LootRoll/);
assert.match(input, /SubmitLootRoll/);
assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
process.stdout.write('Loot-roll command payload contract is complete\n');
