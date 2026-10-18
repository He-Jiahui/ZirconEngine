import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const lifecycle = read('../../dev/world-of-claudecraft/src/sim/combat/casting_lifecycle.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/drain_tick_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const testMain = read('scripts/woc_game/src/world/drain_tick_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_drain_tick_talent_runtime_tests.zrp');

assert.match(source, /case 'drainTick':[\s\S]*?Math\.round\(eff\.min \* dmgMult\)[\s\S]*?Math\.round\(eff\.max \* dmgMult\)/);
assert.match(lifecycle, /const res = ctx\.resolvedAbility\(abilityId, p\.id\)[\s\S]*?applyChannelTick\(ctx, p, res\)/);
assert.match(lifecycle, /else if \(eff\.type === 'drainTick'\)[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\) \+ channelSp[\s\S]*?Math\.round\(dmg \* eff\.healFrac\)/);

const retained = contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === 'drainTick'))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(retained, ['drain_life', 'mind_flay']);

assert.match(scaling, /pub resolveDrainTickRange\([\s\S]*?"meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"/);
const resolverBody = scaling.match(/pub resolveDrainTickRange\([\s\S]*?^}/m)?.[0] ?? '';
assert.doesNotMatch(resolverBody, /"flatDmg"|"dotDmgPct"/);
assert.match(scaling, /endpointFirst == 61 && postRoll == 60/);
assert.match(state, /var drainTickTalentScaling = %import\("combat\/drain_tick_talent_scaling_state"\);/);
assert.match(state, /offlineTalentDrainTickEffectIndex[\s\S]*?effects\.typeAt[\s\S]*?"drainTick"/);
assert.match(state, /offlineTalentProjectileSnapshotIsEligible[\s\S]*?offlineTalentDrainTickEffectIndex/);
assert.match(state, /resolveOfflineTalentDrainTickRange[\s\S]*?drainTickTalentScaling\.resolveDrainTickRange/);
assert.match(state, /offlineTalentDrainTickProjectileRangeIsValid[\s\S]*?offlineProjectileTalentSnapshotIsValid[\s\S]*?drainTickRangeSnapshotMatches/);
assert.match(
  state,
  /schemaVersion < <uint>107\s*&&\s*!offlineProjectileTalentSnapshotsAreLegacyCompatible\(state\)/,
  'WOS106 must reject a non-empty projectile snapshot',
);
assert.match(
  state,
  /if \(schemaVersion >= <uint>107\) \{\s*writer\.byte\(<uint>state\.offlineProjectileTalentSpecCodes\[projectileIndex\], 1\);/,
  'WOS107 must encode projectile snapshots',
);
for (const caller of ['launchOfflineDrainLifeChannelTick', 'launchOfflineMindFlayChannelTick']) {
  assert.match(state, new RegExp(`${caller}[\\s\\S]*?resolveOfflineTalentDrainTickRange`));
}
for (const landing of ['landOfflineDrainLifeProjectile', 'landOfflineMindFlayProjectile']) {
  assert.match(state, new RegExp(`${landing}[\\s\\S]*?offlineTalentDrainTickProjectileRangeIsValid`));
}
assert.match(state, /pub drainTickTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /drainTickTalentScalingRuntimeStateTest[\s\S]*?specCode\("priest", "shadow"\)[\s\S]*?resolved\.minimum != 13[\s\S]*?offlineTalentDrainTickProjectileRangeIsValid/);
assert.ok(state.includes('var restored = decodeState(encodeStateVersion(state, <uint>107));'), 'WOS210 must round-trip its projectile snapshot through WOS107');
assert.match(testMain, /state\.drainTickTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS210 drain-tick talent scaling static guard passed');
