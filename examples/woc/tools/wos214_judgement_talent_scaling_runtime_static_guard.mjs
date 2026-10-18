import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const dispatch = read('../../dev/world-of-claudecraft/src/sim/combat/effect_dispatch.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/judgement_talent_scaling_state.zr');
const numeric = read('scripts/woc_game/src/combat/effect_numeric_dispatch_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const testMain = read('scripts/woc_game/src/world/judgement_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_judgement_talent_runtime_tests.zrp');

assert.match(source, /case 'judgement':[\s\S]*?dmgMult: \(eff\.dmgMult \?\? 1\) \* dmgMult,[\s\S]*?flat: \(eff\.flat \?\? 0\) \+ flat/);
assert.match(dispatch, /case 'judgement':[\s\S]*?p\.auras\.splice\(sealIdx, 1\)[\s\S]*?ctx\.rng\.range\(seal\.value2 \?\? 10, seal\.value3 \?\? 15\)[\s\S]*?baseDmg \* \(eff\.dmgMult \?\? 1\)[\s\S]*?\(eff\.flat \?\? 0\)[\s\S]*?directHitBonus[\s\S]*?Math\.round\(dmg\)/);

const idsFor = (type) => contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === type))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(idsFor('judgement'), ['judgement']);

assert.match(scaling, /pub resolveJudgement\([\s\S]*?school == "physical" \? "meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"[\s\S]*?"flatDmg"/);
assert.match(scaling, /holy\.damageMultiplier == 1\.35[\s\S]*?holy\.flatDamage == 2\.0[\s\S]*?physical\.damageMultiplier == 1\.55/);

assert.match(numeric, /pub var judgementDamageMultiplier: float;/);
assert.match(numeric, /pub var judgementFlatDamage: float;/);
assert.match(numeric, /this\.judgementDamageMultiplier = 1\.0;[\s\S]*?this\.judgementFlatDamage = 0\.0;/);
assert.match(numeric, /dispatchJudgement[\s\S]*?randomRange\(state, minimum, maximum\) \* state\.judgementDamageMultiplier[\s\S]*?state\.judgementFlatDamage[\s\S]*?directHitBonus[\s\S]*?if \(crit\)[\s\S]*?roundPositive\(damage\)/);
assert.match(numeric, /judgement\.judgementDamageMultiplier = 1\.2;[\s\S]*?judgement\.judgementFlatDamage = 2\.0;[\s\S]*?damageAmounts\[0\] != 76/);

assert.match(state, /var judgementTalentScaling = %import\("combat\/judgement_talent_scaling_state"\);/);
assert.match(state, /startOfflineJudgementCast[\s\S]*?entityImbueResolvedJudgementMinimums[\s\S]*?entityImbueResolvedJudgementMaximums[\s\S]*?judgementTalentScaling\.resolveJudgement\([\s\S]*?offlineTalentModifiersForEntity\(state, casterIndex\)[\s\S]*?"holy"[\s\S]*?numeric\.judgementDamageMultiplier[\s\S]*?numeric\.judgementFlatDamage[\s\S]*?clearOfflineImbue\(state, casterIndex\)/);
assert.match(state, /pub judgementTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /judgementTalentScalingRuntimeStateTest[\s\S]*?applyOfflineSealOfRighteousness[\s\S]*?decodeState\(encodeState\(baseline\)\)[\s\S]*?replaceOfflineTalentAllocation\([\s\S]*?startOfflineJudgementCast\(baseline, 0, targetId\)[\s\S]*?startOfflineJudgementCast\(scaled, 0, targetId\)[\s\S]*?scaledDamage > baselineDamage/);
assert.match(testMain, /state\.judgementTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS214 Judgement talent scaling static guard passed');
