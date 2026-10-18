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
  /case 'saveLoadout':[\s\S]*?hasOwn\(msg, 'alloc'\)[\s\S]*?parseTalentAllocation\(msg\.alloc\)[\s\S]*?sim\.saveLoadout\(msg\.name, Array\.isArray\(msg\.bar\) \? msg\.bar : \[\], pid, alloc\)[\s\S]*?sim\.saveLoadout\(msg\.name, Array\.isArray\(msg\.bar\) \? msg\.bar : \[\], pid\)/,
  'source saveLoadout server admission drifted');
const sourceTalents = source('src/sim/progression/talents.ts');
requireText(sourceTalents,
  /saveTalentLoadout[\s\S]*?commitTalentAllocation[\s\S]*?slice\(0, 24\)[\s\S]*?slice\(0, SAVED_LOADOUT_BAR_SLOTS\)[\s\S]*?findIndex[\s\S]*?MAX_LOADOUTS[\s\S]*?activeLoadout/,
  'source save-loadout state transition drifted');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 60) throw new Error('WOS153 command payload schema must be 60');
const save = contract.entries.find((entry) => entry.id === 98 && entry.name === 'saveLoadout');
if (!save || save.kind !== 'save_loadout' || save.min_byte_length !== 6 ||
    save.max_byte_length !== 5858 || save.max_utf16_code_units !== 24 ||
    save.max_collection_entries !== 22) {
  throw new Error('saveLoadout typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated, /saveLoadoutCommandId\(required: bool\): uint[\s\S]*?return <uint>98/,
  'generated saveLoadout command id is missing');
requireText(generated, /pub payloadKind[\s\S]*?if \(id == <uint>98\) \{ return 52; \}/,
  'generated saveLoadout payload kind is missing');

const protocol = read('native', 'crates', 'woc_protocol', 'src', 'save_loadout_payload.rs');
requireText(protocol,
  /pub struct SaveLoadoutCommandPayload[\s\S]*?name: String[\s\S]*?allocation: Option<ApplyTalentsCommandPayload>[\s\S]*?action_bar: Vec<Option<String>>/,
  'native save-loadout payload type is missing');
const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /SaveTalentLoadout \{[\s\S]*?truncate_utf16_code_units[\s\S]*?take\(MAX_LOADOUT_ACTION_BAR_SLOTS\)[\s\S]*?SaveLoadoutCommandPayload|SaveTalentLoadout \{[\s\S]*?SaveLoadoutCommandPayload[\s\S]*?truncate_utf16_code_units[\s\S]*?take\(MAX_LOADOUT_ACTION_BAR_SLOTS\)/,
  'native client save-loadout intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /commandId == saveLoadoutCommand[\s\S]*?applyOfflineSaveTalentLoadoutCommand/,
  'authoritative command routing does not reach saveLoadout');
requireText(world,
  /applyOfflineSaveTalentLoadoutCommand[\s\S]*?replaceOfflineTalentAllocation[\s\S]*?offlineTalentLoadoutNameBytes[\s\S]*?offlineTalentActiveLoadoutIndex/,
  'authoritative save-loadout reducer is missing');
requireText(world,
  /pub offlineSaveTalentLoadoutCommandStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?encodeState/,
  'save-loadout authoritative regression is missing');
requireText(world, /if \(offlineSaveTalentLoadoutCommandStateTest\(\) != 1\) \{[\s\S]*?return -147;/,
  'world selfTest must execute WOS153 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0) {
  throw new Error('WOS153 command payload coverage totals drifted');
}

process.stdout.write(`WOS153 save-loadout static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
