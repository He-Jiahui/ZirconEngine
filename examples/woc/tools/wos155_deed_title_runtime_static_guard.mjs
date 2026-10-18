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
  /case 'deed_set_title':[\s\S]*?msg\.deedId === null \|\| typeof msg\.deedId === 'string'[\s\S]*?sim\.setActiveTitle\(msg\.deedId, pid\)/,
  'source deed_set_title server admission drifted');
const sourceDeeds = source('src/sim/deeds.ts');
requireText(sourceDeeds,
  /setActiveTitle\([\s\S]*?deedsEarned\.has\(deedId\)[\s\S]*?reward\?\.kind !== 'title'[\s\S]*?meta\.activeTitle = deedId;[\s\S]*?e\.title = deedId;/,
  'source deed-title transaction drifted');

const deedContract = JSON.parse(read('reference', 'current-head', 'deed_title_contract.json'));
if (deedContract.deeds.length !== 193 || deedContract.title_deed_count !== 19) {
  throw new Error('source deed/title catalog totals drifted');
}
const generatedDeeds = read('scripts', 'woc_game', 'src', 'generated', 'deed_title_contract.zr');
requireText(generatedDeeds,
  /pub deedCodeForId\(id: string\): uint[\s\S]*?pub deedIsTitle\(index: int\): bool[\s\S]*?pub deedIdUtf8Length[\s\S]*?pub deedIdUtf8Byte/,
  'generated deed/title accessors are missing');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
if (payloadContract.schema_version !== 60) throw new Error('WOS155 command payload schema must be 60');
const payload = payloadContract.entries.find(
  (entry) => entry.id === 159 && entry.name === 'deed_set_title',
);
if (!payload || payload.kind !== 'optional_utf8_id' || payload.min_byte_length !== 1 ||
    payload.max_byte_length !== 261) {
  throw new Error('deed_set_title typed payload contract is missing or non-canonical');
}
const generatedPayloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedPayloads, /deedSetTitleCommandId\(required: bool\): uint[\s\S]*?return <uint>159/,
  'generated deed_set_title command id is missing');
const nativeProtocol = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativeProtocol,
  /pub struct DeedSetTitleCommandPayload[\s\S]*?deed_id: Option<String>[\s\S]*?write_optional_utf8_id[\s\S]*?read_optional_utf8_id/,
  'native deed-set-title payload is missing');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /SetActiveDeedTitle \{[\s\S]*?deed_id: Option<String>[\s\S]*?DeedSetTitleCommandPayload[\s\S]*?DEED_SET_TITLE_COMMAND_ID/,
  'native client deed-title intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must retain the current WOS83 identity');
requireText(world, /commandId == deedSetTitleCommand[\s\S]*?applyOfflineDeedSetTitleCommand/,
  'authoritative command routing does not reach deed_set_title');
requireText(world,
  /applyOfflineDeedSetTitleCommand[\s\S]*?offlineDeedsEarned[\s\S]*?deedCodeIsTitle[\s\S]*?entityActiveTitleDeedCodes/,
  'authoritative deed-title reducer does not enforce earned title rewards');
requireText(world,
  /offlineDeedTitleStateIsValid[\s\S]*?offlineDeedsEarned[\s\S]*?entityActiveTitleDeedCodes/,
  'deed-title snapshot validation is missing');
requireText(world,
  /pub offlineDeedSetTitleCommandStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?encodeState[\s\S]*?decodeState/,
  'deed-title authoritative regression is missing');
requireText(world, /if \(offlineDeedSetTitleCommandStateTest\(\) != 1\) \{[\s\S]*?return -149;/,
  'world selfTest must execute WOS155 coverage');
requireText(world,
  /if \(schemaVersion >= <uint>75\) \{[\s\S]*?offlineDeedsEarned\[deedIndex\][\s\S]*?offlineActiveTitleDeedCode = reader\.u16/,
  'WOS75 deed-ledger codec migration gate is missing');
requireText(world,
  /if \(schemaVersion >= <uint>75\) \{[\s\S]*?entityActiveTitleDeedCodes\.add\(reader\.u16/,
  'WOS75 entity-title codec migration gate is missing');

const nativeIdentity = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeIdentity,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0) {
  throw new Error('WOS155 command payload coverage totals drifted');
}

process.stdout.write(`WOS155 deed-title static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
