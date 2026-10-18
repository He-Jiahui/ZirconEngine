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
const source = (path) => execFileSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], { encoding: 'utf8' });

assert.match(source('src/sim/content/classes.ts'), /blazing_barrier:[\s\S]*?learnLevel: 5[\s\S]*?specs: \['fire'\][\s\S]*?cost: 90[\s\S]*?castTime: 0[\s\S]*?cooldown: 30[\s\S]*?school: 'fire'[\s\S]*?requiresTarget: false[\s\S]*?type: 'absorb', amount: 130, duration: 60/);
assert.match(source('src/sim/combat/fire_mage.ts'), /PERSONAL_BARRIER_IDS[\s\S]*?'ice_barrier'[\s\S]*?'blazing_barrier'/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const cast = world.match(/startOfflineBlazingBarrierCast\([\s\S]*?\r?\n}\r?\n\r?\n/);
assert.ok(cast, 'Blazing Barrier cast reducer must be present');
assert.match(world, /blazingBarrierPayloadAbilityIsExact[\s\S]*?abilityLength == <uint>15/);
assert.match(cast[0], /catalogAdmission[\s\S]*?entityResources\[casterIndex\] < 90[\s\S]*?30000000[\s\S]*?applyOfflineFixedAbsorb\([\s\S]*?offlineOverloadScaledPositiveMetric\(130\.0, overloadMultiplier\), 60\.0/);
assert.match(world, /offlineAbilitySchoolCode[\s\S]*?blazingBarrierAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/);
assert.match(world, /applySupportedCastSlotCommand[\s\S]*?blazingBarrierAbilityCode\(\)[\s\S]*?startOfflineBlazingBarrierCast/);
assert.match(world, /blazingBarrierPayloadAbilityIsExact[\s\S]*?startOfflineBlazingBarrierCast/);
assert.match(world, /offlineAbsorbProfileMatches[\s\S]*?blazingBarrierAbsorbProfileMatches/);
assert.match(world, /offlineWardedPersonalBarrierIsActive[\s\S]*?iceBarrierAbilityCode\(\)[\s\S]*?blazingBarrierAbilityCode\(\)/);
assert.match(world, /applyOfflineWardedShieldConsumed[\s\S]*?blazingBarrierAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityId/);
assert.match(world, /pub blazingBarrierRuntimeStateTest\([\s\S]*?specCode\("mage", "fire"\)[\s\S]*?appendTypedCastCommandForTest[\s\S]*?appendCastSlotCommand[\s\S]*?30000000[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/);

assert.match(read('scripts', 'woc_game', 'src', 'world', 'blazing_barrier_runtime_test_main.zr'), /state\.blazingBarrierRuntimeStateTest\(\)/);
const testPackage = read('scripts', 'woc_game', 'woc_m4_blazing_barrier_runtime_tests.zrp');
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/blazing_barrier_runtime_test_main\.zr"/);
assert.match(read('..', '..', 'docs', 'plans', 'woc', '186-blazing-barrier-runtime.md'), /`zr_vm:project`/);
process.stdout.write('WOS186 Blazing Barrier runtime static guards passed\n');
