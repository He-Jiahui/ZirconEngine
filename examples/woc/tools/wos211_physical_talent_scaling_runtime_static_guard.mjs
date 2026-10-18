import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const dispatch = read('../../dev/world-of-claudecraft/src/sim/combat/effect_dispatch.ts');
const autoSource = read('../../dev/world-of-claudecraft/src/sim/combat/auto_attack.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/physical_talent_scaling_state.zr');
const auto = read('scripts/woc_game/src/combat/auto_attack_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const testMain = read('scripts/woc_game/src/world/physical_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_physical_talent_runtime_tests.zrp');

assert.match(source, /case 'weaponDamage':[\s\S]*?Math\.round\(eff\.bonus \* dmgMult \+ flat\)/);
assert.match(source, /case 'weaponStrike':[\s\S]*?weaponMult: \(eff\.weaponMult \?\? 1\) \* dmgMult/);
assert.match(source, /case 'finisherDamage':[\s\S]*?base: Math\.round\(eff\.base \* dmgMult \+ flat\)[\s\S]*?perCombo: Math\.round\(eff\.perCombo \* dmgMult\)/);
assert.match(autoSource, /const queued = ctx\.resolvedAbility\(p\.queuedOnSwing, p\.id\)[\s\S]*?eff\.type === 'weaponDamage'/);
assert.match(dispatch, /case 'finisherDamage':[\s\S]*?eff\.base \+[\s\S]*?eff\.perCombo \* spentCombo[\s\S]*?ctx\.rng\.range\(0, eff\.variance\)[\s\S]*?effectiveAttackPower\(p\) \/ 14/);

const idsFor = (type) => contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === type))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(idsFor('weaponDamage'), ['heroic_strike', 'maul', 'raptor_strike']);
assert.deepEqual(idsFor('weaponStrike'), [
  'backstab', 'claw', 'hemorrhage', 'rake', 'sinister_strike', 'stormstrike',
]);
assert.deepEqual(idsFor('finisherDamage'), ['eviscerate', 'ferocious_bite']);

assert.match(scaling, /pub resolveWeaponDamageBonus\([\s\S]*?"meleeDmgPct"[\s\S]*?"flatDmg"/);
assert.match(scaling, /pub resolveWeaponStrike\([\s\S]*?authoredWeaponMultiplier \* multiplier/);
assert.match(scaling, /pub resolveFinisher\([\s\S]*?resolved\.base = roundPositive[\s\S]*?resolved\.perCombo = roundPositive/);
assert.match(scaling, /split\.base \+ split\.perCombo \* 3 == 4 && reversed == 5/);
assert.match(auto, /queuedTalentBonusResolved[\s\S]*?queuedTalentBonus/);
assert.match(auto, /actor\.queuedTalentBonusResolved \? actor\.queuedTalentBonus/);
assert.match(state, /var physicalTalentScaling = %import\("combat\/physical_talent_scaling_state"\);/);
assert.match(state, /prepareOfflineAutoActor[\s\S]*?resolveOfflineTalentWeaponDamageBonus/);
for (const caller of [
  'startOfflineRakeCast', 'startOfflineClawCast', 'startOfflineSinisterStrikeCast',
  'startOfflineStormstrikeCast', 'startOfflineBackstabCast',
]) {
  assert.match(state, new RegExp(`${caller}[\\s\\S]*?resolveOfflineTalentWeaponStrike`));
}
for (const caller of ['startOfflineEviscerateCast', 'startOfflineFerociousBiteCast']) {
  assert.match(state, new RegExp(`${caller}[\\s\\S]*?resolveOfflineTalentFinisher`));
}
assert.match(state, /pub physicalTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /physicalTalentScalingRuntimeStateTest[\s\S]*?specCode\("druid", "feral"\)[\s\S]*?queued\.queuedTalentBonus != 31\.0[\s\S]*?laterQueued\.queuedTalentBonus == 27\.0/);
assert.match(testMain, /state\.physicalTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS211 physical talent scaling static guard passed');
