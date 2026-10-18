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

const sourceVendor = source('src/sim/instances/heroic_vendor.ts');
requireText(sourceVendor,
  /HEROIC_VENDOR_STOCK\.find[\s\S]*?ITEMS\[itemId\][\s\S]*?p\.dead[\s\S]*?!heroicVendorInRange[\s\S]*?countItem\(HEROIC_MARK_ITEM_ID[\s\S]*?!ctx\.canAddItem[\s\S]*?removeItem\(HEROIC_MARK_ITEM_ID[\s\S]*?addItem\(itemId[\s\S]*?type: 'vendor', action: 'buy'/,
  'source heroic vendor transaction ordering drifted');
requireText(sourceVendor, /dist2d\(p\.pos, e\.pos\) <= INTERACT_RANGE \+ 2/,
  'source heroic quartermaster range drifted');
const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'heroic_buy':[\s\S]*?typeof msg\.itemId === 'string'[\s\S]*?sim\.buyHeroicVendorItem\(msg\.itemId, pid\)/,
  'source server heroic-buy admission drifted');
const sourceStock = source('src/sim/content/heroic_vendor.ts');
if ((sourceStock.match(/\{ itemId: '[^']+', marks: (?:12|16) \}/g) ?? []).length !== 10) {
  throw new Error('source heroic vendor stock drifted');
}
const sourceItems = source('src/sim/content/items.ts');
requireText(sourceItems,
  /heroic_mark:[\s\S]*?kind: 'tool'[\s\S]*?stackSize: 20[\s\S]*?soulbound: true[\s\S]*?noDiscard: true/,
  'source Heroic Mark definition drifted');

const content = JSON.parse(read('contracts', 'm5_content.json'));
const itemIds = new Set(content.items.map((entry) => entry.id));
const expectedStock = [
  'seal_of_the_nine_oaths',
  'nielas_coldlight_band',
  'sutils_gambit',
  'oath_of_the_round_table',
  'zyzzs_deathless_signet',
  'architects_cornerstone',
  'yumis_keepsake_locket',
  'zense_meridian',
  'swiftfang_talisman',
  'medallion_of_endless_profit',
];
if (content.schema_version !== 6 || content.items.length !== 82 ||
    content.heroic_vendor?.npc_id !== 'heroic_quartermaster' ||
    content.heroic_vendor?.currency_item_id !== 'heroic_mark' ||
    content.heroic_vendor?.stock?.length !== 10 ||
    !['heroic_mark', ...expectedStock].every((id) => itemIds.has(id)) ||
    content.items.slice(38, 49).map((entry) => entry.id).join('|') !==
      ['heroic_mark', ...expectedStock].join('|')) {
  throw new Error('source-pinned heroic vendor content projection is incomplete');
}
if (content.heroic_vendor.stock.map((entry) => entry.item_id).join('|') !==
    expectedStock.join('|') ||
    content.heroic_vendor.stock.some((entry, index) => entry.marks !== (index < 6 ? 12 : 16))) {
  throw new Error('heroic vendor stock order or prices drifted');
}

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
if (payloadContract.schema_version !== 60) throw new Error('WOS159 payload schema must be 60');
const payload = payloadContract.entries.find(
  (entry) => entry.id === 142 && entry.name === 'heroic_buy',
);
if (!payload || payload.kind !== 'utf8_id' || payload.min_byte_length !== 4 ||
    payload.max_byte_length !== 260 || payload.max_utf8_bytes !== 256 ||
    payload.source_shape?.method !== 'buyHeroicVendorItem') {
  throw new Error('heroic-buy typed payload contract is missing or non-canonical');
}

const generatedPayload = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayload,
  /heroicBuyCommandId\(required: bool\): uint[\s\S]*?return <uint>142/,
  'generated heroic-buy payload metadata is missing');
const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativePayload,
  /pub struct HeroicBuyCommandPayload[\s\S]*?encode_utf8_id\(HEROIC_BUY_COMMAND_ID[\s\S]*?decode_utf8_id\(HEROIC_BUY_COMMAND_ID/,
  'native heroic-buy payload is missing');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /BuyHeroicVendorItem \{[\s\S]*?item_id: String[\s\S]*?HeroicBuyCommandPayload \{ item_id \}[\s\S]*?HEROIC_BUY_COMMAND_ID/,
  'native client heroic-buy intent mapping is missing');

const inventoryRules = read(
  'scripts', 'woc_game', 'src', 'progression', 'm5_inventory_rules.zr',
);
requireText(inventoryRules,
  /stackLimitForItemIndex[\s\S]*?catalog\.metric\("item", itemIndex, "stackSize"\)[\s\S]*?heroic_mark[\s\S]*?!= 20/,
  'source stackSize projection or Heroic Mark regression is missing');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must advance to WOS83');
requireText(world,
  /commandId == heroicBuyCommand[\s\S]*?applyHeroicVendorBuyCommand/,
  'authoritative command routing does not reach heroic vendor');
requireText(world,
  /applyHeroicVendorBuyCommand\([\s\S]*?entityDead[\s\S]*?heroicVendorInRange[\s\S]*?heroicVendorOfferMarks[\s\S]*?m5InventoryItemCount[\s\S]*?m5InventoryCanAddItem[\s\S]*?removeM5InventoryItem[\s\S]*?grantM5InventoryItem/,
  'authoritative heroic vendor transaction is incomplete or reordered');
requireText(world,
  /pub heroicVendorCommandStateTest\(\): int[\s\S]*?heroic_mark[\s\S]*?seal_of_the_nine_oaths[\s\S]*?fullBagState[\s\S]*?m5InventoryCapacity[\s\S]*?fullBagBuy[\s\S]*?decodeState\(encodeState\(state\)\)/,
  'heroic vendor world-state regression is incomplete');

const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS118/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS118 in both runtime paths');
}
const nativeLib = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeLib,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS159 command payload coverage totals drifted');
}
const worldContract = read('contracts', 'world-state.md');
requireText(worldContract,
  /WOS82 mech chroma delta[\s\S]*?schema 48[\s\S]*?Heroic Mark[\s\S]*?quartermaster/,
  'world-state contract does not document the scoped WOS159 delta');

process.stdout.write(`WOS159 heroic-vendor static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
