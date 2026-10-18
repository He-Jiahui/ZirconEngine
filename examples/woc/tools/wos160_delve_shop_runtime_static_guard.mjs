import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = path.resolve(root, '..', '..');
const sourceRoot = path.resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), 'utf8');
const source = (file) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${file}`], { encoding: 'utf8' },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'delve_buy':[\s\S]*?typeof msg\.delveId !== 'string'[\s\S]*?typeof msg\.itemId !== 'string'[\s\S]*?DELVES\[msg\.delveId\][\s\S]*?e\.dead[\s\S]*?Math\.hypot[\s\S]*?> 12[\s\S]*?sim\.delveBuyShopItem\(msg\.delveId, msg\.itemId, pid\)/,
  'source server delve-buy admission or range drifted');
const sourceRuns = source('src/sim/delves/runs.ts');
requireText(sourceRuns,
  /delveBuyShopItem\([\s\S]*?DELVE_SHOPS\[delveId\][\s\S]*?ITEMS\[itemId\][\s\S]*?!delveShopGateMet[\s\S]*?meta\.delveMarks < entry\.marks[\s\S]*?meta\.delveMarks -= entry\.marks[\s\S]*?ctx\.addItem\(itemId, 1[\s\S]*?type: 'vendor'/,
  'source delve shop transaction ordering drifted');
const sourceShop = source('src/sim/content/delves/shop.ts');
if ((sourceShop.match(/itemId: '[^']+', marks: \d+, gate: '[^']+'/g) ?? []).length !== 18) {
  throw new Error('source Delve shop stock drifted');
}

const m7Shop = JSON.parse(read('contracts', 'm7_delve_shop_content.json'));
if (m7Shop.content?.shops?.length !== 2 ||
    m7Shop.content.shops.some((shop) => shop.offers.length !== 9)) {
  throw new Error('source-pinned M7 Delve shop projection is incomplete');
}
const content = JSON.parse(read('contracts', 'm5_content.json'));
const shopItemIds = m7Shop.content.shops.flatMap(
  (shop) => shop.offers.map((offer) => offer.item_id),
);
const contentItemIds = new Set(content.items.map((entry) => entry.id));
if (content.schema_version !== 6 || content.items.length !== 82 ||
    content.delve_shops?.length !== 2 ||
    !shopItemIds.every((itemId) => contentItemIds.has(itemId))) {
  throw new Error('M5 Delve shop item vocabulary is incomplete');
}

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
if (payloadContract.schema_version !== 60) throw new Error('WOS160 payload schema must be 60');
const payload = payloadContract.entries.find(
  (entry) => entry.id === 119 && entry.name === 'delve_buy',
);
if (!payload || payload.kind !== 'utf8_id_pair' || payload.min_byte_length !== 8 ||
    payload.max_byte_length !== 520 || payload.max_utf8_bytes !== 256 ||
    payload.encoding !== 'u32_le_utf8+u32_le_utf8' ||
    payload.source_shape?.method !== 'delveBuyShopItem') {
  throw new Error('delve-buy typed payload contract is missing or non-canonical');
}

const generatedPayload = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayload,
  /delveBuyCommandId\(required: bool\): uint[\s\S]*?return <uint>119/,
  'generated delve-buy payload metadata is missing');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must advance to WOS83');
requireText(world,
  /commandId == delveBuyCommand[\s\S]*?applyDelveShopBuyCommand/,
  'authoritative command routing does not reach the Delve shop');
requireText(world,
  /applyDelveShopBuyCommand\([\s\S]*?entityDead[\s\S]*?delveShopInRange[\s\S]*?delveShopOfferGateMet[\s\S]*?entityDelveMarks[\s\S]*?grantM5InventoryItem/,
  'authoritative Delve shop transaction is incomplete or reordered');
requireText(world,
  /pub delveShopCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)/,
  'Delve shop world-state regression is incomplete');
requireText(world,
  /pub selfTest\(\): int[\s\S]*?delveShopCommandStateTest\(\)/,
  'Delve shop regression is not wired into the authoritative world-state self-test');
for (const regression of [
  'unknownShop',
  'unknownItem',
  'oversizedShop',
  'oversizedItem',
  'trailingPayload',
  'legacyWos79',
]) {
  if (!world.includes(regression)) {
    throw new Error(`Delve shop world-state regression omits ${regression}`);
  }
}
for (const identity of ['collapsed_reliquary', 'reliquary_legs', 'drowned_litany']) {
  if (!world.includes(identity)) {
    throw new Error(`Delve shop world-state regression omits ${identity}`);
  }
}

const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativePayload,
  /pub struct DelveBuyCommandPayload[\s\S]*?delve_id: String[\s\S]*?item_id: String[\s\S]*?encode_utf8_id_pair[\s\S]*?decode_utf8_id_pair/,
  'native delve-buy payload is missing');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /BuyDelveShopItem \{[\s\S]*?delve_id: String[\s\S]*?item_id: String[\s\S]*?DelveBuyCommandPayload/,
  'native client delve-buy intent mapping is missing');
const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS118/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS118 in both runtime paths');
}
const nativeLib = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeLib,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const worldContract = read('contracts', 'world-state.md');
requireText(worldContract,
  /WOS82 mech chroma delta[\s\S]*?schema 49[\s\S]*?delve_buy[\s\S]*?12 units[\s\S]*?force-grant/,
  'world-state contract does not document the WOS160 Delve transaction');
const readme = read('README.md');
requireText(readme,
  /Command-payload schema 60 covers 157 of 165 source commands[\s\S]*?delve_buy[\s\S]*?remaining 0 source-shaped client\s+sends/,
  'README WOS160 protocol coverage summary drifted');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS160 command payload coverage totals drifted');
}

process.stdout.write(`WOS160 Delve-shop static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
