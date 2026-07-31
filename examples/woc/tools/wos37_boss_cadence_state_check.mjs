import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const lifecycle = gitShow('src/sim/mob/lifecycle.ts');
const locomotion = gitShow('src/sim/mob/locomotion.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'pulseTimer: 0,', 'stompTimer: 0,', 'bigCastTimer: 0,',
  'yelledEngage: false,', 'stoneskinTimer: 0,',
  'if (template.stomp) e.stompTimer = template.stomp.every;',
  'if (template.stoneskin) e.stoneskinTimer = template.stoneskin.every;',
  'if (template.bigCast) e.bigCastTimer = template.bigCast.every;',
]) invariant(entity.includes(needle), 'pinned boss cadence constructor drifted: ' + needle);
for (const needle of [
  'mob.stompTimer = MOBS[mob.templateId]?.stomp?.every ?? 0;',
  'mob.stoneskinTimer = MOBS[mob.templateId]?.stoneskin?.every ?? 0;',
  'mob.bigCastTimer = bigCastDef?.every ?? 0;',
  'mob.yelledEngage = false;',
]) invariant(lifecycle.includes(needle), 'pinned boss cadence reset drifted: ' + needle);
for (const needle of [
  'mob.pulseTimer -= DT;', 'mob.stompTimer -= DT;',
  'mob.bigCastTimer -= DT;', 'mob.stoneskinTimer -= DT;',
]) invariant(locomotion.includes(needle), 'pinned boss cadence tick source drifted: ' + needle);

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

for (const field of [
  'entityBossPulseTimers: container.Array<float>;',
  'entityBossStompTimers: container.Array<float>;',
  'entityBossBigCastTimers: container.Array<float>;',
  'entityBossYelledEngage: container.Array<bool>;',
  'entityBossStoneskinTimers: container.Array<float>;',
]) invariant(state.includes('pub var ' + field), 'WOS37 boss cadence column is missing: ' + field);
for (const needle of [
  'appendDefaultBossCadenceColumns(this);',
  'appendDefaultBossCadenceColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>37',
  'if (schemaVersion >= <uint>37) {',
  'm8FreshPlayerStats.bossPulseTimerSeconds',
  'm8EastbrookEncounter.bossPulseTimerSeconds',
  'entityState.entityBossPulseTimers[0] = 2.0;',
  'entityState.entityBossStoneskinTimers[0] = 8.0;',
]) invariant(state.includes(needle), 'WOS37 boss cadence projection omitted: ' + needle);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write('checked WOS37 boss cadence source projection: ' + SOURCE_COMMIT.slice(0, 15) + '\n');

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, label + ' catalog drifted');
  for (const row of rows) {
    const cadence = row.boss_cadence;
    invariant(cadence && cadence.pulse_timer === 0 && cadence.stomp_timer === 0 &&
      cadence.big_cast_timer === 0 && cadence.yelled_engage === false &&
      cadence.stoneskin_timer === 0,
    label + ' boss cadence initializer drifted');
  }
}

function readCatalog(relativePath) {
  return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8'));
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
