import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const talentSource = read('../../dev/world-of-claudecraft/src/sim/content/talents_classic.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/flat_buff_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const main = read('scripts/woc_game/src/main.zr');
const testMain = read('scripts/woc_game/src/world/tigers_fury_talent_snapshot_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_tigers_fury_talent_runtime_tests.zrp');

assert.match(source, /SCALABLE_BUFF_KINDS[\s\S]*?'buff_ap'/);
assert.match(source, /case 'selfBuff':[\s\S]*?SCALABLE_BUFF_KINDS\.has\(eff\.kind\)[\s\S]*?Math\.round\(eff\.value \* dmgMult \+ flat\)/);
assert.match(talentSource, /'feral',[\s\S]*?global: \{ meleeDmgPct: 0\.15, dotDmgPct: 0\.15/);

const tiger = contract.entries.find((entry) => entry.id === 'tigers_fury');
assert.ok(tiger, 'Tiger\'s Fury must be generated into the M4 contract');
assert.equal(tiger.definition.school, 'physical');
assert.equal(tiger.definition.effects.length, 1);
assert.deepEqual(tiger.definition.effects[0], {
  type: 'selfBuff', kind: 'buff_ap', value: 40, duration: 6,
});

assert.match(scaling, /pub resolveFlatBuffValue\([\s\S]*?school == "physical" \? "meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"[\s\S]*?"flatDmg"/);
assert.match(scaling, /resolveFlatBuffValue\(40, mixed, "physical", "tigers_fury"\) == 48[\s\S]*?resolveFlatBuffValue\(40, mixed, "fire", "tigers_fury"\) == 50/);

assert.match(state, /var flatBuffTalentScaling = %import\("combat\/flat_buff_talent_scaling_state"\);/);
assert.match(state, /entityTigersFuryResolvedAttackPower[\s\S]*?entityTigersFuryTalentSpecCodes[\s\S]*?entityTigersFuryTalentRowOptionCodes/);
assert.match(state, /startOfflineTigersFuryCast[\s\S]*?captureOfflineTigersFuryTalentState\(state, casterIndex\)/);
assert.match(state, /removeMotionAuraAt[\s\S]*?removedAbilityCode[\s\S]*?clearOfflineTigersFuryTalentState\(state, entityIndex\)/);
assert.match(state, /tigersFuryTalentStateIsValid[\s\S]*?offlineTigersFuryTalentSnapshotModifiers[\s\S]*?entityMotionAuraValues[a-zA-Z\[\]0-9_]*[\s\S]*?entityTigersFuryResolvedAttackPower/);
assert.match(state, /schemaVersion != <uint>111[\s\S]*?schemaVersion < <uint>111[\s\S]*?tigersFuryTalentStateIsSchema110Compatible/);
assert.match(state, /schemaVersion >= <uint>111[\s\S]*?entityTigersFuryResolvedAttackPower[\s\S]*?normalizeOfflineLegacyTigersFuryTalentRows/);
assert.match(state, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(main, /world_state\\":\\"WOS118/);
assert.match(state, /pub tigersFuryTalentSnapshotRuntimeStateTest\(\): int/);
assert.match(state, /tigersFuryTalentSnapshotRuntimeStateTest[\s\S]*?entityMotionAuraValues\[0\] != 46\.0[\s\S]*?replaceOfflineTalentAllocation[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?ageMotionAurasForEntity[\s\S]*?entityMotionAuraValues\[0\] != 40\.0[\s\S]*?encodeStateVersion\(restored, <uint>110\)/);
assert.match(testMain, /state\.tigersFuryTalentSnapshotRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS216 Tiger\'s Fury talent snapshot static guard passed');
