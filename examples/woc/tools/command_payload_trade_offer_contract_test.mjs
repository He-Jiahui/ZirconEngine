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
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'trade_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 66), {
  id: 66,
  name: 'trade_offer',
  kind: 'trade_offer',
  min_byte_length: 9,
  max_byte_length: 1617,
  max_utf8_bytes: 256,
  max_collection_entries: 6,
  encoding: 'u8_count+repeated_u32_le_utf8_item_id+f64_le_count+f64_le_copper',
  source_shape: { kind: 'client_send', method: 'tradeSetOffer', fields: ['items', 'copper'] },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /tradeSetOffer\(items: InvSlot\[\], copper: number\): void\s*\{\s*this\.cmd\(\{ cmd: 'trade_offer', items, copper \}\);/,
);
assert.match(
  readSource('server', 'game.ts'),
  /case 'trade_offer':[\s\S]*?Array\.isArray\(msg\.items\)[\s\S]*?sim\.tradeSetOffer\(msg\.items, Number\(msg\.copper\) \|\| 0, pid\)/,
);
assert.match(
  readSource('src', 'sim', 'social', 'trade.ts'),
  /export function tradeSetOffer\([\s\S]*?items\.slice\(0, 6\)[\s\S]*?typeof slot\.itemId !== 'string'[\s\S]*?Number\.isFinite\(slot\.count\)[\s\S]*?Math\.max\(1, Math\.floor\(slot\.count\)\)[\s\S]*?session\.acceptedA = false;[\s\S]*?session\.acceptedB = false;/,
);
assert.match(zr, /pub tradeOfferCommandId\(required: bool\): uint/);
assert.match(generated, /pub const TRADE_OFFER_COMMAND_ID: u16 = 66;/);
assert.match(protocol, /pub struct TradeOfferCommandPayload/);
assert.match(protocol, /pub struct TradeOfferItem/);
assert.match(input, /SetTradeOffer \{\s*items: Vec<TradeOfferItem>,\s*copper: f64,/);
assert.match(
  input,
  /ClientGameplayIntent::SetTradeOffer\s*\{\s*items,\s*copper,?\s*\}\s*=>[\s\S]*?TradeOfferCommandPayload/,
);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(coverage.entries.find((entry) => entry.id === 66)?.transport_coverage, 'typed_contract');

process.stdout.write('Trade-offer command payload contract is complete\n');
