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
  const normalized = text.replace(/\r\n/g, "\n");
  const declaration = normalized.indexOf(signature);
  if (declaration < 0) throw new Error(`function declaration is missing: ${signature}`);
  const open = normalized.indexOf('{', declaration);
  if (open < 0) throw new Error(`function body is missing: ${signature}`);
  let depth = 0;
  for (let index = open; index < normalized.length; index += 1) {
    if (normalized[index] === '{') depth += 1;
    if (normalized[index] === '}') {
      depth -= 1;
      if (depth === 0) return normalized.slice(open + 1, index);
    }
  }
  throw new Error(`function body is unterminated: ${signature}`);
};

const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'unequip_mech_chroma':[\s\S]*?typeof msg\.chroma === 'string'[\s\S]*?unequipAccountMechChroma/,
  'source mech-chroma command admission drifted');
const sourceServerReducer = functionBody(sourceServer, 'private unequipAccountMechChroma(');
requireText(sourceServerReducer,
  /mechChromaSkinIndex[\s\S]*?mechChromaItemId[\s\S]*?mechChromaIds\.includes[\s\S]*?filter[\s\S]*?skinCatalog === 'mech'[\s\S]*?setPlayerSkin\(live\.pid, 0, 'class'\)[\s\S]*?addItem\(itemId, 1, session\.pid\)[\s\S]*?revokeAccountMechChroma/,
  'source account-wide mech-chroma transaction drifted');
const sourceSim = source('src/sim/sim.ts');
const sourceSimReducer = functionBody(sourceSim, 'unequipMechChroma(');
requireText(sourceSimReducer,
  /mechChromaIds\.includes[\s\S]*?filter[\s\S]*?skinCatalog === 'mech'[\s\S]*?setPlayerSkin\(meta\.entityId, 0, 'class'\)[\s\S]*?addItem\(itemId, 1/,
  'source offline mech-chroma transaction drifted');

const sourceSkins = source('src/sim/content/skins.ts');
const sourceChromas = [...sourceSkins.matchAll(
  /\{\s*id:\s*'([^']+)',\s*rank:\s*'(uncommon|rare|epic)'\s*\}/g,
)].map((match, skinIndex) => ({
  id: match[1],
  rank: match[2],
  skin_index: skinIndex,
  item_id: `${match[1]}_armor_plate`,
}));
if (sourceChromas.length !== 15) throw new Error('source MECH_CHROMAS order drifted');
const expectedRanks = [
  ...Array(8).fill('uncommon'),
  ...Array(3).fill('rare'),
  ...Array(4).fill('epic'),
];
if (sourceChromas.some((chroma, index) => chroma.rank !== expectedRanks[index])) {
  throw new Error('source MECH_CHROMAS rank order drifted');
}

const content = JSON.parse(read('contracts', 'm5_content.json'));
if (content.schema_version !== 6 || content.items?.length !== 82 ||
    JSON.stringify(content.mech_chromas) !== JSON.stringify(sourceChromas)) {
  throw new Error('generated mech-chroma id/rank/skin/item projection drifted');
}
for (const [index, chroma] of sourceChromas.entries()) {
  const item = content.items[67 + index];
  if (item?.id !== chroma.item_id || item.definition?.id !== chroma.item_id ||
      item.definition?.quality !== chroma.rank ||
      item.definition?.use?.type !== 'mechChroma' ||
      item.definition?.use?.chromaId !== chroma.id ||
      item.definition?.noVendorSell !== true ||
      item.definition?.noDiscard !== true ||
      item.definition?.noMarketList !== true) {
    throw new Error(`generated mech-chroma item projection drifted at ${index}`);
  }
}
const generatedContent = read('scripts', 'woc_game', 'src', 'generated', 'm5_content_catalog.zr');
requireText(functionBody(generatedContent, 'pub mechChromaIdUtf8Length('),
  /index == 0[\s\S]*?index == 14/,
  'generated Zr mech-chroma UTF-8 lengths are incomplete');
requireText(functionBody(generatedContent, 'pub mechChromaIdUtf8Byte('),
  /index == 0[\s\S]*?index == 14/,
  'generated Zr mech-chroma UTF-8 bytes are incomplete');
requireText(functionBody(generatedContent, 'pub mechChromaItemIndex('),
  /index == 0[\s\S]*?return 67[\s\S]*?index == 14[\s\S]*?return 81/,
  'generated Zr mech-chroma item mapping is incomplete');

