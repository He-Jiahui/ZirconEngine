import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const damage = gitShow('src/sim/combat/damage.ts');
const lifecycle = gitShow('src/sim/mob/lifecycle.ts');
const locomotion = gitShow('src/sim/mob/locomotion.ts');
const rng = gitShow('src/sim/rng.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'world', 'mob_lifecycle_state.zr'), 'utf8');
const state = readFileSync(resolve(wocRoot, 'src', 'world', 'state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'world', 'mob_lifecycle_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_mob_lifecycle_state_tests.zrp'), 'utf8'));

for (const needle of [
  "e.aiState = 'dead';",
  'e.corpseTimer = CORPSE_DURATION;',
  'e.respawnTimer =',
  'e.aggroTargetId = null;',
  'clearThreat(e);',
]) {
  invariant(damage.includes(needle), `source mob death transition drifted: ${needle}`);
}

for (const needle of [
  'export function respawnMob(ctx: SimContext, mob: Entity): void {',
  'mob.dead = false;',
  'mob.pos = { ...mob.spawnPos };',
  'mob.hp = mob.maxHp;',
  "mob.aiState = 'idle';",
  'mob.wanderTimer = ctx.rng.range(2, 8);',
]) {
  invariant(lifecycle.includes(needle), `source mob respawn transition drifted: ${needle}`);
}

for (const needle of [
  'mob.corpseTimer -= DT;',
  'mob.respawnTimer -= DT;',
  'mob.respawnTimer <= 0 && (mob.corpseTimer <= 0 || !mob.lootable)',
  'ctx.respawnMob(mob);',
]) {
  invariant(locomotion.includes(needle), `source corpse tick drifted: ${needle}`);
}

invariant(rng.includes('return min + this.next() * (max - min);'), 'source rng range drifted');

for (const needle of [
  'pub idleAiState(): uint { return <uint>1; }',
  'pub deadAiState(): uint { return <uint>6; }',
  'pub fixedTickMicros(): uint { return <uint>50000; }',
  'pub defaultCorpseMicros(): uint { return <uint>60000000; }',
  'pub defaultRespawnMicros(): uint { return <uint>30000000; }',
  'pub shouldRespawnWildMob(',
  'pub respawnWanderTimer(randomUnit: float): float {',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC mob lifecycle state is missing: ${needle}`);
}

for (const needle of [
  '%import("world/mob_lifecycle_state")',
  'stepOfflineEastbrookMobLifecycle(state);',
  'mobLifecycle.shouldStartDeath(',
  'mobLifecycle.shouldRespawnWildMob(',
  'state.entityWanderTimers[index] = mobLifecycle.respawnWanderTimer(',
  'clearStateThreat(state, index);',
  'offlineEastbrookMobLifecycleStateTest()',
]) {
  invariant(state.includes(needle), `WOS lifecycle integration is missing: ${needle}`);
}

invariant(
  testMain.includes('%import("world/mob_lifecycle_state")') && testMain.includes('lifecycle.contractTest()'),
  'missing mob lifecycle test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_mob_lifecycle_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-lifecycle-state-tests' &&
    testProject.entry === 'world/mob_lifecycle_state_test_main',
  'mob lifecycle test project contract drifted',
);

process.stdout.write(`checked M4 mob lifecycle source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
