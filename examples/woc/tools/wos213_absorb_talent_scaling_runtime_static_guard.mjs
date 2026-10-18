import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const dispatch = read('../../dev/world-of-claudecraft/src/sim/combat/effect_dispatch.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/absorb_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const main = read('scripts/woc_game/src/main.zr');
const testMain = read('scripts/woc_game/src/world/absorb_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_absorb_talent_runtime_tests.zrp');

assert.match(source, /const healMult = 1 \+ mods\.global\.healPct \+ \(am\?\.dmgPct \?\? 0\);/);
assert.match(source, /const absorbMult = 1 \+ mods\.global\.absorbPct;/);
assert.match(source, /case 'absorb':[\s\S]*?Math\.round\(eff\.amount \* healMult \* absorbMult \+ flat\)/);
assert.match(dispatch, /case 'absorb':[\s\S]*?value: eff\.amount,[\s\S]*?sourceId: p\.id/);

const idsFor = (type) => contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === type))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(idsFor('absorb'), ['power_word_shield', 'temporal_barrier']);

assert.match(scaling, /pub resolveAbsorbAmount\([\s\S]*?"healPct"[\s\S]*?"dmgPct"[\s\S]*?"absorbPct"[\s\S]*?"flatDmg"/);
assert.match(scaling, /resolveAbsorbAmount\(90, mixed, "power_word_shield"\) == 154/);

assert.match(state, /var absorbTalentScaling = %import\("combat\/absorb_talent_scaling_state"\);/);
for (const field of [
  'offlineAbsorbResolvedInitialAmounts',
  'offlineAbsorbTalentSpecCodes',
  'offlineAbsorbTalentRowOptionCodes',
]) {
  assert.match(state, new RegExp(`pub var ${field}:`));
}
assert.match(state, /applyOfflinePowerWordShield\([\s\S]*?resolveOfflineTalentAbsorbAmount\([\s\S]*?captureOfflineTalentAbsorbState\(state, sourceId, abilityCode\)/);
assert.match(state, /startOfflineTemporalBarrierCast[\s\S]*?resolveOfflineTalentAbsorbAmount\([\s\S]*?captureOfflineTalentAbsorbState\([\s\S]*?casterIndex/);
assert.match(state, /offlineAbsorbProfileAndDurationMatches[\s\S]*?offlineAbsorbRawInitialAmount[\s\S]*?offlineAbsorbProfileMatches/);
assert.match(state, /applyOfflineAbsorbDamage[\s\S]*?offlineAbsorbProfileAndDurationMatches\([\s\S]*?offlineAbsorbResolvedInitialAmounts/);
assert.match(state, /offlineAbsorbStateIsValid[\s\S]*?amount > initial[\s\S]*?offlineAbsorbExpectedInitialAmount/);
assert.match(state, /schemaVersion < <uint>110 && !offlineAbsorbStateIsSchema109Compatible\(state\)/);
assert.match(state, /schemaVersion >= <uint>110[\s\S]*?offlineAbsorbResolvedInitialAmounts[\s\S]*?offlineAbsorbTalentRowOptionCodes/);
assert.match(state, /schemaVersion < <uint>110[\s\S]*?normalizeOfflineLegacyAbsorbTalentRows\(state\)/);
assert.match(state, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(main, /world_state\\":\\"WOS118/);
assert.match(state, /pub absorbTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /absorbTalentScalingRuntimeStateTest[\s\S]*?applyOfflinePowerWordShield[\s\S]*?applyOfflineAbsorbDamage[\s\S]*?encodeStateVersion\(legacy, <uint>109\)/);
assert.match(testMain, /state\.absorbTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS213 absorb talent scaling static guard passed');
