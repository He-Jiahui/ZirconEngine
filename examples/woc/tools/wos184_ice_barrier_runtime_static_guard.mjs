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
assert.match(
  classes,
  /ice_barrier:[\s\S]*?specs: \['frost'\][\s\S]*?learnLevel: 5[\s\S]*?cost: 90[\s\S]*?castTime: 0[\s\S]*?cooldown: 30[\s\S]*?range: 0[\s\S]*?school: 'frost'[\s\S]*?requiresTarget: false[\s\S]*?type: 'absorb', amount: 130, duration: 60/,
);
const dispatcher = source('src/sim/combat/effect_dispatch.ts');
assert.match(
  dispatcher,
  /case 'absorb':[\s\S]*?const shieldTarget = target \?\? p;[\s\S]*?id: hasStasisSelfBuff \? `\$\{ability\.id\}_absorb` : ability\.id[\s\S]*?remaining: eff\.duration[\s\S]*?value: eff\.amount[\s\S]*?sourceId: p\.id[\s\S]*?school: ability\.school/,
);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const iceBarrierCast = world.match(/startOfflineIceBarrierCast\([\s\S]*?\r?\n}\r?\n\r?\n/);
assert.ok(iceBarrierCast, 'Ice Barrier cast reducer must be present');
assert.match(world, /iceBarrierAbilityCode[\s\S]*?abilityCode\("ice_barrier"\)[\s\S]*?!= "mage"/);
assert.match(world, /iceBarrierPayloadAbilityIsExact[\s\S]*?abilityLength == <uint>11/);
assert.match(world, /iceBarrierAbsorbProfileMatches[\s\S]*?rank == 1[\s\S]*?amount <= 130[\s\S]*?remaining <= 60\.0/);
assert.match(
  iceBarrierCast[0],
  /entityCastingAbility\[casterIndex\] != <uint>0[\s\S]*?entityCastGcdRemaining\[casterIndex\] > 0\.0[\s\S]*?catalogAdmission[\s\S]*?entityResources\[casterIndex\] < 90[\s\S]*?30000000[\s\S]*?applyOfflineFixedAbsorb\([\s\S]*?offlineOverloadScaledPositiveMetric\(130\.0, overloadMultiplier\), 60\.0/,
);
assert.match(world, /offlineAbilitySchoolCode[\s\S]*?iceBarrierAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FROST/);
assert.match(world, /applySupportedCastSlotCommand[\s\S]*?iceBarrierAbilityCode\(\)[\s\S]*?startOfflineIceBarrierCast/);
assert.match(world, /iceBarrierPayloadAbilityIsExact[\s\S]*?startOfflineIceBarrierCast/);
assert.match(world, /arcanePowerGlobalCooldownSeconds[\s\S]*?OFFLINE_SOURCE_MIN_GLOBAL_COOLDOWN_SECONDS/);
assert.match(world, /offlineAbsorbProfileMatches[\s\S]*?powerWordShieldAbilityCode\(\)[\s\S]*?iceBarrierAbsorbProfileMatches/);
assert.match(world, /applyOfflineFixedAbsorb[\s\S]*?offlineAbsorbTargetIds\[index\] == targetId[\s\S]*?offlineAbsorbSourceIds\[index\] == sourceId[\s\S]*?offlineAbsorbAbilityCodes\[index\] == abilityCode[\s\S]*?removeOfflineAbsorbAt/);
assert.match(world, /applyOfflineAbsorbDamage[\s\S]*?offlineAbsorbProfileAndDurationMatches[\s\S]*?removeOfflineAbsorbAt/);
assert.match(world, /pub iceBarrierRuntimeStateTest\([\s\S]*?specCode\("mage", "frost"\)[\s\S]*?appendTypedCastCommandForTest[\s\S]*?appendCastSlotCommand[\s\S]*?30000000[\s\S]*?offlineAbsorbAmounts\[0\] != 130[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?applyOfflineAbsorbDamage[\s\S]*?entitySpellHaste\[0\] = 1\.5[\s\S]*?0\.75[\s\S]*?offlineAbsorbStateIsValid/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'ice_barrier_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_ice_barrier_runtime_tests.zrp');
assert.match(testMain, /state\.iceBarrierRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/ice_barrier_runtime_test_main\.zr"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '184-ice-barrier-runtime.md');
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS184 Ice Barrier runtime static guards passed\n');
