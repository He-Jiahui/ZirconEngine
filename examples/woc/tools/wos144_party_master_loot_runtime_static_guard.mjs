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

const lootRoll = source('src/sim/loot/loot_roll.ts');
requireText(lootRoll, /setPartyLootMaster[\s\S]*?party\.leader !== r\.meta\.entityId[\s\S]*?looter !== 0 && party\.members\.includes\(looter\) \? looter : 0[\s\S]*?party\.lootStrategies\.master = next/,
  'source master-loot command semantics drifted');
const party = source('src/sim/social/party.ts');
requireText(party, /DEFAULT_PARTY_LOOT_STRATEGIES[\s\S]*?effectiveMasterLooter[\s\S]*?removeFromParty/,
  'source master-loot party lifecycle drifted');
const types = source('src/sim/types.ts');
requireText(types, /MasterLootThreshold = 'uncommon' \| 'rare' \| 'epic'[\s\S]*?master: \{ enabled: false, looter: 0, threshold: 'uncommon' \}/,
  'source master-loot default or threshold drifted');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
const payload = payloadContract.entries.find((entry) => entry.name === 'setLootMaster');
if (payload?.id !== 48 || payload.encoding !== 'u8_enabled+f64_le_looter+u8_threshold' ||
    payload.min_byte_length !== 10 || payload.max_byte_length !== 10) {
  throw new Error('master-loot payload contract drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /entityPartyMasterLootEnabled[\s\S]*?entityPartyMasterLooterIds[\s\S]*?entityPartyMasterLootThresholdCodes/,
  'party master-loot state columns are missing');
requireText(world, /partySetLootMasterCommandId\(true\)[\s\S]*?applyPartySetLootMasterCommand/,
  'party master-loot reducer routing is missing');
requireText(world, /applyPartySetLootMasterCommand[\s\S]*?binary\.readF64LeAt[\s\S]*?partyIdForIndex[\s\S]*?entityPartyLeaderIds[\s\S]*?partySetMasterLoot/,
  'party master-loot reducer semantics are missing');
requireText(world, /applyPartyAcceptCommand[\s\S]*?entityPartyMasterLootEnabled[\s\S]*?entityPartyMasterLooterIds[\s\S]*?entityPartyMasterLootThresholdCodes/,
  'party join must inherit master-loot settings');
requireText(world, /partyClearMember[\s\S]*?entityPartyMasterLootEnabled[\s\S]*?entityPartyMasterLooterIds[\s\S]*?entityPartyMasterLootThresholdCodes/,
  'party removal must clear local master-loot columns');
requireText(world, /writer\.u16\(<uint>71, 1, 1\)[\s\S]*?entityPartyMasterLootEnabled[\s\S]*?entityPartyMasterLooterIds[\s\S]*?entityPartyMasterLootThresholdCodes/,
  'WOS71 master-loot tail is missing');
requireText(world, /if \(schemaVersion >= <uint>69\)[\s\S]*?entityPartyMasterLootEnabled[\s\S]*?entityPartyMasterLooterIds[\s\S]*?entityPartyMasterLootThresholdCodes/,
  'WOS71 master-loot migration is missing');
requireText(world, /pub partyMasterLootCommandStateTest\(\): int[\s\S]*?appendPartyMasterLootCommandForTest[\s\S]*?encodeState[\s\S]*?partyLeaveCommandId/,
  'party master-loot state regression is missing');
requireText(world, /if \(partyMasterLootCommandStateTest\(\) != 1\) \{[\s\S]*?return -138;/,
  'world selfTest must execute party master-loot coverage');

const contract = read('contracts', 'world-state.md');
requireText(contract, /WOS71[\s\S]*?Master Loot[\s\S]*?leader[\s\S]*?uncommon[\s\S]*?rare[\s\S]*?epic/,
  'world-state contract must document retained Master Loot settings');

process.stdout.write(`WOS144 party Master Loot static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
