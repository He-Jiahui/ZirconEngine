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
assert.deepEqual(payloads.entries.find((entry) => entry.id === 101), {
  id: 101,
  name: 'market_search',
  kind: 'market_search',
  min_byte_length: 24,
  max_byte_length: 1048,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8_q+u32_le_utf8_item_type+u32_le_utf8_subtype+u32_le_utf8_rarity+f64_le_page',
  source_shape: {
    kind: 'client_send',
    method: 'marketSearch',
    fields: ['q', 'itemType', 'subtype', 'rarity', 'page'],
  },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /marketSearch\(query: MarketQuery\): void\s*\{[\s\S]*?cmd: 'market_search',[\s\S]*?q: query\.search,[\s\S]*?itemType: query\.itemType,[\s\S]*?subtype: query\.subtype,[\s\S]*?rarity: query\.rarity,[\s\S]*?page: query\.page/,
);
assert.match(
  readSource('server', 'game.ts'),
  /case 'market_search':[\s\S]*?sim\.marketSearch\([\s\S]*?sanitizeMarketQuery\([\s\S]*?search: typeof msg\.q === 'string' \? msg\.q : '',[\s\S]*?itemType: msg\.itemType,[\s\S]*?subtype: msg\.subtype,[\s\S]*?rarity: msg\.rarity,[\s\S]*?page: typeof msg\.page === 'number' \? msg\.page : 0,/,
);
assert.match(
  readSource('src', 'sim', 'market_query.ts'),
  /export function sanitizeMarketQuery\([\s\S]*?Math\.max\(0, Math\.floor\(raw\.page\)\)[\s\S]*?raw\.search\.slice\(0, 40\)/,
);
assert.match(zr, /pub marketSearchCommandId\(required: bool\): uint/);
assert.match(generated, /pub const MARKET_SEARCH_COMMAND_ID: u16 = 101;/);
assert.match(protocol, /pub struct MarketSearchCommandPayload/);
assert.match(protocol, /MarketSearchCommandPayload\.page/);
assert.match(input, /SearchMarket \{\s*search: String,\s*item_type: String,\s*subtype: String,\s*rarity: String,\s*page: f64,/);
assert.match(
  input,
  /ClientGameplayIntent::SearchMarket\s*\{\s*search,\s*item_type,\s*subtype,\s*rarity,\s*page,?\s*\}\s*=>[\s\S]*?MarketSearchCommandPayload/,
);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(coverage.entries.find((entry) => entry.id === 101)?.transport_coverage, 'typed_contract');

process.stdout.write('Market-search command payload contract is complete\n');
