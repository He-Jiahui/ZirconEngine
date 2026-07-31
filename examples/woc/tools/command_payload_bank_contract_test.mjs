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
const bank = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'bank_payload.rs'),
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
  [137, 'bank_deposit', 'bankDeposit', 'bankDeposit', 'BANK_DEPOSIT_COMMAND_ID'],
  [138, 'bank_withdraw', 'bankWithdraw', 'bankWithdraw', 'BANK_WITHDRAW_COMMAND_ID'],
];
for (const [id, name, method, zrFunction, rustConstant] of expected) {
  assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
    id,
    name,
    kind: 'bank_slot_optional_count',
    min_byte_length: 9,
    max_byte_length: 17,
    encoding: 'f64_le_slot+u8_presence+f64_le_count',
    source_shape: { kind: 'client_send', method, fields: ['slot', 'count'] },
  });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}
assert.deepEqual(payloads.entries.find((entry) => entry.id === 139), {
  id: 139,
  name: 'bank_buy_slots',
  kind: 'empty',
  min_byte_length: 0,
  max_byte_length: 0,
  encoding: 'empty',
  source_shape: { kind: 'client_send', method: 'bankBuySlots', fields: [] },
});
assert.match(zr, /pub bankBuySlotsCommandId\(required: bool\): uint/);
assert.match(generated, /pub const BANK_BUY_SLOTS_COMMAND_ID: u16 = 139;/);
assert.match(bank, /\bBankAction\b/);
assert.match(bank, /\bBankSlotCommandPayload\b/);
assert.match(protocol, /\bvalidate_bank_slot_optional_count_payload\b/);
for (const intent of ['DepositBank', 'WithdrawBank', 'BuyBankSlots']) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('bank command payload contracts are complete\n');
