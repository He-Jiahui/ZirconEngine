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
const mail = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'mail_payload.rs'),
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
  [129, 'mail_take', 'mailTake', 'mailTake', 'MAIL_TAKE_COMMAND_ID'],
  [130, 'mail_delete', 'mailDelete', 'mailDelete', 'MAIL_DELETE_COMMAND_ID'],
  [131, 'mail_read', 'mailMarkRead', 'mailRead', 'MAIL_READ_COMMAND_ID'],
];
for (const [id, name, method, zrFunction, rustConstant] of expected) {
  assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
    id,
    name,
    kind: 'mail_id',
    min_byte_length: 8,
    max_byte_length: 8,
    encoding: 'f64_le_mail_id',
    source_shape: { kind: 'client_send', method, fields: ['id'] },
  });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}
assert.match(mail, /\bMailAction\b/);
assert.match(mail, /\bMailIdCommandPayload\b/);
assert.match(protocol, /\bvalidate_mail_id_payload\b/);
for (const intent of ['TakeMail', 'DeleteMail', 'MarkMailRead']) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('mail command payload contracts are complete\n');
