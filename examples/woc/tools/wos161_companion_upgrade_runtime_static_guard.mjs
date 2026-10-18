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
const functionBody = (text, signature) => {
  const declaration = text.indexOf(signature);
  if (declaration < 0) throw new Error(`function declaration is missing: ${signature}`);
  const open = text.indexOf('{', declaration);
  if (open < 0) throw new Error(`function body is missing: ${signature}`);
  let depth = 0;
  for (let index = open; index < text.length; index += 1) {
    if (text[index] === '{') depth += 1;
    if (text[index] === '}') {
      depth -= 1;
      if (depth === 0) return text.slice(open + 1, index);
    }
  }
  throw new Error(`function body is unterminated: ${signature}`);
};

const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'companion_upgrade':[\s\S]*?typeof msg\.companionId !== 'string'[\s\S]*?e\.dead[\s\S]*?Object\.values\(DELVES\)[\s\S]*?autoCompanionId === msg\.companionId[\s\S]*?Math\.hypot[\s\S]*?> 12[\s\S]*?sim\.companionUpgrade\(msg\.companionId, pid\)/,
  'source companion-upgrade admission or range drifted');
const sourceRuns = source('src/sim/delves/runs.ts');
requireText(sourceRuns,
  /companionUpgrade\([\s\S]*?DELVE_COMPANIONS\[companionId\][\s\S]*?\?\? 1[\s\S]*?DELVE_COMPANION_MAX_RANK[\s\S]*?COMPANION_UPGRADE_COSTS\[next\][\s\S]*?delveMarks < cost\.marks[\s\S]*?copper < cost\.copper[\s\S]*?delveMarks -= cost\.marks[\s\S]*?copper -= cost\.copper[\s\S]*?companionUpgrades\[companionId\] = next[\s\S]*?markDeedsDirty/,
  'source companion-upgrade transaction ordering drifted');

const companion = JSON.parse(read('contracts', 'm7_delve_companion_content.json'));
if (companion.content?.max_rank !== 3 || companion.content?.companions?.length !== 2 ||
    companion.content.upgrade_costs?.[0]?.marks !== 3 ||
    companion.content.upgrade_costs?.[1]?.marks !== 5) {
  throw new Error('source-pinned M7 companion content is incomplete');
}
const content = JSON.parse(read('contracts', 'm5_content.json'));
if (content.schema_version !== 6 || content.items?.length !== 82 ||
    content.delve_shops?.[0]?.auto_companion_id !== 'companion_tessa' ||
    content.delve_shops?.[1]?.auto_companion_id !== 'companion_edda') {
  throw new Error('M5 companion-to-door projection is incomplete');
}
const generatedContent = read('scripts', 'woc_game', 'src', 'generated', 'm5_content_catalog.zr');
requireText(generatedContent,
  /delveShopCompanionId\(index: int\): string[\s\S]*?companion_tessa[\s\S]*?companion_edda/,
  'generated companion-to-door projection is missing');
const generatedCompanion = read(
  'scripts', 'woc_game', 'src', 'instances', 'delve_companion_content.zr',
);
requireText(generatedCompanion,
  /companionIdUtf8Length\(index: int, required: bool\)[\s\S]*?companionIdUtf8Byte\(index: int, byteIndex: int, required: bool\)/,
  'generated companion UTF-8 projection is missing');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
const payload = payloadContract.entries.find(
  (entry) => entry.id === 118 && entry.name === 'companion_upgrade',
);
if (payloadContract.schema_version !== 60 || !payload || payload.kind !== 'utf8_id' ||
    payload.min_byte_length !== 4 || payload.max_byte_length !== 260 ||
    payload.max_utf8_bytes !== 256 || payload.encoding !== 'u32_le_utf8' ||
    payload.source_shape?.method !== 'companionUpgrade') {
  throw new Error('companion-upgrade typed payload contract is missing or non-canonical');
}
const generatedPayload = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayload,
  /companionUpgradeCommandId\(required: bool\): uint[\s\S]*?return <uint>118/,
  'generated companion-upgrade payload metadata is missing');

