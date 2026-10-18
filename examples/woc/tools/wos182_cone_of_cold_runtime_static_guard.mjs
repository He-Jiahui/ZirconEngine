import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = resolve(projectRoot, '..', '..');
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const source = (path) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], { encoding: 'utf8' },
);

const classes = source('src/sim/content/classes.ts');
assert.match(classes, /cone_of_cold:[\s\S]*?learnLevel: 10[\s\S]*?cost: 60[\s\S]*?cooldown: 20[\s\S]*?school: 'frost'[\s\S]*?type: 'aoeDamage', min: 28, max: 36, radius: 8/);
const dispatcher = source('src/sim/combat/effect_dispatch.ts');
assert.match(dispatcher, /const aoeCrit =[\s\S]*?\(eff\.canCrit \?\? false\)[\s\S]*?aoeTargets\.length > 0/);
assert.match(dispatcher, /case 'aoeDamage':[\s\S]*?let dmg = ctx\.rng\.range\(eff\.min, eff\.max\) \+ aoeSpBonus;[\s\S]*?if \(isSpell\) dmg \*= spellDamageMultFromAuras\(p\);/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /coneOfColdPayloadAbilityIsExact[\s\S]*?abilityLength == <uint>12/);
assert.match(world, /startOfflineConeOfColdCast[\s\S]*?catalogAdmission[\s\S]*?entityResources\[casterIndex\] < 60[\s\S]*?20000000[\s\S]*?radius = 8\.0[\s\S]*?minimum = offlineOverloadScaledPositiveMetric\(28\.0, overloadMultiplier\)[\s\S]*?maximum = offlineOverloadScaledPositiveMetric\(36\.0, overloadMultiplier\)[\s\S]*?projection\.addSource\([\s\S]*?entitySpellDamageMultipliers\[casterIndex\][\s\S]*?groundAoE\.eligibleTargetCount\(projection, pulse\)[\s\S]*?projection\.randomUnits\.add\(nextAuthoritativeRandomUnit\(state\)\)[\s\S]*?groundAoE\.castInstantAoE\(projection, pulse\)/);
assert.match(world, /offlineAbilitySchoolCode[\s\S]*?coneOfColdAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FROST/);
assert.match(world, /applySupportedCastSlotCommand[\s\S]*?coneOfColdAbilityCode\(\)[\s\S]*?startOfflineConeOfColdCast/);
assert.match(world, /coneOfColdPayloadAbilityIsExact[\s\S]*?startOfflineConeOfColdCast/);
assert.match(world, /pub coneOfColdRuntimeStateTest\([\s\S]*?entitySpellDamageMultipliers\[0\] = 1\.2[\s\S]*?state\.rngDraws != <uint>2[\s\S]*?whiff[\s\S]*?whiff\.rngDraws == <uint>0/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'cone_of_cold_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_cone_of_cold_runtime_tests.zrp');
assert.match(testMain, /state\.coneOfColdRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/cone_of_cold_runtime_test_main\.zr"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '182-cone-of-cold-runtime.md');
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS182 Cone of Cold runtime static guards passed\n');
