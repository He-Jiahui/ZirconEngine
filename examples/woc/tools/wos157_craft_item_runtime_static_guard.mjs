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
  /case 'craft_item':[\s\S]*?typeof msg\.recipe === 'string'[\s\S]*?sim\.craftItem\(msg\.recipe, pid\)/,
  'source craft_item server admission drifted');
const sourceCraft = source('src/sim/professions/crafting.ts');
requireText(sourceCraft,
  /recipe\.requiresHubStation[\s\S]*?meetsComboRequirement[\s\S]*?isRecipeKnown[\s\S]*?hasRecipeMaterials[\s\S]*?withinCraftThrottle/,
  'source craft gate order drifted');
requireText(sourceCraft,
  /Math\.ceil\(recipe\.itemLevelBudget[\s\S]*?hasSignedInstance[\s\S]*?ctx\.removeItem[\s\S]*?const procRoll = ctx\.rng\.next\(\)[\s\S]*?ctx\.addItem[\s\S]*?gainCraftSkill[\s\S]*?meta\.craftThrottle\.count \+= 1[\s\S]*?craftActionXp/,
  'source craft success mutation/RNG order drifted');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
if (payloadContract.schema_version !== 60) {
  throw new Error('WOS157 command payload schema must be 60');
}
const payload = payloadContract.entries.find(
  (entry) => entry.id === 30 && entry.name === 'craft_item',
);
if (!payload || payload.kind !== 'utf8_id' || payload.min_byte_length !== 4 ||
    payload.max_byte_length !== 260 || payload.max_utf8_bytes !== 256 ||
    payload.source_shape?.method !== 'craftItem' ||
    JSON.stringify(payload.source_shape?.fields) !== JSON.stringify(['recipe'])) {
  throw new Error('craft_item typed payload contract is missing or non-canonical');
}

const generatedPayloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayloads,
  /craftItemCommandId\(required: bool\): uint[\s\S]*?return <uint>30[\s\S]*?if \(id == <uint>30\) \{ return 4; \}/,
  'generated craft_item id or payload kind is missing');
const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativePayload,
  /pub struct CraftItemCommandPayload[\s\S]*?recipe_id: String[\s\S]*?encode_utf8_id\(CRAFT_ITEM_COMMAND_ID[\s\S]*?decode_utf8_id\(CRAFT_ITEM_COMMAND_ID/,
  'native craft-item payload round trip is missing');
const nativeLib = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeLib, /CraftItemCommandPayload/,
  'native protocol does not export CraftItemCommandPayload');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /CraftItem \{[\s\S]*?recipe_id: String[\s\S]*?CraftItemCommandPayload \{ recipe_id \}[\s\S]*?CRAFT_ITEM_COMMAND_ID/,
  'native client craft-item intent mapping is missing');

const parity = read('scripts', 'woc_game', 'src', 'generated', 'professions_craft_parity_contract.zr');
requireText(parity,
  /recipeIdUtf8Length\(index: int\)[\s\S]*?recipeIdUtf8Byte\(index: int, byteIndex: int\)[\s\S]*?recipe_minor_healing_potion[\s\S]*?recipe_eastbrook_ritual_vestments/,
  'source-generated craft recipe byte accessors are missing');
const adapter = read('scripts', 'woc_game', 'src', 'progression', 'craft_item_state.zr');
requireText(adapter,
  /first >= <uint>194[\s\S]*?first >= <uint>240[\s\S]*?first <= <uint>244[\s\S]*?recipeCodeFromPayload/,
  'craft-item adapter does not enforce canonical UTF-8');
requireText(adapter,
  /reasonUnsupportedInstanceOutput[\s\S]*?throttleWindowMicros[\s\S]*?goldFeeCopper[\s\S]*?reagentItemCode[\s\S]*?craftExperience/,
  'craft-item adapter source constants are incomplete');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must retain the current WOS83 identity');
requireText(world,
  /commandId == craftItemCommand[\s\S]*?applyOfflineCraftItemCommand/,
  'authoritative command routing does not reach craft_item');
const reducerStart = world.indexOf('\napplyOfflineCraftItemCommand(');
const reducerEnd = world.indexOf('\napplyOfflineHarvestNodeCommand(', reducerStart);
if (reducerStart < 0 || reducerEnd < 0) throw new Error('craft-item reducer boundary is missing');
const reducer = world.slice(reducerStart, reducerEnd);
requireText(reducer,
  /recipeCodeFromPayload[\s\S]*?reasonUnknownRecipe[\s\S]*?reasonUnsupportedInstanceOutput[\s\S]*?offlineCraftHasMaterials[\s\S]*?reasonInsufficientMaterials[\s\S]*?reasonThrottled[\s\S]*?entityInventoryCopper[\s\S]*?consumeOfflineCraftMaterials[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?grantM5InventoryItem[\s\S]*?offlineCraftAlchemySkill[\s\S]*?grantOfflineQuestExperience/,
  'authoritative craft-item transaction order is incomplete');
if (reducer.includes('entityDead')) {
  throw new Error('craft-item reducer must retain the source lack of a dead-state gate');
}
requireText(world,
  /if \(schemaVersion >= <uint>77\) \{[\s\S]*?offlineCraftAlchemySkill[\s\S]*?offlineCraftResultRecipeBytes\.add\(reader\.byte[\s\S]*?offlineCraftResultMasterwork/,
  'WOS77 craft-item codec migration gate is missing');
requireText(world,
  /pub offlineCraftItemCommandStateTest\(\): int[\s\S]*?reasonInsufficientMaterials[\s\S]*?reasonUnsupportedInstanceOutput[\s\S]*?entityDead\[0\] = true[\s\S]*?reasonThrottled[\s\S]*?invalidUtf8[\s\S]*?decodeState\(encodeState\(state\)\)/,
  'craft-item authoritative regression is incomplete');
requireText(world,
  /if \(offlineCraftItemCommandStateTest\(\) != 1\) \{[\s\S]*?return -151;/,
  'world selfTest must execute WOS157 coverage');

const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS118/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS118 in both runtime paths');
}
requireText(nativeLib,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const worldContract = read('contracts', 'world-state.md');
requireText(worldContract,
  /WOS82 mech chroma delta[\s\S]*?WOS77 appends the bounded offline crafting projection[\s\S]*?internal reason\s+`7`[\s\S]*?remaining 17 recipes are complete/,
  'world-state contract does not document the bounded WOS77 craft delta');
const readme = read('README.md');
requireText(readme,
  /The generated command coverage now contains 157 typed entries[\s\S]*?eight dispatch commands remain[\s\S]*?unmapped/,
  'README command-payload coverage summary drifted');
requireText(readme,
  /Command-payload schema 60 covers 157 of 165 source commands[\s\S]*?remaining 0 source-shaped client\s+sends/,
  'README schema 49 coverage summary drifted');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS157 command payload coverage totals drifted');
}

process.stdout.write(`WOS157 craft-item static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
