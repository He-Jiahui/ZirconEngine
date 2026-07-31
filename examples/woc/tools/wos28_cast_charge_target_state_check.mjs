import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const casting = gitShow('src/sim/combat/casting_lifecycle.ts');
const sim = gitShow('src/sim/sim.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'castAim: null,', 'queuedCastAim: null,', 'chargeTargetId: null,',
  'chargeTimeLeft: 0,', 'chargePath: [],', 'followTargetId: null,',
]) invariant(entity.includes(needle), `pinned cast-charge field drifted: ${needle}`);
for (const needle of ['p.castAim = null;', 'p.queuedCastAim = null;']) {
  invariant(casting.includes(needle), `pinned cast-aim lifecycle drifted: ${needle}`);
}
for (const needle of ['p.chargeTargetId === null', 'p.chargeTimeLeft -= DT;', 'p.followTargetId === null']) {
  invariant(sim.includes(needle), `pinned charge/follow behavior drifted: ${needle}`);
}

assertCatalog(encounter, 'Eastbrook', 24, (row) => row.cast_charge_target);
assertCatalog(freshPlayers, 'fresh player', 9, (row) => row.cast_charge_target);

for (const field of [
  'entityCastAimPresent: container.Array<bool>;', 'entityCastAimX: container.Array<float>;',
  'entityCastAimY: container.Array<float>;', 'entityCastAimZ: container.Array<float>;',
  'entityQueuedCastAimPresent: container.Array<bool>;', 'entityQueuedCastAimX: container.Array<float>;',
  'entityQueuedCastAimY: container.Array<float>;', 'entityQueuedCastAimZ: container.Array<float>;',
  'entityChargeTargetIds: container.Array<uint>;', 'entityChargeTimeLeft: container.Array<float>;',
  'entityFollowTargetIds: container.Array<uint>;',
]) invariant(state.includes(`pub var ${field}`), `WOS28 column is missing: ${field}`);
for (const needle of [
  'appendDefaultCastChargeTargetColumns(this);', 'appendDefaultCastChargeTargetColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>28',
  'if (schemaVersion >= <uint>28) {', 'm8FreshPlayerStats.castChargeTargetFlag',
  'm8EastbrookEncounter.castChargeTargetId', 'entityState.entityCastAimPresent[0] = true;',
  'entityState.entityChargeTimeLeft[0] = 1.5;',
]) invariant(state.includes(needle), `WOS28 cast-charge projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS28 cast-charge source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count, project) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    const value = project(row);
    invariant(value && typeof value === 'object', `${label} cast-charge target is missing`);
    invariant(value.cast_aim_present === false && value.queued_cast_aim_present === false &&
      value.charge_target_id === 0 && value.charge_time_left === 0 && value.follow_target_id === 0,
    `${label} cast-charge initializer drifted`);
  }
}
function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' }); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
