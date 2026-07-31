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

const sourceParty = source('src/sim/social/party.ts');
requireText(sourceParty, /const RAID_MAX = 10;/, 'source raid-size bound drifted');
const sourceLootRoll = source('src/sim/loot/loot_roll.ts');
requireText(sourceLootRoll, /export function assignMasterLoot[\s\S]*?only the master looter decides[\s\S]*?targets\.length === 0[\s\S]*?targets\.length === 1[\s\S]*?convertMasterRollToNeedGreed/,
  'source master-loot assignment semantics drifted');
requireText(sourceLootRoll, /if \(!isPidResolvable\(ctx, targets\[0\]\)\)[\s\S]*?convertMasterRollToNeedGreed\(ctx, roll, roll\.candidates\)/,
  'source offline single-target fallback drifted');

const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const payload = payloads.entries.find((entry) => entry.name === 'masterAssign');
if (payload?.id !== 49 || payload.kind !== 'master_loot_assignment' ||
    payload.encoding !== 'f64_le_roll_id+u8_count_0_to_10+f64_le_target_pid' ||
    payload.min_byte_length !== 9 || payload.max_byte_length !== 89) {
  throw new Error('master-loot assignment payload contract is missing');
}

const generatedZr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generatedZr, /pub masterAssignCommandId\(required: bool\): uint[\s\S]*?return <uint>49;/,
  'generated Zr master-assign command id is missing');
requireText(generatedZr, /if \(id == <uint>49\) \{ return 9; \}[\s\S]*?pub payloadMaxLength[\s\S]*?if \(id == <uint>49\) \{ return 89; \}/,
  'generated Zr master-assign payload bounds are missing');

const generatedRust = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
requireText(generatedRust, /pub const MASTER_ASSIGN_COMMAND_ID: u16 = 49;/,
  'generated Rust master-assign command id is missing');
requireText(generatedRust, /MasterLootAssignment[\s\S]*?name: "masterAssign"[\s\S]*?min_byte_length: 9[\s\S]*?max_byte_length: 89/,
  'generated Rust master-assign descriptor is missing');

const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'master_loot_assignment_payload.rs');
requireText(nativePayload, /pub struct MasterLootAssignmentPayload[\s\S]*?roll_id: f64[\s\S]*?target_pids: Vec<f64>/,
  'native master-assign payload type is missing');
requireText(nativePayload, /MAX_TARGET_PIDS: usize = 10[\s\S]*?pub fn encode[\s\S]*?pub fn decode[\s\S]*?validate_master_loot_assignment_payload/,
  'native master-assign codec or bound is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /masterAssignCommandId\(true\)[\s\S]*?applyMasterLootAssignmentCommand/,
  'master-assign reducer routing is missing');
requireText(world, /applyMasterLootAssignmentCommand[\s\S]*?pendingLootRollMasterLooterIds[\s\S]*?actorId[\s\S]*?targetCount[\s\S]*?10/,
  'master-assign authorization or bounded payload parsing is missing');
requireText(world, /applyMasterLootAssignmentCommand[\s\S]*?validTargets\.length == 0[\s\S]*?validTargets\.length == 1[\s\S]*?convertOfflineMasterLootRollToNeedGreed/,
  'master-assign empty, direct, or subset transition semantics are missing');
requireText(world, /pub masterLootAssignmentCommandStateTest\(\): int[\s\S]*?appendMasterLootAssignmentCommandForTest[\s\S]*?encodeState/,
  'master-assign authoritative regression is missing');
requireText(world, /if \(masterLootAssignmentCommandStateTest\(\) != 1\) \{[\s\S]*?return -140;/,
  'world selfTest must execute master-assign coverage');

const contract = read('contracts', 'world-state.md');
requireText(contract, /WOS71[\s\S]*?Master Loot[\s\S]*?one target[\s\S]*?multiple targets[\s\S]*?Need\/Greed/i,
  'world-state contract must document master-loot assignment');

process.stdout.write(`WOS146 master-loot assignment static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
