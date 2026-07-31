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
const trade = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'trade_payload.rs'),
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
  [64, 'trade_req', 'trade_request', 8, 'f64_le_target_id', 'tradeRequest', ['id'], 'tradeRequest', 'TRADE_REQUEST_COMMAND_ID'],
  [65, 'trade_accept', 'empty', 0, 'empty', 'tradeAccept', [], 'tradeAccept', 'TRADE_ACCEPT_COMMAND_ID'],
  [67, 'trade_confirm', 'empty', 0, 'empty', 'tradeConfirm', [], 'tradeConfirm', 'TRADE_CONFIRM_COMMAND_ID'],
  [68, 'trade_cancel', 'empty', 0, 'empty', 'tradeCancel', [], 'tradeCancel', 'TRADE_CANCEL_COMMAND_ID'],
];

for (const [id, name, kind, length, encoding, method, fields, zrFunction, rustConstant] of expected) {
  assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
    id,
    name,
    kind,
    min_byte_length: length,
    max_byte_length: length,
    encoding,
    source_shape: { kind: 'client_send', method, fields },
  });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}

assert.match(trade, /\bTradeRequestCommandPayload\b/);
assert.match(protocol, /\bvalidate_trade_request_payload\b/);
for (const intent of ['RequestTrade', 'AcceptTrade', 'ConfirmTrade', 'CancelTrade']) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(
  coverage.entries.find((entry) => entry.id === 66).transport_coverage,
  'source_shape_only',
);

process.stdout.write('trade command payload contracts are complete\n');
