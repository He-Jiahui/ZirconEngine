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
const rust = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

const entry = payloads.entries.find((candidate) => candidate.name === 'releaseEmpowered');
assert.deepEqual(entry, {
  id: 149,
  name: 'releaseEmpowered',
  kind: 'utf8_id',
  min_byte_length: 4,
  max_byte_length: 260,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8',
  source_shape: {
    kind: 'client_send',
    method: 'releaseEmpoweredAbility',
    fields: ['ability'],
  },
});
assert.match(zr, /pub releaseEmpoweredCommandId\(required: bool\): uint/);
assert.match(zr, /return <uint>149;/);
assert.match(zr, /payloadKind\(<uint>149, 1\) == 4/);
assert.match(zr, /payloadMinLength\(<uint>149, true\) == 4/);
assert.match(zr, /payloadMaxLength\(<uint>149, true\) == 260/);
assert.match(rust, /pub const RELEASE_EMPOWERED_COMMAND_ID: u16 = 149;/);
assert.match(rust, /name: "releaseEmpowered",\n        kind: CommandPayloadKind::Utf8Id,/);
assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);

process.stdout.write('releaseEmpowered payload contract is complete\n');
