import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const payloads = JSON.parse(readFileSync(join(projectRoot, 'contracts', 'command_payloads.json'), 'utf8'));
const zr = readFileSync(join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr'), 'utf8');
const generated = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'), 'utf8');
const protocol = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'), 'utf8');
const rite = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'delve_rite_payload.rs'), 'utf8');
const input = readFileSync(join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'), 'utf8');
const coverage = JSON.parse(readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'));

assert.equal(payloads.schema_version, 38);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 124), {
  id: 124,
  name: 'delve_rite_choose',
  kind: 'delve_rite_intensity',
  min_byte_length: 1,
  max_byte_length: 1,
  encoding: 'u8_rite_intensity_easy_medium_hard',
  source_shape: { kind: 'client_send', method: 'delveRiteChoose', fields: ['intensity'] },
});
assert.match(zr, /pub delveRiteChooseCommandId\(required: bool\): uint/);
assert.match(generated, /pub const DELVE_RITE_CHOOSE_COMMAND_ID: u16 = 124;/);
assert.match(generated, /DelveRiteIntensity/);
assert.match(rite, /DelveRiteChoosePayload/);
assert.match(rite, /InvalidDelveRiteIntensity/);
assert.match(protocol, /CommandPayloadKind::DelveRiteIntensity/);
assert.match(input, /ChooseDelveRite/);
assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
process.stdout.write('Delve Rite command payload contract is complete\n');
