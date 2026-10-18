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
  'direct_heal_talent_scaling_state.zr',
);

assert.match(
  readSource('src', 'sim', 'content', 'classes.ts'),
  /case 'heal':[\s\S]*?min: Math\.round\(eff\.min \* healMult \+ flat\)[\s\S]*?max: Math\.round\(eff\.max \* healMult \+ flat\)/,
);
assert.match(
  readSource('src', 'sim', 'content', 'classes.ts'),
  /case 'consumeAura':[\s\S]*?heal: eff\.heal[\s\S]*?Math\.round\(eff\.heal\.min \* healMult \+ \(eff\.deal \? 0 : flat\)\)/,
);
assert.ok(existsSync(scalingPath), 'WOS206 direct-heal talent scaling module is missing');

const scaling = read('scripts', 'woc_game', 'src', 'combat', 'direct_heal_talent_scaling_state.zr');
const state = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const testMain = read('scripts', 'woc_game', 'src', 'world', 'direct_heal_talent_runtime_test_main.zr');
const manifest = JSON.parse(read('scripts', 'woc_game', 'woc_m4_direct_heal_talent_runtime_tests.zrp'));

assert.match(scaling, /pub resolveDirectHealRange\(/);
assert.match(scaling, /"healPct"/);
assert.match(scaling, /"dmgPct"/);
assert.match(scaling, /"flatDmg"/);
assert.match(scaling, /resolveDirectHealRange\(230, 270[\s\S]*?holy\.minimum != 276[\s\S]*?holy\.maximum != 324/);
assert.match(scaling, /endpoints\.minimum == 56[\s\S]*?endpoints\.maximum == 69[\s\S]*?endpointFirst == 63[\s\S]*?postRoll == 62/);
assert.match(state, /var directHealTalentScaling = %import\("combat\/direct_heal_talent_scaling_state"\);/);
assert.match(state, /applyOfflineDirectHeal[\s\S]*?offlineTalentModifiersForEntity[\s\S]*?resolveDirectHealRange/);
assert.match(state, /numeric\.randomUnits\[0\][\s\S]*?numeric\.healAmounts\[0\]/);
assert.match(state, /pub directHealTalentRuntimeStateTest\(\): int/);
assert.match(
  state,
  /directHealTalentRuntimeStateTest[\s\S]*?state\.entitySpellPower\[0\] = 120;[\s\S]*?state\.entityCritChance\[0\] = 1\.0;/,
);
assert.match(state, /swiftmendRange\.minimum != 126[\s\S]*?swiftmendRange\.maximum != 150[\s\S]*?applyOfflineDirectHeal\([\s\S]*?swiftmendIndex/);
assert.match(
  state,
  /swiftmendState\.entitySpellPower\[0\] = 120;[\s\S]*?swiftmendState\.entityCritChance\[0\] = 1\.0;/,
);
assert.doesNotMatch(state, /applyOfflineResolvedDirectHeal[\s\S]{0,350}resolveDirectHealRange/);
assert.match(testMain, /state\.directHealTalentRuntimeStateTest\(\)/);
assert.equal(manifest.backend, 'zr_vm:project');

process.stdout.write('WOS206 direct-heal talent scaling static guard passed\n');