const deedContract = JSON.parse(read('reference', 'current-head', 'deed_title_contract.json'));
if (deedContract.schema_version !== 2 ||
    !deedContract.deeds.some((entry) => entry.id === 'col_first_rare' &&
      entry.item_discovery_trigger?.kind === 'quality') ||
    !deedContract.deeds.some((entry) => entry.id === 'col_discovery_25' &&
      entry.item_discovery_trigger?.kind === 'items_discovered_count')) {
  throw new Error('generated item-discovery deed contract is incomplete');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must advance to WOS83');
requireText(world,
  /encodeState\(state: WorldState\)[\s\S]*?encodeStateVersion\(state, <uint>118\)/,
  'production world-state writer must select WOS118');
requireText(world,
  /commandId == companionUpgradeCommand[\s\S]*?applyDelveCompanionUpgradeCommand/,
  'authoritative command routing does not reach companion upgrade');
requireText(world,
  /applyDelveCompanionUpgradeCommand\([\s\S]*?entityDead[\s\S]*?delveShopInRange[\s\S]*?upgradeMarks[\s\S]*?upgradeCopper[\s\S]*?entityDelveMarks[\s\S]*?entityInventoryCopper[\s\S]*?entityDelveCompanionRanks[\s\S]*?applyCompanionUpgradeDeeds/,
  'authoritative companion-upgrade transaction is incomplete or reordered');
const grantInventoryBody = functionBody(world, 'pub grantM5InventoryItem(');
requireText(grantInventoryBody,
  /markOfflineM5ItemDiscovered\(state, entityIndex, itemCode\)/,
  'persistent item-discovery side effect is missing from the addItem projection');
const decodeBody = functionBody(world, 'decodeState(bytes: container.Array<uint>)');
requireText(decodeBody,
  /schemaVersion >= <uint>81[\s\S]*?entityDelveCompanionRanks[\s\S]*?schemaVersion >= <uint>81[\s\S]*?offlineM5ItemsDiscovered/,
  'WOS81 companion ranks or item-discovery ledger are not decoded');
requireText(decodeBody,
  /seedOfflineM5ItemDiscoveryFromHoldings\(state\)[\s\S]*?evaluateOfflineItemDiscoveryDeeds\(state\)[\s\S]*?evaluateOfflineCompanionUpgradeDeeds\(state\)/,
  'WOS81 decode does not perform join-time retro deed closure');
requireText(world,
  /initialBytes\.length != 501[\s\S]*?first\.length != 481/,
  'WOS83 empty-state codec lengths are stale');
requireText(world,
  /pub companionUpgradeCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)/,
  'companion-upgrade world-state regression is missing');
requireText(world,
  /pub selfTest\(\): int[\s\S]*?companionUpgradeCommandStateTest\(\)/,
  'companion-upgrade regression is not wired into the world-state self-test');
for (const evidence of [
  'companion_tessa', 'companion_edda', 'dlv_companion_max', 'dlv_companions_both',
  'legacyWos80', 'unknownCompanion', 'maximumCompanion', 'oversizedCompanion',
  'trailingCompanionPayload', 'retroClosure', 'retroRareItemCode',
]) {
  if (!world.includes(evidence)) throw new Error(`WOS161 regression omits ${evidence}`);
}

const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativePayload,
  /pub struct CompanionUpgradeCommandPayload[\s\S]*?companion_id: String[\s\S]*?encode_utf8_id[\s\S]*?decode_utf8_id/,
  'native companion-upgrade payload is missing');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /UpgradeDelveCompanion \{[\s\S]*?companion_id: String[\s\S]*?CompanionUpgradeCommandPayload/,
  'native client companion-upgrade intent mapping is missing');
const nativeLib = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeLib,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS118/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS118 in both runtime paths');
}
const worldContract = read('contracts', 'world-state.md');
requireText(worldContract,
  /WOS82 mech chroma delta[\s\S]*?schema 50[\s\S]*?companion_upgrade[\s\S]*?item-discovery/,
  'world-state contract does not document WOS161');
const readme = read('README.md');
requireText(readme,
  /Command-payload schema 60 covers 157 of 165 source commands[\s\S]*?companion_upgrade[\s\S]*?remaining 0 source-shaped client\s+sends/,
  'README WOS161 protocol coverage summary drifted');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS161 command payload coverage totals drifted');
}

process.stdout.write(`WOS161 companion-upgrade static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
