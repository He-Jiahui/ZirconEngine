import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const locomotion = gitShow('src/sim/mob/locomotion.ts');
const sim = gitShow('src/sim/sim.ts');
const rng = gitShow('src/sim/rng.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const wander = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'eastbrook_idle_wander_state.zr'),
  'utf8',
);
const cursor = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'eastbrook_rng_cursor.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'eastbrook_rng_cursor_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m8_eastbrook_rng_cursor_tests.zrp'), 'utf8'),
);

for (const needle of [
  'mob.wanderTimer -= DT;',
  'const ang = ctx.rng.range(0, Math.PI * 2);',
  'const r = ctx.rng.range(2, 9);',
  'mob.wanderTimer = 30;',
  'const arrived = ctx.moveToward(mob, mob.wanderTarget, mob.moveSpeed * 0.35);',
  'mob.wanderTimer = ctx.rng.range(3, 10);',
]) {
  invariant(locomotion.includes(needle), `source Eastbrook wander drifted: ${needle}`);
}

for (const needle of [
  'for (const e of this.entities.values()) {',
  "if (e.kind === 'mob') {",
  'this.updateMob(e);',
]) {
  invariant(sim.includes(needle), `source mob-pass ordering drifted: ${needle}`);
}

for (const needle of [
  'private s: number;',
  'let t = (this.s += 0x6d2b79f5);',
  'if (this.s === 0) this.s = 0x9e3779b9;',
]) {
  invariant(rng.includes(needle), `source Mulberry32 cursor drifted: ${needle}`);
}

for (const needle of [
  'pub constructorCursorAfterCampSpawns(',
  'pub currentCampSpawnDrawsPerMob(',
  'pub contractTest(): int',
]) {
  invariant(cursor.includes(needle), `WOC Eastbrook RNG cursor is missing: ${needle}`);
}

for (const needle of [
  'pub var entityWanderTargetPresent: container.Array<bool>;',
  'pub var entityWanderTargetX: container.Array<float>;',
  'pub var entityWanderTargetZ: container.Array<float>;',
  'WOS50',
  'stepOfflineEastbrookMobWander(state);',
  'eastbrookRngCursor.constructorCursorAfterCampSpawns(',
]) {
  invariant(state.includes(needle), `WOS wander migration is missing: ${needle}`);
}

for (const needle of [
  'pub stepEastbrookIdleWanderWithWorldMotion(',
  'mobMotion.stepMobToward(',
]) {
  invariant(wander.includes(needle), `WOC idle-wander transition is missing: ${needle}`);
}

for (const needle of [
  '%import("world/eastbrook_rng_cursor")',
  'cursor.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC RNG cursor test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m8_eastbrook_rng_cursor_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m8-eastbrook-rng-cursor-tests' &&
    testProject.entry === 'world/eastbrook_rng_cursor_test_main',
  'Eastbrook RNG cursor test project contract drifted',
);

process.stdout.write(`checked M8 Eastbrook WOS wander source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
