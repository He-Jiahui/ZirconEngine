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
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'leashAnchor: null,',
  'evadeStall: 0,',
  'fleeTimer: 0,',
  'fleeReturnTimer: 0,',
  'hasFled: false,',
]) {
  invariant(entity.includes(needle), `pinned Entity locomotion-recovery field drifted: ${needle}`);
}
for (const needle of [
  'const leashAnchor = mob.leashAnchor ?? mob.spawnPos;',
  'mob.fleeTimer -= DT;',
  'mob.evadeStall += DT;',
  'function resetEvadingMob(',
  'export function recoverFromFlee(',
]) {
  invariant(locomotion.includes(needle), `pinned locomotion-recovery behavior drifted: ${needle}`);
}

assertCatalog(encounter, 'Eastbrook', 24, (row) => row.locomotion_recovery);
assertCatalog(freshPlayers, 'fresh player', 9, (row) => row.locomotion_recovery);

for (const field of [
  'entityLeashAnchorPresent: container.Array<bool>;',
  'entityLeashAnchorX: container.Array<float>;',
  'entityLeashAnchorY: container.Array<float>;',
  'entityLeashAnchorZ: container.Array<float>;',
  'entityEvadeStalls: container.Array<float>;',
  'entityFleeTimers: container.Array<float>;',
  'entityFleeReturnTimers: container.Array<float>;',
  'entityHasFled: container.Array<bool>;',
]) {
  invariant(state.includes(`pub var ${field}`), `WOS25 locomotion column is missing: ${field}`);
}
for (const needle of [
  'appendDefaultLocomotionRecoveryColumns(this);',
  'appendDefaultLocomotionRecoveryColumns(state);',
  'writer.u16(<uint>38, 1, 1);',
  'schemaVersion != <uint>25',
  'if (schemaVersion >= <uint>25) {',
  'm8FreshPlayerStats.locomotionRecoveryFlag',
  'm8EastbrookEncounter.locomotionRecoveryDecimal',
  'entityState.entityLeashAnchorPresent[0] = true;',
  'entityState.entityFleeTimers[0] = 4.25;',
]) {
  invariant(state.includes(needle), `WOS25 locomotion-recovery projection omitted: ${needle}`);
}
invariant((state.match(/entityFleeTimers/g) ?? []).length >= 9,
  'WOS25 flee timer lacks persistence coverage');
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the WOS38 snapshot version');

process.stdout.write(`checked WOS25 locomotion-recovery source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count, project) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    const recovery = project(row);
    invariant(recovery && typeof recovery === 'object', `${label} locomotion recovery is missing`);
    invariant(recovery.leash_anchor_present === false && recovery.leash_anchor_x === 0 &&
      recovery.leash_anchor_y === 0 && recovery.leash_anchor_z === 0 &&
      recovery.evade_stall === 0 && recovery.flee_timer === 0 &&
      recovery.flee_return_timer === 0 && recovery.has_fled === false,
    `${label} locomotion recovery initializer drifted`);
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