const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const payload = payloads.entries.find((entry) => entry.id === 32);
if (payloads.schema_version !== 60 || payload?.name !== 'unequip_mech_chroma' ||
    payload.kind !== 'utf8_id' || payload.min_byte_length !== 4 ||
    payload.max_byte_length !== 260 || payload.max_utf8_bytes !== 256 ||
    payload.encoding !== 'u32_le_utf8' ||
    payload.source_shape?.method !== 'unequipMechChroma') {
  throw new Error('mech-chroma typed payload is missing or non-canonical');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const routeBody = functionBody(world, 'pub applyCommands(');
requireText(routeBody,
  /commandId == unequipMechChromaCommand[\s\S]*?applyOfflineUnequipMechChromaCommand/,
  'authoritative command routing does not reach mech-chroma unequip');
const payloadBody = functionBody(world, 'mechChromaPayloadShapeIsValid(');
requireText(payloadBody,
  /chromaLength > <uint>256[\s\S]*?payloadLength != chromaLength \+ <uint>4[\s\S]*?craftItemState\.utf8RangeValid/,
  'authoritative mech-chroma payload does not enforce canonical UTF-8');
const reducerBody = functionBody(
  world,
  'applyOfflineUnequipMechChromaCommand(\n    state: WorldState',
);
requireText(reducerBody,
  /offlineMechChromaIndexFromPayload[\s\S]*?offlineMechChromaOwned[\s\S]*?= false[\s\S]*?entitySkinCatalogs[\s\S]*?entitySkinIndices[\s\S]*?grantM5InventoryItem/,
  'authoritative mech-chroma transaction is incomplete');
const encoderBody = functionBody(
  world,
  'encodeStateVersion(state: WorldState, schemaVersion: uint)',
);
requireText(encoderBody,
  /schemaVersion >= <uint>81[\s\S]*?discoveredItemCount = schemaVersion == <uint>81 \?[\s\S]*?67[\s\S]*?offlineM5ItemsDiscovered[\s\S]*?schemaVersion >= <uint>82[\s\S]*?offlineMechChromaOwned/,
  'WOS81 historical encoder or WOS82 ownership encoder is missing');
const decoderBody = functionBody(world, 'decodeState(bytes: container.Array<uint>)');
requireText(decoderBody,
  /schemaVersion >= <uint>81[\s\S]*?discoveredItemCount = schemaVersion == <uint>81 \?[\s\S]*?67[\s\S]*?offlineM5ItemsDiscovered[\s\S]*?schemaVersion >= <uint>82[\s\S]*?offlineMechChromaOwned/,
  'WOS81 discovery migration or WOS82 ownership decoder is missing');
const regressionBody = functionBody(world, 'pub mechChromaCommandStateTest()');
for (const evidence of [
  'mappingIndex', 'unownedChroma', 'unknownChroma', 'deadChroma',
  'repeatedUnequip', 'inactiveChroma', 'protectedChroma', 'fullBagUnequip',
  'maximumChroma', 'oversizedChroma', 'trailingChroma', 'invalidUtf8Chroma',
  'legacyWos81', 'decodeState(encodeState(state))',
]) {
  if (!regressionBody.includes(evidence)) {
    throw new Error(`WOS162 regression omits ${evidence}`);
  }
}
const selfTestBody = functionBody(world, 'pub selfTest()');
if (!selfTestBody.includes('mechChromaCommandStateTest()')) {
  throw new Error('mech-chroma regression is not wired into selfTest');
}

const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativePayload,
  /pub struct UnequipMechChromaCommandPayload\s*\{\s*pub chroma_id: String,?\s*\}/,
  'native mech-chroma payload is missing');
const nativePayloadImpl = functionBody(
  nativePayload, 'impl UnequipMechChromaCommandPayload'
);
requireText(nativePayloadImpl,
  /pub fn encode\(&self\)[\s\S]*?encode_utf8_id\(UNEQUIP_MECH_CHROMA_COMMAND_ID, &self\.chroma_id\)[\s\S]*?pub fn decode\(bytes: &\[u8\]\)[\s\S]*?decode_utf8_id\(UNEQUIP_MECH_CHROMA_COMMAND_ID, bytes\)/,
  'native mech-chroma codec is not command-local');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /UnequipMechChroma\s*\{\s*chroma_id: String,?\s*\}/,
  'native client mech-chroma intent variant is missing');
const encodeIntentBody = functionBody(clientIntent, 'fn encode_intent(');
requireText(encodeIntentBody,
  /ClientGameplayIntent::UnequipMechChroma \{ chroma_id \} => \{[\s\S]*?UnequipMechChromaCommandPayload \{ chroma_id \}[\s\S]*?\.encode\(\)[\s\S]*?UNEQUIP_MECH_CHROMA_COMMAND_ID/,
  'native client mech-chroma intent mapping is missing or misrouted');
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
  /WOS82 mech chroma delta[\s\S]*?15 mech-chroma ownership markers[\s\S]*?67 to 82 fixed bytes[\s\S]*?schema 51[\s\S]*?WOS82 decodes that exact historical suffix/,
  'world-state contract does not document WOS82 and its WOS81 migration');
const contentContract = read('contracts', 'm5-content.md');
requireText(contentContract,
  /Schema 6[\s\S]*?82 item definitions[\s\S]*?15 source-ordered mech chromas[\s\S]*?noVendorSell[\s\S]*?noDiscard[\s\S]*?noMarketList/,
  'M5 content contract does not document the complete mech-chroma projection');
const readme = read('README.md');
requireText(readme,
  /WOS83[\s\S]*?Command-payload schema 60 covers 157 of 165 source commands[\s\S]*?unequip_mech_chroma[\s\S]*?remaining 0 source-shaped client\s+sends/,
  'README WOS162 identity or protocol coverage summary drifted');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS162 command payload coverage totals drifted');
}

process.stdout.write(`WOS162 mech-chroma static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
