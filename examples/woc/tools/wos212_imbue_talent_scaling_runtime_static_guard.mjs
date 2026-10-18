import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const dispatch = read('../../dev/world-of-claudecraft/src/sim/combat/effect_dispatch.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/imbue_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const main = read('scripts/woc_game/src/main.zr');
const testMain = read('scripts/woc_game/src/world/imbue_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_imbue_talent_runtime_tests.zrp');

assert.match(source, /case 'imbue':[\s\S]*?bonus: Math\.round\(eff\.bonus \* dmgMult \+ flat\)[\s\S]*?judgeMin:[\s\S]*?Math\.round\(eff\.judgeMin \* dmgMult \+ flat\)[\s\S]*?judgeMax:[\s\S]*?Math\.round\(eff\.judgeMax \* dmgMult \+ flat\)/);
assert.match(dispatch, /case 'imbue':[\s\S]*?value: eff\.bonus,[\s\S]*?value2: eff\.judgeMin,[\s\S]*?value3: eff\.judgeMax/);
assert.match(dispatch, /case 'judgement':[\s\S]*?p\.auras\.splice\(sealIdx, 1\)[\s\S]*?ctx\.rng\.range\(seal\.value2 \?\? 10, seal\.value3 \?\? 15\)/);

const idsFor = (type) => contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === type))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(idsFor('imbue'), [
  'flametongue_weapon',
  'frostbrand_weapon',
  'seal_of_righteousness',
]);

assert.match(scaling, /school == "physical" \? "meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"/);
assert.match(scaling, /pub resolveImbue\([\s\S]*?"flatDmg"/);
assert.match(scaling, /if \(hasJudgement\) \{[\s\S]*?judgementMinimum[\s\S]*?judgementMaximum/);
assert.match(scaling, /spell\.bonus != 11[\s\S]*?spell\.judgementMinimum != 26[\s\S]*?spell\.judgementMaximum != 40/);
assert.match(scaling, /physical\.bonus != 13[\s\S]*?physical\.judgementMinimum != 30[\s\S]*?physical\.judgementMaximum != 45/);

assert.match(state, /var imbueTalentScaling = %import\("combat\/imbue_talent_scaling_state"\);/);
for (const field of [
  'entityImbueResolvedBonuses',
  'entityImbueResolvedJudgementMinimums',
  'entityImbueResolvedJudgementMaximums',
  'entityImbueTalentSpecCodes',
  'entityImbueTalentRowOptionCodes',
]) {
  assert.match(state, new RegExp(`pub var ${field}:`));
}
assert.match(state, /applyOfflineImbue\([\s\S]*?captureOfflineTalentImbueState\(state, casterIndex\)/);
assert.match(state, /offlineImbueBonus\([\s\S]*?entityImbueResolvedBonuses/);
assert.match(state, /startOfflineJudgementCast[\s\S]*?entityImbueAbilityCodes\[casterIndex\] !=[\s\S]*?sealOfRighteousnessAbilityCode\(\)[\s\S]*?entityImbueResolvedJudgementMinimums[\s\S]*?entityImbueResolvedJudgementMaximums[\s\S]*?clearOfflineImbue\(state, casterIndex\)/);
assert.match(state, /imbueStateIsValid[\s\S]*?offlineTalentImbueResolvedValues[\s\S]*?judgementMinimum != expected\.judgementMinimum/);
assert.match(state, /schemaVersion < <uint>109 && !imbueStateIsSchema108Compatible\(state\)/);
assert.match(state, /schemaVersion >= <uint>109[\s\S]*?entityImbueResolvedBonuses[\s\S]*?entityImbueTalentRowOptionCodes/);
assert.match(state, /schemaVersion < <uint>109[\s\S]*?normalizeOfflineLegacyImbueTalentRows\(state\)/);
assert.match(state, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(main, /world_state\\":\\"WOS118/);
assert.match(state, /pub imbueTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /imbueTalentScalingRuntimeStateTest[\s\S]*?applyOfflineSealOfRighteousness[\s\S]*?applyOfflineFlametongueWeapon[\s\S]*?applyOfflineFrostbrandWeapon/);
assert.match(state, /encodeStateVersion\(sealState, <uint>108\)/);
assert.match(testMain, /state\.imbueTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS212 imbue talent scaling static guard passed');
