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
assert.match(classes, /cold_snap:[\s\S]*?learnLevel: 17[\s\S]*?cost: 0[\s\S]*?castTime: 0[\s\S]*?cooldown: 120[\s\S]*?school: 'frost'[\s\S]*?offGcd: true[\s\S]*?type: 'clearCooldowns',[\s\S]*?'blink',[\s\S]*?'ice_barrier',[\s\S]*?'blazing_barrier',[\s\S]*?'temporal_barrier',[\s\S]*?'greater_invisibility'/);
const dispatcher = source('src/sim/combat/effect_dispatch.ts');
assert.match(dispatcher, /case 'clearCooldowns':[\s\S]*?p\.cooldowns\.delete\(abilityId\)/);
const casting = source('src/sim/combat/casting_lifecycle.ts');
assert.match(casting, /if \(p\.castingAbility\) \{[\s\S]*?if \(!blinkThrough\)/);
assert.match(casting, /if \(!ability\.offGcd && p\.gcdRemaining > 0 && !blinkThrough\) return/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const coldSnapCast = world.match(/startOfflineColdSnapCast\([\s\S]*?\r?\n}\r?\n\r?\n/);
assert.ok(coldSnapCast, 'Cold Snap cast reducer must be present');
assert.match(coldSnapCast[0], /entityCastingAbility\[casterIndex\] != <uint>0/);
assert.doesNotMatch(coldSnapCast[0], /entityCastGcdRemaining/);
assert.match(world, /coldSnapPayloadAbilityIsExact[\s\S]*?abilityLength == <uint>9/);
assert.match(world, /coldSnapResetAbilityCodes[\s\S]*?codes\.add\(blink\)[\s\S]*?codes\.add\(iceBarrier\)[\s\S]*?codes\.add\(blazingBarrier\)[\s\S]*?codes\.add\(temporalBarrier\)[\s\S]*?codes\.add\(greaterInvisibility\)/);
assert.match(coldSnapCast[0], /catalogAdmission[\s\S]*?120000000[\s\S]*?coldSnapResetAbilityCodes\(\)[\s\S]*?clearAbilityCooldown/);
assert.match(world, /offlineAbilitySchoolCode[\s\S]*?coldSnapAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FROST/);
assert.match(world, /applySupportedCastSlotCommand[\s\S]*?coldSnapAbilityCode\(\)[\s\S]*?startOfflineColdSnapCast/);
assert.match(world, /coldSnapPayloadAbilityIsExact[\s\S]*?startOfflineColdSnapCast/);
assert.match(world, /pub coldSnapRuntimeStateTest\([\s\S]*?spawnPlayer\(<uint>2, 16[\s\S]*?appendTypedCastCommandForTest[\s\S]*?state\.entityLevels\[0\] = 17[\s\S]*?entityCastingAbility\[0\] = unrelated[\s\S]*?appendCastSlotCommand[\s\S]*?entityCastingAbility\[0\] = <uint>0[\s\S]*?120000000[\s\S]*?decodeState\(encodeState\(state\)\)/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'cold_snap_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_cold_snap_runtime_tests.zrp');
assert.match(testMain, /state\.coldSnapRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/cold_snap_runtime_test_main\.zr"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '183-cold-snap-runtime.md');
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS183 Cold Snap runtime static guards passed\n');
