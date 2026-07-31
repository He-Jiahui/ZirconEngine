import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/mob/locomotion.ts');
const projection = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'eastbrook_idle_wander_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'eastbrook_idle_wander_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m8_eastbrook_idle_wander_state_tests.zrp'), 'utf8'));

for (const needle of [
  'mob.wanderTimer -= DT;',
  'mob.wanderTarget = null;',
  'mob.wanderTimer = ctx.rng.range(3, 10);',
  'const ang = ctx.rng.range(0, Math.PI * 2);',
  'const r = ctx.rng.range(2, 9);',
  'mob.wanderTimer = 30;',
  'ctx.moveToward(mob, mob.wanderTarget, mob.moveSpeed * 0.35);',
]) invariant(source.includes(needle), `source Eastbrook idle-wander rule drifted: ${needle}`);

for (const needle of [
  'pub class EastbrookIdleWanderState {',
  'pub wanderRange(minimum: float, maximum: float, unit: float): float {',
  'pub stepEastbrookIdleWander(',
  'state.wanderTimer = state.wanderTimer - 0.05;',
  'var angle = wanderRange(0.0, 6.283185307179586, firstUnit);',
  'var radius = wanderRange(2.0, 9.0, secondUnit);',
  'state.wanderTimer = 30.0;',
  'var arrivalUnit = draws == 2 ? thirdUnit : firstUnit;',
  'var step = state.moveSpeed * 0.35 * 0.05;',
  'stepEastbrookIdleWander(state, 0.25, 0.5, 0.0) != 2',
  'stepEastbrookIdleWander(state, 0.25, 0.5, 0.75) != 3',
]) invariant(projection.includes(needle), `WOC Eastbrook idle-wander projection is missing: ${needle}`);

for (const needle of ['%import("world/eastbrook_idle_wander_state")', 'wander.contractTest()']) invariant(testMain.includes(needle), `WOC Eastbrook idle-wander test is missing: ${needle}`);
invariant(testProject.name === 'woc_m8_eastbrook_idle_wander_state_tests' && testProject.source === 'src' && testProject.binary === 'bin-m8-eastbrook-idle-wander-state-tests' && testProject.entry === 'world/eastbrook_idle_wander_state_test_main', 'Eastbrook idle-wander test project contract drifted');
process.stdout.write(`checked M8 Eastbrook idle-wander source: ${SOURCE_COMMIT.slice(0, 15)}\n`);
function gitShow(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' }); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
