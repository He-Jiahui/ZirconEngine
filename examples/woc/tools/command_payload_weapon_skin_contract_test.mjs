import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const generated = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 34), {
  id: 34,
  name: 'change_weapon_skin',
  kind: 'weapon_skin_change',
  min_byte_length: 2,
  max_byte_length: 261,
  max_utf8_bytes: 256,
  encoding: 'u8_mode+(u32_le_utf8_skin|u8_weapon_type)',
  source_shape: {
    kind: 'client_send',
    method: 'changeWeaponSkin',
    fields: ['skin', 'wtype'],
  },
});
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'weapon_skin_payload.rs');
assert.match(zr, /pub changeWeaponSkinCommandId\(required: bool\): uint/);
assert.match(generated, /pub const CHANGE_WEAPON_SKIN_COMMAND_ID: u16 = 34;/);
assert.match(protocol, /pub enum WeaponSkinChange/);
assert.match(protocol, /pub enum WeaponSkinType/);
assert.match(protocol, /pub struct ChangeWeaponSkinCommandPayload/);
assert.match(input, /ChangeWeaponSkin/);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
process.stdout.write('Weapon-skin command payload contract is complete\n');
