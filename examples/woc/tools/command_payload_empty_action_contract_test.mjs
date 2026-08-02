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
const expected = [
  [28, 'sell_all_junk', 'sellAllJunk', 'sellAllJunk', 'SELL_ALL_JUNK_COMMAND_ID', 'SellAllJunk'],
  [105, 'market_collect', 'marketCollect', 'marketCollect', 'MARKET_COLLECT_COMMAND_ID', 'CollectMarketProceeds'],
  [114, 'leave_dungeon', 'leaveDungeon', 'leaveDungeon', 'LEAVE_DUNGEON_COMMAND_ID', 'LeaveDungeon'],
  [116, 'leave_delve', 'leaveDelve', 'leaveDelve', 'LEAVE_DELVE_COMMAND_ID', 'LeaveDelve'],
];
for (const [id, name, method, zrFunction, rustConstant, intent] of expected) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  assert.deepEqual(entry, {
    id,
    name,
    kind: 'empty',
    min_byte_length: 0,
    max_byte_length: 0,
    encoding: 'empty',
    source_shape: { kind: 'client_send', method, fields: [] },
  });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 148);
assert.equal(coverage.totals.typed_contract_client_send_commands, 147);
assert.equal(coverage.totals.source_shape_only_commands, 9);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('empty-action command payload contracts are complete\n');
