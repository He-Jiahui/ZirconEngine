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
  /overload:[\s\S]*?class: 'mage',[\s\S]*?learnLevel: 14,[\s\S]*?cost: 0,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 30,[\s\S]*?school: 'arcane',[\s\S]*?offGcd: true,[\s\S]*?kind: 'overload', value: 0\.4, duration: 10/,
);
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
assert.match(lifecycle, /consumeOverload[\s\S]*?res\.def\.school === 'physical' \|\| res\.cost <= 0/);
assert.match(lifecycle, /applyAbility[\s\S]*?res = consumeOverload\(ctx, p, res\)/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityOverloadRemaining[\s\S]*?entityOverloadValues/);
assert.match(world, /overloadStateIsValid[\s\S]*?entityOverloadRemaining[\s\S]*?entityOverloadValues/);
assert.match(world, /startOfflineOverloadCast[\s\S]*?overloadAbilityCode[\s\S]*?entityOverloadRemaining[\s\S]*?10\.0[\s\S]*?entityOverloadValues[\s\S]*?0\.4[\s\S]*?setAbilityCooldownExpiration/);
const overloadStart = world.slice(
  world.indexOf('startOfflineOverloadCast'), world.indexOf('startOfflineIcyVeinsCast'),
);
assert.doesNotMatch(overloadStart, /entityCastGcdRemaining/);
assert.match(world, /ageOfflineOverload[\s\S]*?0\.05[\s\S]*?entityOverloadValues/);
assert.match(world, /schemaVersion >= <uint>92[\s\S]*?entityOverloadRemaining[\s\S]*?entityOverloadValues/);
// WOS190 extends the same authority stream with a projectile DoT snapshot.
// Keep WOS189's guarantee that its schema boundary remains present while
// accepting the current forward-compatible encoder revision.
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /pub overloadArmingRuntimeStateTest\(/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'overload_arming_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_overload_arming_runtime_tests.zrp');
assert.match(testMain, /state\.overloadArmingRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/overload_arming_runtime_test_main\.zr"/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '189-overload-arming-runtime.md');
assert.match(plan, /WOS190/);
assert.match(plan, /off-GCD/);

process.stdout.write('WOS189 Overload arming runtime static guards passed\n');
