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
const input = readFileSync(
  join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

assert.equal(payloads.schema_version, 51);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 94), {
  id: 94,
  name: 'prestige',
  kind: 'empty',
  min_byte_length: 0,
  max_byte_length: 0,
  encoding: 'empty',
  source_shape: { kind: 'client_send', method: 'prestige', fields: [] },
});
assert.match(zr, /if \(id == <uint>94\) \{ return 1; \}/);
assert.match(zr, /if \(id == <uint>94\) \{ return 0; \}/);
assert.match(generated, /name: "prestige"/);
assert.match(input, /\bPrestige\b/);
assert.match(input, /empty_client_command\("prestige"\)/);

assert.equal(coverage.totals.typed_contract_commands, 148);
assert.equal(coverage.totals.typed_contract_client_send_commands, 147);
assert.equal(coverage.totals.source_shape_only_commands, 9);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('prestige command payload contract is complete\n');
