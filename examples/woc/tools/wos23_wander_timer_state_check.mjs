import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const locomotion = gitShow('src/sim/mob/locomotion.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = JSON.parse(readFileSync(
  resolve(wocRoot, 'reference', 'current-head', 'm8_eastbrook_encounter.json'), 'utf8',
));

for (const needle of [
  'wanderTarget: null,',
  'wanderTimer: 0,',
]) {
  invariant(entity.includes(needle), `pinned Entity wander field drifted: ${needle}`);
}
for (const needle of [
  "case 'idle': {",
  'mob.wanderTimer -= DT;',
  'if (mob.wanderTimer <= 0) {',
  'mob.wanderTimer = ctx.rng.range(3, 10);',
  'mob.wanderTimer = 30;',
]) {
  invariant(locomotion.includes(needle), `pinned idle-wander behavior drifted: ${needle}`);
}

invariant(encounter.schema_version === 17 && encounter.spawns.length === 24,
  'Eastbrook encounter catalog drifted');
for (const spawn of encounter.spawns) {
  invariant(Number.isFinite(spawn.wander_timer) && spawn.wander_timer >= 0,
    `Eastbrook wander timer is invalid: ${spawn.source_entity_id}`);
}

for (const needle of [
  'pub var entityWanderTimers: container.Array<float>;',
  'this.entityWanderTimers = new container.Array<float>();',
  'appendDefaultWanderColumns(this);',
  'appendDefaultWanderColumns(state);',
  'writer.u16(<uint>38, 1, 1);',
  'schemaVersion != <uint>23',
  'if (schemaVersion >= <uint>23) {',
  'state.entityWanderTimers[entityIndex] = m8EastbrookEncounter.wanderTimer(spawnIndex);',
  'entityState.entityWanderTimers[0] = 4.75;',
]) {
  invariant(state.includes(needle), `WOS23 wander-timer projection omitted: ${needle}`);
}
invariant((state.match(/entityWanderTimers/g) ?? []).length >= 9,
  'WOS23 wander timer lacks persistence coverage');
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the WOS38 snapshot version');

process.stdout.write(`checked WOS23 wander-timer source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
