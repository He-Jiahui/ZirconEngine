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
const mail = read('native', 'crates', 'woc_protocol', 'src', 'mail_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const pluginFailure = read(
  '..', '..', 'docs', 'plans', 'zircon_plugins', '08',
  'failure-2026-07-18-woc-zrvm-deterministic-bulk-cross-platform-runtime.md',
);

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 128), {
  id: 128,
  name: 'mail_send',
  kind: 'mail_send',
  min_byte_length: 21,
  max_byte_length: 16 * 1024,
  max_utf8_bytes: 16 * 1024,
  max_collection_entries: 3,
  encoding: 'u32_le_utf8_to+u32_le_utf8_subject+u32_le_utf8_body+f64_le_copper+u8_attachment_count+repeated_u32_le_utf8_item_id_f64_le_count',
  source_shape: {
    kind: 'client_send',
    method: 'mailSend',
    fields: ['to', 'subject', 'body', 'copper', 'items'],
  },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /mailSend\(to: string, subject: string, body: string, copper: number, items: InvSlot\[\]\): void[\s\S]*?items: items\.map\(\(s\) => \(\{ itemId: s\.itemId, count: s\.count \}\)\)/,
);
assert.match(readSource('server', 'main.ts'), /const WS_MAX_PAYLOAD_BYTES = 16 \* 1024;/);
assert.match(
  readSource('server', 'game.ts'),
  /case 'mail_send':[\s\S]*?msg\.items\.length > 3[\s\S]*?Math\.floor\(slot\.count\)[\s\S]*?msg\.subject\.slice\(0, 64\)[\s\S]*?msg\.body\.slice\(0, 600\)[\s\S]*?msg\.to\.trim\(\)\.slice\(0, 32\)/,
);
assert.match(zr, /pub mailSendCommandId\(required: bool\): uint/);
assert.match(generated, /pub const MAIL_SEND_COMMAND_ID: u16 = 128;/);
assert.match(generated, /MailSend,/);
assert.match(mail, /pub struct MailSendAttachment/);
assert.match(mail, /pub struct MailSendCommandPayload/);
assert.match(input, /SendMail \{\s*to: String,/);
assert.match(input, /ClientGameplayIntent::SendMail \{[\s\S]*?MailSendCommandPayload/);
assert.match(
  pluginFailure,
  /WOS166 raw JavaScript text and frame ingress addendum[\s\S]*?UTF-16 code units[\s\S]*?complete serialized[\s\S]*?JSON frame/,
);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.entries.find((entry) => entry.id === 128)?.transport_coverage, 'typed_contract');

process.stdout.write('Mail-send command payload contract is complete\n');
