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
  /case 'harvest_node':[\s\S]*?typeof msg\.node === 'string'[\s\S]*?sim\.harvestNode\(msg\.node, pid\)/,
  'source harvest_node server admission drifted');
const sourceGathering = source('src/sim/professions/gathering.ts');
requireText(sourceGathering,
  /harvestNode\(ctx: SimContext, nodeId: string, pid\?: number\)[\s\S]*?INTERACT_RANGE[\s\S]*?canAddItem[\s\S]*?resolveHarvest[\s\S]*?addItem[\s\S]*?grantXp[\s\S]*?gatherResult/,
  'source harvest-node transaction drifted');
requireText(sourceGathering,
  /nodeHarvestReadyAt\[node\.id\] = now \+ entry\.respawnSeconds;[\s\S]*?rollMaterialRarity[\s\S]*?queueGatheringGrant/,
  'source node readiness, rarity, or proficiency sequencing drifted');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 60) throw new Error('WOS154 command payload schema must be 60');
const harvest = contract.entries.find((entry) => entry.id === 29 && entry.name === 'harvest_node');
if (!harvest || harvest.kind !== 'utf8_id' || harvest.min_byte_length !== 4 ||
    harvest.max_byte_length !== 260) {
  throw new Error('harvest_node typed payload contract is missing or non-canonical');
}

const generatedPayloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayloads, /harvestNodeCommandId\(required: bool\): uint[\s\S]*?return <uint>29/,
  'generated harvest_node command id is missing');

const nativeProtocol = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativeProtocol,
  /pub struct HarvestNodeCommandPayload[\s\S]*?node_id: String[\s\S]*?encode_utf8_id[\s\S]*?decode_utf8_id/,
  'native harvest-node payload type is missing');
const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /HarvestNode \{[\s\S]*?node_id: String[\s\S]*?HarvestNodeCommandPayload[\s\S]*?HARVEST_NODE_COMMAND_ID/,
  'native client harvest-node intent mapping is missing');

const m5 = JSON.parse(read('contracts', 'm5_content.json'));
const itemIds = m5.items.map((entry) => entry.id);
for (const [index, id] of ['bone_fragments', 'linen_scrap', 'spider_leg'].entries()) {
  if (itemIds[35 + index] !== id) throw new Error(`${id} must retain stable item code ${36 + index}`);
}
const m5Catalog = read('scripts', 'woc_game', 'src', 'generated', 'm5_content_catalog.zr');
for (const [code, id] of [[35, 'bone_fragments'], [36, 'linen_scrap'], [37, 'spider_leg']]) {
  requireText(m5Catalog, new RegExp(`if \\(id == "${id}"\\) \\{ return ${code}; \\}`),
    `${id} generated catalog index is missing`);
}

const gatheringContract = JSON.parse(read('reference', 'current-head', 'gathering_contract.json'));
if (gatheringContract.interact_range !== 5) throw new Error('gathering interact range must remain source-pinned to 5');
const generatedGathering = read('scripts', 'woc_game', 'src', 'generated', 'gathering_contract.zr');
requireText(generatedGathering, /pub interactRange\(\): float \{ return 5\.0; \}/,
  'generated gathering interact range is missing');
requireText(generatedGathering,
  /pub nodeIdUtf8Length\(index: int\): int[\s\S]*?pub nodeIdUtf8Byte\(index: int, byteIndex: int\): uint/,
  'generated gathering node UTF-8 accessors are missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /commandId == harvestNodeCommand[\s\S]*?applyOfflineHarvestNodeCommand/,
  'authoritative command routing does not reach harvest_node');
requireText(world,
  /applyOfflineHarvestNodeCommand[\s\S]*?offlineGatherNodeReadyAtMicros[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?offlinePendingGatheringGains[\s\S]*?grantM5InventoryItem[\s\S]*?offlineGatherVisitedMask[\s\S]*?offlineGatherResultSequence/,
  'authoritative harvest-node reducer is incomplete');
requireText(world,
  /offlineGatheringStateIsValid[\s\S]*?resultNodeIndex[\s\S]*?professionCode\(resultNodeIndex\)[\s\S]*?itemCode\(resultNodeIndex\)/,
  'gather-result snapshot validation does not bind node, profession and material');
requireText(world,
  /pub offlineHarvestNodeCommandStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?drainOfflineGatheringGains[\s\S]*?encodeState/,
  'harvest-node authoritative regression is missing');
requireText(world, /pub fixedTick[\s\S]*?advanceState[\s\S]*?drainOfflineGatheringGains/,
  'fixed tick does not settle queued gathering proficiency');
requireText(world, /if \(offlineHarvestNodeCommandStateTest\(\) != 1\) \{[\s\S]*?return -148;/,
  'world selfTest must execute WOS154 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0) {
  throw new Error('WOS154 command payload coverage totals drifted');
}

process.stdout.write(`WOS154 harvest-node static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
