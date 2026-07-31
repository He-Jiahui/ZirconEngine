import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const payloads = JSON.parse(readFileSync(join(projectRoot, 'contracts', 'command_payloads.json'), 'utf8'));
const zr = readFileSync(join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr'), 'utf8');
const generated = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'), 'utf8');
const protocol = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'), 'utf8');
const eventSkin = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'event_skin_payload.rs'), 'utf8');
const input = readFileSync(join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'), 'utf8');
const coverage = JSON.parse(readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'));

assert.equal(payloads.schema_version, 38);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 33), {
  id: 33,
  name: 'claim_event_skin',
  kind: 'event_skin',
  min_byte_length: 8,
  max_byte_length: 8,
  encoding: 'f64_le_event_skin_id',
  source_shape: { kind: 'client_send', method: 'claimEventSkin', fields: ['skin'] },
});
assert.match(zr, /pub claimEventSkinCommandId\(required: bool\): uint/);
assert.match(generated, /pub const CLAIM_EVENT_SKIN_COMMAND_ID: u16 = 33;/);
assert.match(generated, /EventSkin/);
assert.match(eventSkin, /EventSkinPayload/);
assert.match(protocol, /CommandPayloadKind::EventSkin/);
assert.match(input, /ClaimEventSkin/);
assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
process.stdout.write('Event-skin command payload contract is complete\n');
