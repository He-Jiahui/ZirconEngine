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
const freshPlayers = JSON.parse(readFileSync(
  resolve(wocRoot, 'contracts', 'm8_fresh_player_stats.json'), 'utf8',
));

for (const needle of [
  'inCombat: false,',
  'combatTimer: 99,',
  'aggroTargetId: null,',
]) {
  invariant(entity.includes(needle), `pinned Entity combat-state field drifted: ${needle}`);
}
for (const needle of [
  'mob.combatTimer += DT;',
  'if (mob.inCombat ||',
  'mob.inCombat = false;',
  'mob.aggroTargetId = null;',
]) {
  invariant(locomotion.includes(needle), `pinned mob combat-state behavior drifted: ${needle}`);
}

invariant(encounter.schema_version === 17 && encounter.spawns.length === 24,
  'Eastbrook encounter combat-state catalog drifted');
for (const spawn of encounter.spawns) {
  assertCombatState(spawn.combat_state, `Eastbrook ${spawn.source_entity_id}`);
}
invariant(freshPlayers.schema_version === 17 && freshPlayers.players.length === 9,
  'fresh-player combat-state catalog drifted');
for (const player of freshPlayers.players) {
  assertCombatState(player.combat_state, `fresh ${player.class_id}`);
}

for (const needle of [
  'pub var entityInCombat: container.Array<bool>;',
  'pub var entityCombatTimers: container.Array<float>;',
  'pub var entityAggroTargetIds: container.Array<uint>;',
  'this.entityInCombat = new container.Array<bool>();',
  'appendDefaultMobCombatStateColumns(this);',
  'appendDefaultMobCombatStateColumns(state);',
  'writer.u16(<uint>38, 1, 1);',
  'schemaVersion != <uint>24',
  'if (schemaVersion >= <uint>24) {',
  'state.entityInCombat[entityIndex] = m8FreshPlayerStats.combatStateFlag',
  'm8EastbrookEncounter.combatStateDecimal(spawnIndex, "combatTimer")',
  'state.entityAggroTargetIds[entityIndex] = m8EastbrookEncounter.combatStateTargetId',
  'entityState.entityInCombat[0] = true;',
  'entityState.entityCombatTimers[0] = 4.25;',
]) {
  invariant(state.includes(needle), `WOS24 mob combat-state projection omitted: ${needle}`);
}
invariant((state.match(/entityInCombat/g) ?? []).length >= 9,
  'WOS24 in-combat state lacks persistence coverage');
invariant((state.match(/entityCombatTimers/g) ?? []).length >= 9,
  'WOS24 combat timer lacks persistence coverage');
invariant((state.match(/entityAggroTargetIds/g) ?? []).length >= 9,
  'WOS24 aggro target lacks persistence coverage');
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the WOS38 snapshot version');

process.stdout.write(`checked WOS24 mob combat-state source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCombatState(combatState, subject) {
  invariant(combatState && typeof combatState === 'object', `${subject} combat state is missing`);
  invariant(combatState.in_combat === false, `${subject} in-combat initializer drifted`);
  invariant(combatState.combat_timer === 99, `${subject} combat timer initializer drifted`);
  invariant(combatState.aggro_target_id === 0, `${subject} aggro target initializer drifted`);
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
