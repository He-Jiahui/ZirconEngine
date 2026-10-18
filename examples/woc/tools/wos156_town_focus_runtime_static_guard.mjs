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
  /case 'set_town_focus':[\s\S]*?Object\.entries\(msg\.allocation[\s\S]*?typeof v === 'number'[\s\S]*?sim\.setTownFocus\(allocation, pid\)/,
  'source set_town_focus server admission drifted');
const sourceFocus = source('src/sim/professions/focus.ts');
requireText(sourceFocus,
  /FOCUS_POINT_BUDGET = 10[\s\S]*?dx \* dx \+ dz \* dz <= zone\.hub\.radius \* zone\.hub\.radius/,
  'source town-focus budget or inclusive town circle drifted');
requireText(sourceFocus,
  /setTownFocus\([\s\S]*?!isInTown[\s\S]*?Number\.isInteger\(points\)[\s\S]*?points < 0[\s\S]*?total > FOCUS_POINT_BUDGET[\s\S]*?if \(points > 0\) allocation\[componentType\] = points/,
  'source town-focus reducer ordering drifted');
const sourceSim = source('src/sim/sim.ts');
requireText(sourceSim,
  /setTownFocus\(allocation[\s\S]*?zoneAt\(p\.pos\.z\)[\s\S]*?isInTownZone\(p\.pos, zone\)[\s\S]*?meta\.townFocus = result\.allocation/,
  'source Sim town-focus commit path drifted');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
if (payloadContract.schema_version !== 60) {
  throw new Error('WOS156 command payload schema must be 60');
}
const payload = payloadContract.entries.find(
  (entry) => entry.id === 140 && entry.name === 'set_town_focus',
);
if (!payload || payload.kind !== 'town_focus_allocation' ||
    payload.min_byte_length !== 2 || payload.max_byte_length !== 65536 ||
    payload.max_utf8_bytes !== 256 || payload.max_collection_entries !== 256 ||
    payload.source_shape?.method !== 'setTownFocus') {
  throw new Error('set_town_focus typed payload contract is missing or non-canonical');
}

const generatedPayloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayloads,
  /setTownFocusCommandId\(required: bool\): uint[\s\S]*?return <uint>140[\s\S]*?if \(id == <uint>140\) \{ return 53; \}/,
  'generated set_town_focus id or payload kind is missing');
const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'town_focus_payload.rs');
requireText(nativePayload,
  /pub struct TownFocusAllocationEntry[\s\S]*?component: String[\s\S]*?points: i32[\s\S]*?allocation: Vec<TownFocusAllocationEntry>/,
  'native town-focus payload does not preserve ordered entries');
for (const [pattern, label] of [
  [/BTreeSet[\s\S]*?DuplicateCommandObjectKey/, 'duplicate keys'],
  [/std::str::from_utf8[\s\S]*?ProtocolError::InvalidUtf8/, 'UTF-8'],
  [/cursor != bytes\.len\(\)[\s\S]*?ProtocolError::TrailingPayload/, 'trailing bytes'],
]) {
  requireText(nativePayload, pattern, `native town-focus ${label} validation is missing`);
}
if (nativePayload.includes('BTreeMap')) {
  throw new Error('native town-focus payload must not sort the source allocation');
}
const nativeDispatch = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativeDispatch,
  /CommandPayloadKind::TownFocusAllocation => validate_town_focus_payload\(bytes\)/,
  'native command validator does not dispatch town-focus payloads');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /SetTownFocus \{[\s\S]*?Vec<TownFocusAllocationEntry>[\s\S]*?TownFocusCommandPayload \{ allocation \}[\s\S]*?SET_TOWN_FOCUS_COMMAND_ID/,
  'native client town-focus intent mapping is missing');

const wireState = read('scripts', 'woc_game', 'src', 'progression', 'town_focus_wire_state.zr');
requireText(wireState,
  /entryUtf8Valid[\s\S]*?first >= <uint>194[\s\S]*?first >= <uint>240[\s\S]*?first <= <uint>244/,
  'Zr town-focus adapter does not enforce canonical UTF-8');
requireText(wireState,
  /value > budget - total[\s\S]*?copyPositiveEntries[\s\S]*?result\.reason = 1[\s\S]*?result\.reason = 2[\s\S]*?result\.reason = 3/,
  'Zr town-focus budget or rejection semantics are incomplete');
requireText(wireState,
  /invalidUtf8Bytes[\s\S]*?2147483647[\s\S]*?reason == 3/,
  'Zr town-focus overflow and malformed UTF-8 regressions are missing');
const stringState = read('scripts', 'woc_game', 'src', 'progression', 'town_focus_state.zr');
requireText(stringState,
  /points > budget - total[\s\S]*?addEntry\(huge, "hide", 2147483647\)[\s\S]*?reason != 3/,
  'Zr string town-focus rules do not reject overflow-sized allocations');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must retain the current WOS83 identity');
requireText(world,
  /commandId == setTownFocusCommand[\s\S]*?applyOfflineSetTownFocusCommand/,
  'authoritative command routing does not reach set_town_focus');
const reducerStart = world.indexOf('\napplyOfflineSetTownFocusCommand(');
const reducerEnd = world.indexOf('\nofflineCraftStateIsValid(', reducerStart);
const reducer = world.slice(reducerStart, reducerEnd);
requireText(reducer,
  /townFocusAllocationFromPayload[\s\S]*?zoneIndexAt[\s\S]*?isInTownHub[\s\S]*?setTownFocus[\s\S]*?result\.ok[\s\S]*?commitOfflineTownFocusAllocation/,
  'authoritative town-focus reducer is incomplete');
if (reducer.includes('entityDead') || reducer.includes('nextAuthoritativeRandom')) {
  throw new Error('town-focus reducer must have no dead-state gate or RNG draw');
}
requireText(world,
  /if \(schemaVersion >= <uint>76\) \{[\s\S]*?townFocusCount[\s\S]*?offlineTownFocusComponentBytes\.add\(reader\.byte[\s\S]*?offlineTownFocusPoints\.add/,
  'WOS76 town-focus codec migration gate is missing');
requireText(world,
  /pub offlineSetTownFocusCommandStateTest\(\): int[\s\S]*?outOfTownRequest[\s\S]*?invalidRequest[\s\S]*?unicodeRequest[\s\S]*?decodeState\(encodeState\(state\)\)/,
  'town-focus authoritative regression is missing');
requireText(world,
  /if \(offlineSetTownFocusCommandStateTest\(\) != 1\) \{[\s\S]*?return -150;/,
  'world selfTest must execute WOS156 coverage');

const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS118/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS118 in both runtime paths');
}
const nativeIdentity = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeIdentity,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const worldContract = read('contracts', 'world-state.md');
requireText(worldContract,
  /WOS82 mech chroma delta[\s\S]*?WOS76 appends `PlayerMeta\.townFocus`[\s\S]*?no\s+dead-state gate and consumes no RNG/,
  'world-state contract does not document the WOS76 town-focus delta');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS156 command payload coverage totals drifted');
}

process.stdout.write(`WOS156 town-focus static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
