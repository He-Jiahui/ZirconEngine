import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = path.resolve(root, '..', '..');
const sourceRoot = path.resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const normalizeLineEndings = (text) => text.replace(/\r\n?/g, '\n');
const read = (...parts) => normalizeLineEndings(
  fs.readFileSync(path.join(root, ...parts), 'utf8'),
);
const source = (file) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${file}`], { encoding: 'utf8' },
).replace(/\r\n?/g, '\n');
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};
const functionBody = (text, signature) => {
  // Generated Zr sources can carry mixed LF/CRLF endings after a Windows checkout;
  // semantic owner checks must not depend on the file's newline representation.
  text = normalizeLineEndings(text);
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

const sourceRules = source('src/sim/content/weapon_skin_rules.ts');
for (const evidence of [
  "if (cls === 'hunter') return ['crossbow', 'bow'];",
  "if (def.weaponType === 'bow') delete next.crossbow;",
  "else if (def.weaponType === 'crossbow') delete next.bow;",
]) {
  if (!sourceRules.includes(evidence)) throw new Error(`source weapon-skin rule drifted: ${evidence}`);
}
const sourceSim = source('src/sim/sim.ts');
requireText(sourceSim,
  /setWeaponSkin\([\s\S]*?weaponSkinTypeMatches[\s\S]*?weaponSkinLoadout/,
  'source simulation weapon-skin reducer drifted');
const sourceServer = source('server/game.ts');
for (const evidence of [
  'if (!current.weaponSkinIds.includes(skinId)) return;',
  'if (!current.weaponSkinLoadout[wtype]) return;',
  'this.updateLiveAccountCosmetics(session.accountId, { ...current, weaponSkinLoadout });',
  'if (e.weaponSkinId) out.wsk = e.weaponSkinId;',
]) {
  if (!sourceServer.includes(evidence)) throw new Error(`source account weapon-skin rule drifted: ${evidence}`);
}

const contract = JSON.parse(read('reference', 'current-head', 'weapon_skin_contract.json'));
const expectedTypes = ['sword', 'axe', 'mace', 'dagger', 'staff', 'wand', 'bow', 'crossbow'];
if (contract.schema_version !== 1 || contract.source_commit !== SOURCE_COMMIT ||
    JSON.stringify(contract.weapon_skin_types) !== JSON.stringify(expectedTypes) ||
    contract.weapon_skins?.length !== 29 || contract.item_weapon_types?.length !== 92 ||
    !contract.item_weapon_types.some((entry) => entry.weapon_type === 'polearm')) {
  throw new Error('generated weapon-skin contract is incomplete');
}
const generatedContract = read(
  'scripts', 'woc_game', 'src', 'generated', 'weapon_skin_contract.zr',
);
for (const [signature, pattern] of [
  ['pub weaponSkinId(', /index == 0[\s\S]*?index == 28/],
  ['pub weaponSkinTypeCode(', /index == 0[\s\S]*?index == 28/],
  ['pub weaponSkinCodeForId(', /guildmark_arming_sword[\s\S]*?meteorlatch_crossbow/],
  ['pub itemWeaponTypeCodeForId(', /worn_sword[\s\S]*?drowned_choir_fang[\s\S]*?gnarled_staff/],
  ['pub weaponSkinIdUtf8Byte(', /index == 0[\s\S]*?index == 28/],
]) {
  requireText(functionBody(generatedContract, signature), pattern,
    `generated weapon-skin accessor is incomplete: ${signature}`);
}

const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const payload = payloads.entries.find((entry) => entry.id === 34);
if (payloads.schema_version !== 60 || payloads.entries.length !== 157 ||
    payload?.name !== 'change_weapon_skin' || payload.kind !== 'weapon_skin_change' ||
    payload.min_byte_length !== 2 || payload.max_byte_length !== 261 ||
    payload.max_utf8_bytes !== 256 || payload.source_shape?.method !== 'changeWeaponSkin') {
  throw new Error('weapon-skin command payload is missing or non-canonical');
}
const generatedPayload = read(
  'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr',
);
requireText(generatedPayload,
  /changeWeaponSkinCommandId\(required: bool\): uint[\s\S]*?return <uint>34/,
  'generated Zr weapon-skin payload metadata is missing');

const bootstrapContract = JSON.parse(read('contracts', 'woc.contracts.json'));
const bootstrapFields = bootstrapContract.contracts.find(
  (entry) => entry.name === 'OfflineSessionBootstrap',
)?.fields;
if (!bootstrapFields ||
    bootstrapFields.find((field) => field.name === 'weapon_skin_owned')?.max_length !== 29 ||
    bootstrapFields.find((field) => field.name === 'weapon_skin_loadout_codes')?.max_length !== 8 ||
    bootstrapContract.contracts.find((entry) => entry.name === 'FixedTickInput')
      ?.fields.find((field) => field.name === 'offline_bootstrap')?.max_length !== 66) {
  throw new Error('offline bootstrap weapon-skin contract fields are missing');
}
const generatedContracts = read('scripts', 'woc_game', 'src', 'generated', 'contracts.zr');
requireText(generatedContracts,
  /fieldOfflineSessionBootstrapWeaponSkinOwnedMaxLength[\s\S]*?return 29[\s\S]*?fieldOfflineSessionBootstrapWeaponSkinLoadoutCodesMaxLength[\s\S]*?return 8/,
  'generated offline bootstrap weapon-skin bounds are missing');
const nativeBootstrap = read('native', 'crates', 'woc_protocol', 'src', 'payload.rs');
requireText(nativeBootstrap,
  /weapon_skin_account\.owned[\s\S]*?weapon_skin_account\.loadout_codes[\s\S]*?OfflineWeaponSkinAccount[\s\S]*?weapon_skin_code_matches_loadout_type[\s\S]*?loadout code \{code\} does not match loadout type/,
  'native bootstrap does not preserve and validate weapon-skin account state');
const nativeWeaponSkinContract = read(
  'native', 'crates', 'woc_protocol', 'src', 'weapon_skin_contract.rs',
);
requireText(nativeWeaponSkinContract,
  /WEAPON_SKIN_COUNT: usize = 29[\s\S]*?WEAPON_SKIN_TYPE_COUNT: usize = 8[\s\S]*?WEAPON_SKIN_TYPE_BY_CODE[\s\S]*?weapon_skin_code_matches_loadout_type/,
  'native weapon-skin type mapping is missing from the generated contract');
const nativeProtocolTests = read('native', 'crates', 'woc_protocol', 'tests', 'protocol.rs');
requireText(nativeProtocolTests,
  /OFFLINE_WEAPON_SKIN_COUNT[\s\S]*?mismatched_weapon_skin_type[\s\S]*?owned\[1\] = true[\s\S]*?loadout_codes: \[2, 0, 0, 0, 0, 0, 0, 0\][\s\S]*?tampered_weapon_skin_type[\s\S]*?OfflineSessionBootstrap::decode_payload/,
  'native bootstrap lacks the owned-but-wrong-type encode/decode rejection regression');
const nativeBootstrapIdentity = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeBootstrapIdentity,
  /OFFLINE_SESSION_BOOTSTRAP_VERSION: u16 = 2/,
  'native bootstrap version is not v2');
const generatedNativeContracts = read('native', 'crates', 'woc_protocol', 'src', 'generated.rs');
requireText(generatedNativeContracts,
  /SCHEMA_FINGERPRINT_HEX[\s\S]*?7e69e47677529a0cbe4352a5f555cdddb5e8586fed885cca5612abaa93752719[\s\S]*?OFFLINE_BOOTSTRAP_MAX_LENGTH: u64 = 66/,
  'generated native bootstrap contract projection is stale');

const weaponState = read(
  'scripts', 'woc_game', 'src', 'progression', 'weapon_skin_state.zr',
);
requireText(functionBody(weaponState, 'pub payloadShapeIsValid('),
  /mode == <uint>0[\s\S]*?payloadLength == <uint>2[\s\S]*?mode != <uint>1[\s\S]*?skinLength > <uint>256[\s\S]*?craftItemState\.utf8RangeValid/,
  'authoritative weapon-skin payload shape is not strict');
requireText(functionBody(weaponState, 'pub activeSkinCode('),
  /isHunter[\s\S]*?loadoutCodes\[7\][\s\S]*?loadoutCodes\[6\]/,
  'Hunter crossbow-before-bow resolution is missing');
requireText(functionBody(weaponState, 'pub loadoutStateIsValid('),
  /weaponSkinTypeCode[\s\S]*?owned/,
  'weapon-skin loadout does not enforce type and ownership');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(functionBody(world, 'decodeOfflineBootstrap(bytes: container.Array<uint>)'),
  /WeaponSkinOwnedMaxLength[\s\S]*?WeaponSkinLoadoutCodesMaxLength[\s\S]*?launchVersion != <uint>2[\s\S]*?loadoutStateIsValid/,
  'wire bootstrap v2 does not decode and validate weapon-skin account state');
requireText(functionBody(world, 'applyOfflineBootstrap(state: WorldState'),
  /offlineWeaponSkinOwned[\s\S]*?bootstrap\.weaponSkinOwned[\s\S]*?offlineWeaponSkinLoadoutCodes[\s\S]*?bootstrap\.weaponSkinLoadoutCodes/,
  'offline bootstrap does not hydrate the world weapon-skin account state');
const routeBody = functionBody(world, 'pub applyCommands(');
requireText(routeBody,
  /changeWeaponSkinCommandId[\s\S]*?commandId == changeWeaponSkinCommand[\s\S]*?applyOfflineWeaponSkinCommand/,
  'authoritative command routing does not reach weapon skins');
const reducerBody = functionBody(
  world,
  'applyOfflineWeaponSkinCommand(\n    state: WorldState',
);
for (const evidence of [
  'payloadShapeIsValid', 'detachTypeCodeFromPayload', 'skinIndexFromPayload',
  'offlineWeaponSkinOwned', 'typeMatchesPlayer', 'typeCode == <uint>7',
  'typeCode == <uint>8', 'refreshAllOfflineWeaponSkins',
]) {
  if (!reducerBody.includes(evidence)) throw new Error(`weapon-skin reducer omits ${evidence}`);
}
if (/entityDead|rng(State|Draws|Digest)|nextRandom/.test(reducerBody)) {
  throw new Error('weapon-skin reducer gained a dead-state or RNG gate');
}
requireText(functionBody(world, 'offlineWeaponSkinMainhandItemIndex('),
  /emptyMainhandStoredCode[\s\S]*?startingMainhandStoredCode[\s\S]*?storedItemCodeToIndex/,
  'weapon-skin mainhand resolution omits empty or starting equipment');
requireText(functionBody(world, 'setM5BaselineStatProfile(state: WorldState'),
  /refreshOfflineWeaponSkinForEntity/,
  'baseline profile changes do not refresh weapon skins');
requireText(functionBody(world, 'pub setM5EquipmentItemCode('),
  /recomputeBaselineFormDerivedStats[\s\S]*?refreshOfflineWeaponSkinForEntity/,
  'equipment changes do not refresh weapon skins');
const encoderBody = functionBody(
  world,
  'encodeStateVersion(state: WorldState, schemaVersion: uint)',
);
requireText(encoderBody,
  /schemaVersion >= <uint>83[\s\S]*?offlineWeaponSkinOwned[\s\S]*?offlineWeaponSkinLoadoutCodes/,
  'WOS83 weapon-skin encoder tail is missing');
const decoderBody = functionBody(world, 'decodeState(bytes: container.Array<uint>)');
requireText(decoderBody,
  /schemaVersion != <uint>83[\s\S]*?schemaVersion >= <uint>83[\s\S]*?offlineWeaponSkinOwned[\s\S]*?offlineWeaponSkinLoadoutCodes[\s\S]*?refreshAllOfflineWeaponSkins/,
  'WOS83 weapon-skin decoder or WOS82 migration is missing');
requireText(world,
  /encodeState\(state: WorldState\)[\s\S]*?encodeStateVersion\(state, <uint>118\)/,
  'production world-state writer must select the current WOS118 schema');
requireText(world,
  /initialBytes\.length != 501[\s\S]*?first\.length != 481/,
  'current WOS118 empty-state codec lengths are stale');
const regressionBody = functionBody(world, 'pub weaponSkinCommandStateTest()');
for (const evidence of [
  'catalogIndex', 'bootstrap.weaponSkinOwned', 'applyOfflineBootstrap', 'deadApply',
  'unownedApply', 'mismatchedApply', 'repeatedDetach',
  'emptyMainhandStoredCode', 'applyBow', 'applyCrossbow', 'detachCrossbow',
  'legacyWos82', 'maximumPayload', 'oversizedPayload', 'invalidUtf8Payload',
  'rngDraws', 'decodeState(encodeState(state))',
]) {
  if (!regressionBody.includes(evidence)) throw new Error(`WOS163 regression omits ${evidence}`);
}
if (!functionBody(world, 'pub selfTest()').includes('weaponSkinCommandStateTest()')) {
  throw new Error('weapon-skin regression is not wired into selfTest');
}

const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'weapon_skin_payload.rs');
requireText(nativePayload,
  /pub enum WeaponSkinType[\s\S]*?Sword[\s\S]*?Crossbow[\s\S]*?pub enum WeaponSkinChange[\s\S]*?Apply[\s\S]*?Detach/,
  'native weapon-skin typed union is missing');
requireText(functionBody(nativePayload, 'impl ChangeWeaponSkinCommandPayload'),
  /pub fn encode\(&self\)[\s\S]*?CHANGE_WEAPON_SKIN_COMMAND_ID[\s\S]*?pub fn decode\(bytes: &\[u8\]\)/,
  'native weapon-skin codec is not command-local');
requireText(functionBody(nativePayload, 'pub(crate) fn validate_weapon_skin_payload('),
  /DETACH_MODE[\s\S]*?APPLY_MODE[\s\S]*?InvalidWeaponSkinMode/,
  'native weapon-skin validator is incomplete');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /ChangeWeaponSkin \{[\s\S]*?change: WeaponSkinChange/,
  'native client weapon-skin intent variant is missing');
requireText(functionBody(clientIntent, 'fn encode_intent('),
  /ClientGameplayIntent::ChangeWeaponSkin \{ change \}[\s\S]*?ChangeWeaponSkinCommandPayload \{ change \}[\s\S]*?CHANGE_WEAPON_SKIN_COMMAND_ID/,
  'native client weapon-skin intent mapping is missing or misrouted');
const offlineSession = read(
  'native', 'apps', 'woc_client', 'src', 'shell', 'offline_session.rs',
);
requireText(offlineSession,
  /OfflineWeaponSkinAccount[\s\S]*?with_weapon_skin_account[\s\S]*?set_weapon_skin_account[\s\S]*?weapon_skin_account: self\.weapon_skin_account\.clone\(\)/,
  'client offline launch cannot inject weapon-skin account state');
const appearanceProjection = read(
  'native', 'plugins', 'woc_runtime', 'src', 'client_projection.rs',
);
requireText(appearanceProjection,
  /pub weapon_skin_id: Option<String>/,
  'native actor appearance has no weapon-skin projection field');
const pluginFailure = read(
  '..', '..', 'docs', 'plans', 'zircon_plugins', '08',
  'failure-2026-07-18-woc-zrvm-deterministic-bulk-cross-platform-runtime.md',
);
requireText(pluginFailure,
  /WOS163 presentation transaction addendum[\s\S]*?lossless bulk[\s\S]*?two lossless binary outputs[\s\S]*?weapon-skin change[\s\S]*?updates the returned[\s\S]*?presentation actor identity/,
  'Plugins08 lacks the WOS163 presentation transaction handoff');

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
  /WOS118[\s\S]*?## WOS83 historical delta[\s\S]*?29 source-ordered[\s\S]*?eight nullable[\s\S]*?schema 52[\s\S]*?crossbow before bow/,
  'world-state contract does not document the current WOS118 identity and WOS83 migration');
if (!worldContract.includes('WOS2-WOS82')) {
  throw new Error('world-state contract does not document the WOS82 migration boundary');
}
const readme = read('README.md');
requireText(readme,
  /WOS118[\s\S]*?WOS83 appends[\s\S]*?Schema 52 adds[\s\S]*?change_weapon_skin[\s\S]*?remaining 0 source-shaped client sends; eight dispatch-only commands remain/,
  'README WOS163 identity or protocol coverage summary drifted');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS163 command payload coverage totals drifted');
}

process.stdout.write(`WOS163 weapon-skin static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
