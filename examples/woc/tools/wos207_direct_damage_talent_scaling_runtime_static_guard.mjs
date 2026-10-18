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
  'direct_damage_talent_scaling_state.zr',
);

assert.match(
  readSource('src', 'sim', 'content', 'classes.ts'),
  /case 'directDamage':[\s\S]*?min: Math\.round\(eff\.min \* dmgMult \+ flat\)[\s\S]*?max: Math\.round\(eff\.max \* dmgMult \+ flat\)/,
);
assert.match(
  readSource('src', 'sim', 'content', 'classes.ts'),
  /case 'consumeAura':[\s\S]*?deal: eff\.deal[\s\S]*?Math\.round\(eff\.deal\.min \* dmgMult \+ flat\)/,
);
assert.ok(existsSync(scalingPath), 'WOS207 direct-damage talent scaling module is missing');

const scaling = read('scripts', 'woc_game', 'src', 'combat', 'direct_damage_talent_scaling_state.zr');
const state = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const testMain = read('scripts', 'woc_game', 'src', 'world', 'direct_damage_talent_runtime_test_main.zr');
const manifest = JSON.parse(read('scripts', 'woc_game', 'woc_m4_direct_damage_talent_runtime_tests.zrp'));

assert.match(scaling, /pub resolveDirectDamageRange\(/);
assert.match(scaling, /"meleeDmgPct"/);
assert.match(scaling, /"spellDmgPct"/);
assert.match(scaling, /"dmgPct"/);
assert.match(scaling, /"flatDmg"/);
assert.match(scaling, /endpointFirst != 63[\s\S]*?postRoll != 62/);
assert.match(
  scaling,
  /directDamageRangeSnapshotMatches\([\s\S]*?minimum == expectedMinimum && maximum == expectedMaximum/,
);
assert.doesNotMatch(scaling, /maximum <= 65535/);
assert.match(state, /var directDamageTalentScaling = %import\("combat\/direct_damage_talent_scaling_state"\);/);
assert.match(state, /resolveOfflineTalentDirectDamageRange\(/);
assert.match(state, /appendOfflineAbilityProjectile\([\s\S]*?resolveOfflineTalentDirectDamageRange/);
assert.match(state, /offlineProjectileTalentSpecCodes[\s\S]*?offlineProjectileTalentRowOptionCodes/);
assert.match(state, /appendOfflineProjectileTalentSnapshot\([\s\S]*?offlineTalentAllocationIsValid/);
assert.match(
  state,
  /schemaVersion < <uint>107[\s\S]*?!offlineProjectileTalentSnapshotsAreLegacyCompatible\(state\)[\s\S]*?cannot be encoded by a historical schema/,
);
assert.match(
  state,
  /offlineProjectileTalentSnapshotsAreLegacyCompatible\([\s\S]*?offlineProjectileTalentSnapshotIsEmpty/,
);
assert.match(state, /offlineTalentDirectDamageProjectileRangeIsValid[\s\S]*?offlineTalentDirectDamageSnapshotModifiers[\s\S]*?directDamageRangeSnapshotMatches/);
assert.match(state, /completeOfflineFireballCast[\s\S]*?resolveOfflineTalentDirectDamageRange[\s\S]*?offlineOverloadTalentDirectDamageRange[\s\S]*?launchRange/);
assert.match(state, /launchOfflinePyroblastProjectile[\s\S]*?resolveOfflineTalentDirectDamageRange[\s\S]*?offlineOverloadTalentDirectDamageRange[\s\S]*?launchRange/);
assert.match(state, /offlineTalentDirectDamageProjectileRangeIsValid\(/);
assert.match(state, /landOfflineConflagrateProjectile[\s\S]*?spell\.minimum = <float>state\.offlineProjectileMinimums\[projectileIndex\]/);
assert.match(state, /pub directDamageTalentRuntimeStateTest\(\): int/);
assert.match(state, /directDamageTalentRuntimeStateTest[\s\S]*?tampered\.offlineProjectileMinimums\[0\] = 1\.0[\s\S]*?offlineProjectileStateIsValid\(tampered\)/);
assert.match(
  state,
  /offlineProjectileTalentSpecCodes\[0\] != frost[\s\S]*?offlineProjectileTalentSnapshotsAreLegacyCompatible\(state\)/,
);
assert.match(state, /fireballTalentRange[\s\S]*?fireballReverseRange[\s\S]*?pyroblastTalentRange[\s\S]*?pyroblastReverseRange/);
assert.match(state, /legacyRestored = decodeState\(encodeStateVersion\(legacy, <uint>106\)\)[\s\S]*?offlineProjectileTalentSnapshotsAreLegacyCompatible\(legacyRestored\)/);
assert.match(testMain, /state\.directDamageTalentRuntimeStateTest\(\)/);
assert.equal(manifest.backend, 'zr_vm:project');

process.stdout.write('WOS207 direct-damage talent scaling static guard passed\n');
