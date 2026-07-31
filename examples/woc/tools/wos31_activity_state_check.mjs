import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const types = gitShow('src/sim/types.ts');
const weaponStow = gitShow('src/sim/weapon_stow.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'sitting: false,', 'weaponStowed: false,', "aiState: 'idle',",
]) invariant(entity.includes(needle), `pinned activity initializer drifted: ${needle}`);
invariant(types.includes("export type AiState = 'idle' | 'chase' | 'attack' | 'flee' | 'evade' | 'dead';"),
  'pinned AI-state domain drifted');
for (const needle of [
  'e.weaponStowed = !e.weaponStowed;', 'e.weaponStowed = false;',
]) invariant(weaponStow.includes(needle), `pinned weapon-stow behavior drifted: ${needle}`);

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);
for (const player of freshPlayers.players) {
  assertActivity(player.activity_state, `fresh activity initializer drifted: ${player.class_id}`);
}

for (const field of [
  'entityAiStates: container.Array<uint>;',
  'entitySitting: container.Array<bool>;',
  'entityWeaponStowed: container.Array<bool>;',
]) invariant(state.includes(`pub var ${field}`), `WOS31 activity-state column is missing: ${field}`);
for (const needle of [
  'appendDefaultActivityStateColumns(this);',
  'appendDefaultActivityStateColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>31', 'schemaVersion != <uint>32',
  'schemaVersion != <uint>33',
  'if (schemaVersion >= <uint>31) {',
  'm8FreshPlayerStats.activityStateAiState',
  'm8EastbrookEncounter.activityStateSitting',
  'entityState.entityAiStates[0] = <uint>4;',
]) invariant(state.includes(needle), `WOS31 activity-state projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS31 activity-state source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) assertActivity(row.activity_state, `${label} activity state is invalid`);
}

function assertActivity(value, message) {
  invariant(value && typeof value === 'object' && value.ai_state === 1 &&
    value.sitting === false && value.weapon_stowed === false, message);
}

function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
