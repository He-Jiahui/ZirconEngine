import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const lootRoll = gitShow('src/sim/loot/loot_roll.ts');
const locomotion = gitShow('src/sim/mob/locomotion.ts');
const ffa = gitShow('src/sim/loot/loot_ffa.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

invariant(entity.includes('lootFfaTimer: Infinity,'), 'pinned loot-FFA initializer drifted');
invariant(lootRoll.includes('mob.lootFfaTimer = LOOT_FFA_DELAY;'), 'pinned loot-FFA start drifted');
invariant(locomotion.includes('if (mob.lootFfaTimer > 0) mob.lootFfaTimer -= DT;'),
  'pinned loot-FFA countdown drifted');
invariant(ffa.includes('return lootFfaTimer <= 0;'), 'pinned loot-FFA expiry policy drifted');

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

for (const field of [
  'entityLootFfaTimerPresent: container.Array<bool>;',
  'entityLootFfaTimers: container.Array<float>;',
]) invariant(state.includes(`pub var ${field}`), `WOS35 loot-FFA column is missing: ${field}`);
for (const needle of [
  'appendDefaultLootFfaColumns(this);',
  'appendDefaultLootFfaColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>35',
  'if (schemaVersion >= <uint>35) {',
  'm8FreshPlayerStats.lootFfaTimerPresent',
  'm8EastbrookEncounter.lootFfaTimerPresent',
  'entityState.entityLootFfaTimerPresent[0] = true;',
  'entityState.entityLootFfaTimers[0] = 60.0;',
]) invariant(state.includes(needle), `WOS35 loot-FFA projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS35 loot-FFA source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    const value = row.loot_ffa;
    invariant(value && typeof value === 'object' && value.timer_present === false &&
      value.timer_seconds === 0, `${label} loot-FFA initializer drifted`);
  }
}

function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
