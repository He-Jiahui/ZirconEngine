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
  'terrifyTimer: 0,', 'aoeSlowTimer: 0,', 'loudYellTimer: 0,', 'loudYellIndex: 0,',
  'detonateTimer: Infinity,', 'mendTimer: 0,', 'wardTimer: 0,', 'channelTimer: 0,',
  'channelRamp: 0,', 'rallyTimer: 0,', 'warcryTimer: 0,', 'firedSummons: 0,',
  'enraged: false,', 'healedThisPull: false,',
]) invariant(entity.includes(needle), 'pinned boss special initializer drifted: ' + needle);
for (const needle of [
  'mob.firedSummons = 0;', 'mob.enraged = false;', 'mob.healedThisPull = false;',
  'mob.terrifyTimer = MOBS[mob.templateId]?.terrify?.every ?? 0;',
  'mob.mendTimer = MOBS[mob.templateId]?.mendAlly?.every ?? 0;',
]) invariant(lifecycle.includes(needle), 'pinned boss special reset drifted: ' + needle);
for (const needle of [
  'if (mob.detonateTimer !== Infinity)', 'mob.terrifyTimer -= DT;',
  'mob.aoeSlowTimer -= DT;', 'mob.loudYellTimer -= DT;',
]) invariant(locomotion.includes(needle), 'pinned boss special timer source drifted: ' + needle);

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

for (const field of [
  'entityBossTerrifyTimers: container.Array<float>;',
  'entityBossAoeSlowTimers: container.Array<float>;',
  'entityBossLoudYellTimers: container.Array<float>;',
  'entityBossLoudYellIndices: container.Array<int>;',
  'entityBossDetonateTimerPresent: container.Array<bool>;',
  'entityBossDetonateTimers: container.Array<float>;',
  'entityBossMendTimers: container.Array<float>;',
  'entityBossWardTimers: container.Array<float>;',
  'entityBossChannelTimers: container.Array<float>;',
  'entityBossChannelRamps: container.Array<float>;',
  'entityBossRallyTimers: container.Array<float>;',
  'entityBossWarcryTimers: container.Array<float>;',
  'entityBossFiredSummons: container.Array<int>;',
  'entityBossEnraged: container.Array<bool>;',
  'entityBossHealedThisPull: container.Array<bool>;',
]) invariant(state.includes('pub var ' + field), 'WOS38 boss special column is missing: ' + field);
for (const needle of [
  'appendDefaultBossSpecialColumns(this);',
  'appendDefaultBossSpecialColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>38',
  'if (schemaVersion >= <uint>38) {',
  'm8FreshPlayerStats.bossDetonateTimerPresent',
  'm8EastbrookEncounter.bossDetonateTimerPresent',
  'entityState.entityBossDetonateTimerPresent[0] = true;',
  'entityState.entityBossHealedThisPull[0] = true;',
]) invariant(state.includes(needle), 'WOS38 boss special projection omitted: ' + needle);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write('checked WOS38 boss special source projection: ' + SOURCE_COMMIT.slice(0, 15) + '\n');

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, label + ' catalog drifted');
  for (const row of rows) {
    const special = row.boss_special;
    invariant(special && special.detonate_timer?.present === false &&
      special.detonate_timer?.seconds === 0 && special.fired_summons === 0 &&
      special.enraged === false && special.healed_this_pull === false,
    label + ' boss special initializer drifted');
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
