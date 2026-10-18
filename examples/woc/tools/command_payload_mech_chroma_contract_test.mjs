import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const generated = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 32), {
  id: 32,
  name: 'unequip_mech_chroma',
  kind: 'utf8_id',
  min_byte_length: 4,
  max_byte_length: 260,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8',
  source_shape: {
    kind: 'client_send',
    method: 'unequipMechChroma',
    fields: ['chroma'],
  },
});
assert.match(zr, /pub unequipMechChromaCommandId\(required: bool\): uint/);
assert.match(generated, /pub const UNEQUIP_MECH_CHROMA_COMMAND_ID: u16 = 32;/);
assert.match(protocol, /UnequipMechChromaCommandPayload/);
assert.match(input, /UnequipMechChroma/);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
process.stdout.write('Mech-chroma command payload contract is complete\n');
