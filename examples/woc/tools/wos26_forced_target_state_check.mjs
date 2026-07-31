import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const targeting = gitShow('src/sim/mob/targeting.ts');
const sim = gitShow('src/sim/sim.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'forcedTargetId: null,',
  'forcedTargetTimer: 0,',
  'shuffleTargetTimer: 0,',
]) {
  invariant(entity.includes(needle), `pinned Entity forced-target field drifted: ${needle}`);
}
for (const needle of [
  'mob.forcedTargetTimer -= DT;',
  'const forced = mob.forcedTargetId',
  'mob.shuffleTargetTimer =',
]) {
  invariant(targeting.includes(needle), `pinned forced-target reducer drifted: ${needle}`);
}
for (const needle of [
  'mob.forcedTargetId = p.id;',
  'mob.forcedTargetTimer = TAUNT_FORCE_SECONDS;',
]) {
  invariant(sim.includes(needle), `pinned taunt forced-target transition drifted: ${needle}`);
}

assertCatalog(encounter, 'Eastbrook', 24, (row) => row.forced_target);
assertCatalog(freshPlayers, 'fresh player', 9, (row) => row.forced_target);

for (const field of [
  'entityForcedTargetIds: container.Array<uint>;',
  'entityForcedTargetTimers: container.Array<float>;',
  'entityShuffleTargetTimers: container.Array<float>;',
]) {
  invariant(state.includes(`pub var ${field}`), `WOS26 forced-target column is missing: ${field}`);
}
for (const needle of [
  'appendDefaultForcedTargetColumns(this);',
  'appendDefaultForcedTargetColumns(state);',
  'writer.u16(<uint>38, 1, 1);',
  'schemaVersion != <uint>26',
  'if (schemaVersion >= <uint>26) {',
  'm8FreshPlayerStats.forcedTargetId',
  'm8EastbrookEncounter.forcedTargetDecimal',
  'entityState.entityForcedTargetIds[0] = <uint>800;',
  'entityState.entityForcedTargetTimers[0] = 3.0;',
]) {
  invariant(state.includes(needle), `WOS26 forced-target projection omitted: ${needle}`);
}
invariant((state.match(/entityForcedTargetIds/g) ?? []).length >= 9,
  'WOS26 forced target lacks persistence coverage');
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the WOS38 snapshot version');

process.stdout.write(`checked WOS26 forced-target source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count, project) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    const forcedTarget = project(row);
    invariant(forcedTarget && typeof forcedTarget === 'object', `${label} forced target is missing`);
    invariant(forcedTarget.forced_target_id === 0 && forcedTarget.forced_target_timer === 0 &&
      forcedTarget.shuffle_target_timer === 0, `${label} forced-target initializer drifted`);
  }
}

function readCatalog(relativePath) {
  return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8'));
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
