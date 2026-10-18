import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const readSource = (...parts) => readFileSync(join(sourceRoot, ...parts), 'utf8');
const scalingPath = join(
  projectRoot,
  'scripts',
  'woc_game',
  'src',
  'combat',
  'dot_talent_scaling_state.zr',
);

assert.match(
  readSource('src', 'sim', 'content', 'classes.ts'),
  /case 'dot':[\s\S]*?eff\.directPct\s*\?[\s\S]*?Math\.round\(eff\.total \* dmgMult \* dotMult \+ flat\)/,
);
assert.ok(existsSync(scalingPath), 'WOS208 dot talent scaling module is missing');

const scaling = read('scripts', 'woc_game', 'src', 'combat', 'dot_talent_scaling_state.zr');
const numeric = read('scripts', 'woc_game', 'src', 'combat', 'effect_numeric_dispatch_state.zr');
const state = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const testMain = read('scripts', 'woc_game', 'src', 'world', 'dot_talent_scaling_runtime_test_main.zr');
const manifest = JSON.parse(read('scripts', 'woc_game', 'woc_m4_dot_talent_runtime_tests.zrp'));

assert.match(scaling, /pub resolveDotTotal\(/);
assert.match(scaling, /baseTotal \* damageMultiplier \* dotMultiplier \+ flat/);
assert.match(scaling, /"meleeDmgPct"/);
assert.match(scaling, /"spellDmgPct"/);
assert.match(scaling, /"dotDmgPct"/);
assert.match(scaling, /"dmgPct"/);
assert.match(scaling, /"flatDmg"/);
assert.match(scaling, /directPct\) \{ return baseTotal; \}/);
assert.match(scaling, /ordering != 78[\s\S]*?reverse == ordering/);
assert.match(numeric, /dotProfileUsesPowerBonus\([\s\S]*?"directDamage"[\s\S]*?"chainDamage"[\s\S]*?"aoeDamage"[\s\S]*?"aoeRoot"/);
assert.match(numeric, /pub dotProfileFromResolvedTotal\(/);
assert.match(numeric, /var bonus = dotProfileUsesPowerBonus/);
assert.match(state, /var dotTalentScaling = %import\("combat\/dot_talent_scaling_state"\);/);
assert.match(state, /offlineDotResolvedTotals[\s\S]*?offlineDotTalentSpecCodes[\s\S]*?offlineDotTalentRowOptionCodes/);
assert.match(state, /appendOfflineDotTalentSnapshot\([\s\S]*?offlineProjectileTalentSnapshotRowCodeAt/);
assert.match(state, /offlineTalentDotProjectileResolvedTotal\(/);
assert.match(state, /offlineDotTalentSnapshotIsValid\([\s\S]*?offlineTalentAllocationValuesAreValid/);
assert.match(state, /schemaVersion < <uint>108[\s\S]*?!offlineDotStateIsSchema107Compatible\(state\)[\s\S]*?cannot be encoded by a historical schema/);
assert.match(state, /offlineDotStateIsSchema107Compatible\([\s\S]*?offlineDotTalentSnapshotsAreLegacyCompatible[\s\S]*?effects\.metric/);
assert.match(state, /offlineLegacyDotCanRecoverGeneratedTotal\(abilityCode\)[\s\S]*?offlineLegacyDotProfileIsValid[\s\S]*?not reconstructible/);
assert.match(state, /offlineTalentDotResolvedTotalMatchesAura[\s\S]*?fireballAbilityCode\(\)[\s\S]*?offlineOverloadScaledPositiveMetric/);
assert.match(state, /schemaVersion >= <uint>108[\s\S]*?offlineDotResolvedTotals\.add\(reader\.signed/);
assert.match(state, /normalizeOfflineLegacyDotTalentRows\(state\)/);
assert.match(state, /landOfflineFireballProjectile[\s\S]*?offlineTalentDotProjectileResolvedTotal[\s\S]*?offlineOverloadScaledPositiveMetric/);
assert.match(state, /landOfflinePureDotProjectile[\s\S]*?dotProfileFromResolvedTotal[\s\S]*?appendOfflinePureDot/);
assert.match(state, /pub dotTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /dotTalentScalingRuntimeStateTest[\s\S]*?tampered\.offlineDotResolvedTotals\[0\] = resolvedTotal \+ 1[\s\S]*?offlineDotStateIsValid\(tampered\)/);
assert.match(state, /normalizeOfflineLegacyDotTalentRows\(state\)/);
assert.match(state, /encodeStateVersion\(legacy, <uint>107\)/);
assert.match(state, /encodeStateVersion\(overload, <uint>107\)[\s\S]*?offlineDotRanks\[0\] != <uint>0[\s\S]*?offlineDotRanks\[1\] != <uint>0/);
assert.match(testMain, /state\.dotTalentScalingRuntimeStateTest\(\)/);
assert.equal(manifest.backend, 'zr_vm:project');

process.stdout.write('WOS208 dot talent scaling static guard passed\n');
