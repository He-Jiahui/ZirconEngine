import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const readSource = (...parts) => readFileSync(join(sourceRoot, ...parts), 'utf8');
const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
const generated = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 115), {
  id: 115,
  name: 'enter_delve',
  kind: 'utf8_id_pair',
  min_byte_length: 8,
  max_byte_length: 520,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8+u32_le_utf8',
  source_shape: { kind: 'client_send', method: 'enterDelve', fields: ['delveId', 'tierId'] },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /enterDelve\(delveId: string, tierId: string\): void\s*\{\s*this\.cmd\(\{ cmd: 'enter_delve', delveId, tierId \}\);/,
);
assert.match(
  readSource('server', 'game.ts'),
  /case 'enter_delve':[\s\S]*?typeof msg\.delveId !== 'string'[\s\S]*?typeof msg\.tierId !== 'string'[\s\S]*?Math\.hypot\(e\.pos\.x - delve\.doorPos\.x, e\.pos\.z - delve\.doorPos\.z\) > 12[\s\S]*?sim\.enterDelve\(msg\.delveId, msg\.tierId, pid\)[\s\S]*?this\.resyncDelves\(session\);/,
);
assert.match(readSource('src', 'sim', 'delves', 'runs.ts'), /export function enterDelve\(/);
assert.match(zr, /pub enterDelveCommandId\(required: bool\): uint/);
assert.match(generated, /pub const ENTER_DELVE_COMMAND_ID: u16 = 115;/);
assert.match(protocol, /pub struct EnterDelveCommandPayload/);
assert.match(protocol, /encode_utf8_id_pair\(ENTER_DELVE_COMMAND_ID, &self\.delve_id, &self\.tier_id\)/);
assert.match(input, /EnterDelve \{\s*delve_id: String,\s*tier_id: String,/);
assert.match(
  input,
  /ClientGameplayIntent::EnterDelve\s*\{\s*delve_id,\s*tier_id\s*\}\s*=>[\s\S]*?EnterDelveCommandPayload/,
);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(coverage.entries.find((entry) => entry.id === 115)?.transport_coverage, 'typed_contract');

process.stdout.write('Enter-delve command payload contract is complete\n');
