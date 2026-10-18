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
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'market_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 102), {
  id: 102,
  name: 'market_list',
  kind: 'utf8_id_f64_pair',
  min_byte_length: 20,
  max_byte_length: 276,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8+f64_le_x+f64_le_z',
  source_shape: { kind: 'client_send', method: 'marketList', fields: ['item', 'count', 'price'] },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /marketList\(itemId: string, count: number, price: number\): void\s*\{\s*this\.cmd\(\{ cmd: 'market_list', item: itemId, count, price \}\);/,
);
assert.match(
  readSource('server', 'game.ts'),
  /case 'market_list':[\s\S]*?typeof msg\.item === 'string'[\s\S]*?typeof msg\.count === 'number'[\s\S]*?Number\.isFinite\(msg\.count\)[\s\S]*?typeof msg\.price === 'number'[\s\S]*?Number\.isFinite\(msg\.price\)[\s\S]*?sim\.marketList\(msg\.item, msg\.count, msg\.price, pid\)/,
);
assert.match(readSource('src', 'sim', 'market.ts'), /marketList\(itemId: string, count: number, price: number, pid\?: number\): void/);
assert.match(zr, /pub marketListCommandId\(required: bool\): uint/);
assert.match(generated, /pub const MARKET_LIST_COMMAND_ID: u16 = 102;/);
assert.match(protocol, /pub struct MarketListCommandPayload/);
assert.match(protocol, /MarketListCommandPayload\.count/);
assert.match(protocol, /MarketListCommandPayload\.price/);
assert.match(input, /ListMarketItem \{\s*item_id: String,\s*count: f64,\s*price: f64,/);
assert.match(
  input,
  /ClientGameplayIntent::ListMarketItem\s*\{\s*item_id,\s*count,\s*price,?\s*\}\s*=>[\s\S]*?MarketListCommandPayload/,
);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(coverage.entries.find((entry) => entry.id === 102)?.transport_coverage, 'typed_contract');

process.stdout.write('Market-list command payload contract is complete\n');
