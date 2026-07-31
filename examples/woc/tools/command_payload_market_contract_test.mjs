import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const payloads = JSON.parse(readFileSync(join(projectRoot, 'contracts', 'command_payloads.json'), 'utf8'));
const zr = readFileSync(join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr'), 'utf8');
const generated = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'), 'utf8');
const protocol = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'), 'utf8');
const market = readFileSync(join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'market_payload.rs'), 'utf8');
const input = readFileSync(join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'), 'utf8');
const coverage = JSON.parse(readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'));

assert.equal(payloads.schema_version, 38);
for (const [id, name, method, zrFunction, constant, intent] of [
  [103, 'market_buy', 'marketBuy', 'marketBuy', 'MARKET_BUY_COMMAND_ID', 'BuyMarketListing'],
  [104, 'market_cancel', 'marketCancel', 'marketCancel', 'MARKET_CANCEL_COMMAND_ID', 'CancelMarketListing'],
]) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  assert.deepEqual(entry, { id, name, kind: 'market_listing_id', min_byte_length: 8, max_byte_length: 8, encoding: 'f64_le_market_listing_id', source_shape: { kind: 'client_send', method, fields: ['id'] } });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${constant}: u16 = ${id};`));
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}
assert.match(market, /MarketListingIdPayload/);
assert.match(protocol, /CommandPayloadKind::MarketListingId/);
assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
process.stdout.write('market command payload contracts are complete\n');
