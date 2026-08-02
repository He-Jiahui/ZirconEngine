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

const sourceDamage = source('src/sim/combat/damage.ts');
requireText(sourceDamage,
  /const eligible: PlayerMeta\[\] = \[\][\s\S]*?for \(const mPid of party\.members\)[\s\S]*?dist2d\(participationPos, e\.pos\) <= PARTY_XP_RANGE[\s\S]*?e\.lootRecipientIds = eligible\.map/,
  'source kill-time loot-recipient snapshot semantics drifted');

const sourceLoot = source('src/sim/loot/loot_roll.ts');
requireText(sourceLoot,
  /if \(mob\.lootRecipientIds && mob\.lootRecipientIds\.length > 0\)[\s\S]*?mob\.lootRecipientIds\.flatMap/,
  'source corpse loot candidates no longer prefer the death-time snapshot');
requireText(sourceLoot,
  /function tryAwardCopperByFairSplit[\s\S]*?const base = Math\.floor\(copper \/ candidates\.length\)[\s\S]*?ctx\.rng\.int\(i, order\.length - 1\)[\s\S]*?grantLootCopper/,
  'source fair-split currency semantics drifted');
requireText(sourceLoot,
  /function tryAwardItemByRoundRobin[\s\S]*?candidates\[party\.lootTurn % candidates\.length\][\s\S]*?party\.lootTurn\+\+[\s\S]*?ctx\.addItem/,
  'source round-robin item semantics drifted');

const sourceLifecycle = source('src/sim/mob/lifecycle.ts');
requireText(sourceLifecycle, /mob\.lootRecipientIds = undefined;/,
  'source respawn no longer clears corpse recipient snapshots');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world,
  /pub var partyLootTurnPartyIds: container\.Array<uint>;[\s\S]*?pub var partyLootTurnValues: container\.Array<uint>;/,
  'WOS72 party loot-turn state is missing');
requireText(world,
  /pub var corpseLootRecipientMobIds: container\.Array<uint>;[\s\S]*?pub var corpseLootRecipientPlayerIds: container\.Array<uint>;[\s\S]*?pub var corpseLootRecipientOrders: container\.Array<uint>;/,
  'WOS72 corpse recipient rows are missing');
requireText(world,
  /captureOfflineCorpseLootRecipients[\s\S]*?memberOrder <= <uint>10[\s\S]*?dx \* dx \+ dz \* dz <= 10000\.0[\s\S]*?corpseLootRecipientPlayerIds\.add/,
  'authoritative death-time recipient capture is missing');
requireText(world,
  /offlineCorpseLootCandidateIds[\s\S]*?corpseLootRecipientMobIds[\s\S]*?entityPartyMemberOrders/,
  'snapshot-first loot candidate lookup or historical fallback is missing');
requireText(world,
  /tryAwardOfflineCopperByFairSplit[\s\S]*?base = copper \/ candidates\.length[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?entityInventoryCopper/,
  'deterministic fair-split currency reducer is missing');
requireText(world,
  /tryAwardOfflineItemByRoundRobin[\s\S]*?partyLootTurn[\s\S]*?candidates\[<int>\(turn % <uint>candidates\.length\)\][\s\S]*?grantM5InventoryItem/,
  'deterministic round-robin item reducer is missing');
requireText(world,
  /rollOfflineEastbrookCorpseLoot[\s\S]*?captureOfflineCorpseLootRecipients/,
  'corpse loot generation does not capture recipients at death');
requireText(world,
  /applyOfflineCorpseLootCommand[\s\S]*?tryAwardOfflineCopperByFairSplit[\s\S]*?tryAwardOfflineItemByRoundRobin/,
  'corpse loot command does not route fair split and round robin');
requireText(world,
  /writer\.u16\(<uint>78[\s\S]*?partyLootTurnPartyIds\.length[\s\S]*?corpseLootRecipientMobIds\.length/,
  'WOS72 encoder tail is missing');
requireText(world,
  /schemaVersion != <uint>70 &&\s*schemaVersion != <uint>71[\s\S]*?schemaVersion >= <uint>71[\s\S]*?partyLootTurnPartyIds\.add[\s\S]*?corpseLootRecipientMobIds\.add/,
  'WOS72 decoder admission, tail, or WOS70 compatibility is missing');
requireText(world,
  /pub partyLootDistributionStateTest\(\): int[\s\S]*?encodeState[\s\S]*?decodeState/,
  'WOS147 authoritative regression is missing');
requireText(world,
  /if \(partyLootDistributionStateTest\(\) != 1\) \{[\s\S]*?return -141;/,
  'world selfTest must execute WOS147 coverage');

const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS78/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS74 in both runtime paths');
}
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(protocol, /WORLD_STATE_FORMAT: &str = "WOS78"[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 78/,
  'native protocol identity must publish WOS74');

const contract = read('contracts', 'world-state.md');
requireText(contract,
  /WOS71[\s\S]*?death-time[\s\S]*?fair-split/i,
  'world-state contract must document WOS71 party loot distribution');
requireText(contract, /WOS71[\s\S]*?round-robin/i,
  'world-state contract must document the WOS71 round-robin cursor');

process.stdout.write(`WOS147 party loot distribution static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
