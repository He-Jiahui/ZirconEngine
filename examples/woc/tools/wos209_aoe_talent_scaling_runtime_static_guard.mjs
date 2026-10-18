import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const lifecycle = read('../../dev/world-of-claudecraft/src/sim/combat/casting_lifecycle.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const aoe = read('scripts/woc_game/src/combat/aoe_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const testMain = read('scripts/woc_game/src/world/aoe_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_aoe_talent_runtime_tests.zrp');

assert.match(source, /case 'aoeDamage':[\s\S]*?case 'aoeHeal':[\s\S]*?Math\.round\(eff\.min \* \(eff\.type === 'aoeHeal' \? healMult : dmgMult\) \+ flat\)/);
assert.match(source, /Math\.round\(eff\.max \* \(eff\.type === 'aoeHeal' \? healMult : dmgMult\) \+ flat\)/);
assert.match(lifecycle, /const res = ctx\.resolvedAbility\(abilityId, p\.id\)[\s\S]*?applyChannelTick\(ctx, p, res\)/);
assert.match(aoe, /resolveAoEDamageRange[\s\S]*?school == "physical" \? "meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"[\s\S]*?"flatDmg"/);
assert.match(aoe, /resolveAoEHealRange[\s\S]*?"healPct"[\s\S]*?"dmgPct"[\s\S]*?"flatDmg"/);
assert.match(aoe, /endpointFirst == 63 && postRoll == 62/);

const retained = contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === 'aoeDamage' || effect.type === 'aoeHeal'))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(retained, [
  'arcane_explosion', 'bladestorm', 'flamestrike', 'hurricane', 'rain_of_fire', 'swipe',
]);

assert.match(state, /resolveOfflineTalentAoEDamageRange[\s\S]*?aoeTalentScaling\.resolveAoEDamageRange/);
for (const caller of [
  'startOfflineArcaneExplosionCast',
  'resolveOfflineFlamestrikeBurst',
  'launchOfflineRainOfFireChannelTick',
  'startOfflineSwipeCast',
  'launchOfflineHurricaneChannelTick',
]) {
  assert.match(state, new RegExp(`${caller}[\\s\\S]*?resolveOfflineTalentAoEDamageRange`));
}
assert.match(state, /pub aoeTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /aoeTalentScalingRuntimeStateTest[\s\S]*?specCode\("druid", "balance"\)[\s\S]*?during\.minimum != 14[\s\S]*?after\.minimum == 12/);
assert.match(testMain, /state\.aoeTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS209 AoE talent scaling static guard passed');
