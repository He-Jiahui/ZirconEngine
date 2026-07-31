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
requireText(lootRoll, /LOOT_ROLL_TIMEOUT = 60[\s\S]*?startNeedGreedRoll[\s\S]*?submitLootRoll[\s\S]*?resolveLootRoll/,
  'source need-greed lifecycle drifted');
requireText(lootRoll, /choice === 'need' \|\| choice === 'greed' \? ctx\.rng\.int\(1, 100\) : null[\s\S]*?needers\.length > 0[\s\S]*?tiedWinners[\s\S]*?ctx\.rng\.int\(0, tiedWinners\.length - 1\)/,
  'source need-greed winner semantics drifted');
requireText(lootRoll, /Everyone passed[\s\S]*?returnLootRollItemToCorpse[\s\S]*?openToAll: true/,
  'source need-greed return-to-corpse semantics drifted');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
const payload = payloadContract.entries.find((entry) => entry.name === 'lootRoll');
if (payload?.id !== 14 || payload.encoding !== 'f64_le_roll_id+u8_need_greed_pass' ||
    payload.min_byte_length !== 9 || payload.max_byte_length !== 9) {
  throw new Error('loot-roll payload contract drifted');
}

const runtime = read('scripts', 'woc_game', 'src', 'progression', 'loot_roll_runtime.zr');
requireText(runtime, /sourceCommit\(required: bool\)[\s\S]*?5ef9f7cb21cd8875b6d2c49701015dfcd78de35a/,
  'loot-roll runtime source pin is missing');
requireText(runtime, /wireChoiceToState[\s\S]*?choiceNeedsRoll[\s\S]*?preferredChoice[\s\S]*?winnerCandidateIndex/,
  'loot-roll runtime decision helpers are incomplete');
requireText(runtime, /pub contractTest\(\): int[\s\S]*?return 1/,
  'loot-roll runtime contract test is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /nextLootRollId[\s\S]*?pendingLootRollIds[\s\S]*?pendingLootRollMobIds[\s\S]*?pendingLootRollItemCodes[\s\S]*?pendingLootRollExpiresAtMicros[\s\S]*?pendingLootRollMasterLooterIds/,
  'authoritative pending loot-roll columns are missing');
requireText(world, /lootRollCandidateRollIds[\s\S]*?lootRollCandidatePlayerIds[\s\S]*?lootRollCandidateChoiceCodes[\s\S]*?lootRollCandidateValues/,
  'authoritative loot-roll candidate columns are missing');
requireText(world, /lootRollCommandId\(true\)[\s\S]*?applyLootRollCommand/,
  'loot-roll reducer routing is missing');
requireText(world, /applyOfflineCorpseLootCommand[\s\S]*?startOfflineNeedGreedRoll[\s\S]*?entityCorpseSharedItemCodes/,
  'corpse loot does not start source-compatible need-greed rolls');
requireText(world, /applyLootRollCommand[\s\S]*?binary\.readF64LeAt[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?resolveOfflineLootRoll/,
  'loot-roll command semantics are missing');
requireText(world, /resolveOfflineLootRoll[\s\S]*?lootRollRuntime\.tiedWinnerCount[\s\S]*?lootRollRuntime\.winnerCandidateIndex[\s\S]*?grantM5InventoryItem/,
  'loot-roll resolution semantics are missing');
requireText(world, /updateOfflineLootRolls[\s\S]*?pendingLootRollExpiresAtMicros[\s\S]*?resolveOfflineLootRoll/,
  'loot-roll timeout resolution is missing');
requireText(world, /writer\.u16\(<uint>71, 1, 1\)[\s\S]*?nextLootRollId[\s\S]*?pendingLootRollIds[\s\S]*?lootRollCandidateRollIds/,
  'WOS71 loot-roll snapshot tail is missing');
requireText(world, /if \(schemaVersion >= <uint>70\)[\s\S]*?nextLootRollId[\s\S]*?pendingLootRollIds[\s\S]*?lootRollCandidateRollIds/,
  'WOS71 loot-roll migration is missing');
requireText(world, /pub lootRollCommandStateTest\(\): int[\s\S]*?encodeState[\s\S]*?updateOfflineLootRolls[\s\S]*?appendLootRollCommandForTest/,
  'loot-roll state regression is missing');
requireText(world, /if \(lootRollCommandStateTest\(\) != 1\) \{[\s\S]*?return -139;/,
  'world selfTest must execute loot-roll coverage');

const contract = read('contracts', 'world-state.md');
requireText(contract, /WOS71[\s\S]*?Need\/Greed[\s\S]*?60[\s\S]*?Need[\s\S]*?Greed[\s\S]*?Pass/,
  'world-state contract must document retained Need/Greed state');

process.stdout.write(`WOS145 loot-roll runtime static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
