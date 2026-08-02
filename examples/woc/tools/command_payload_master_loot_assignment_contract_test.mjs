import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const generated = read(
  'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs',
);
const nativePayload = read(
  'native', 'crates', 'woc_protocol', 'src', 'master_loot_assignment_payload.rs',
);
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const coverage = JSON.parse(
  read('reference', 'current-head', 'command_payload_coverage.json'),
);

assert.equal(payloads.schema_version, 51);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 49), {
  id: 49,
  name: 'masterAssign',
  kind: 'master_loot_assignment',
  min_byte_length: 9,
  max_byte_length: 89,
  encoding: 'f64_le_roll_id+u8_count_0_to_10+f64_le_target_pid',
  source_shape: {
    kind: 'client_send',
    method: 'assignMasterLoot',
    fields: ['rollId', 'pids'],
  },
});
assert.match(zr, /pub masterAssignCommandId\(required: bool\): uint/);
assert.match(generated, /pub const MASTER_ASSIGN_COMMAND_ID: u16 = 49;/);
assert.match(generated, /MasterLootAssignment/);
assert.match(nativePayload, /pub struct MasterLootAssignmentPayload/);
assert.match(nativePayload, /MAX_TARGET_PIDS: usize = 10/);
assert.match(protocol, /validate_master_loot_assignment_payload/);
assert.match(input, /AssignMasterLoot/);
assert.match(input, /MASTER_ASSIGN_COMMAND_ID/);

const entry = coverage.entries.find((candidate) => candidate.id === 49);
assert.equal(entry.transport_coverage, 'typed_contract');
assert.equal(entry.descriptor.kind, 'master_loot_assignment');
assert.equal(coverage.totals.typed_contract_commands, 148);
assert.equal(coverage.totals.typed_contract_client_send_commands, 147);
assert.equal(coverage.totals.source_shape_only_commands, 9);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('master-loot assignment payload contract is complete\n');
