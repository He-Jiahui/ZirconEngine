import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const damage = gitShow('src/sim/combat/damage.ts');
const lifecycle = gitShow('src/sim/mob/lifecycle.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

invariant(entity.includes('tappedById: null,'), 'pinned tap-owner initializer drifted');
invariant(damage.includes('target.tappedById = sourcePid;'), 'pinned tap-owner acquisition drifted');
invariant(lifecycle.includes('mob.tappedById = null;'), 'pinned tap-owner reset drifted');

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

invariant(state.includes('pub var entityTappedByIds: container.Array<uint>;'),
  'WOS32 tap-owner column is missing');
for (const needle of [
  'appendDefaultTapOwnershipColumns(this);',
  'appendDefaultTapOwnershipColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>32', 'schemaVersion != <uint>33',
  'if (schemaVersion >= <uint>32) {',
  'm8FreshPlayerStats.tapOwnershipId',
  'm8EastbrookEncounter.tapOwnershipId',
  'entityState.entityTappedByIds[0] = <uint>900;',
]) invariant(state.includes(needle), `WOS32 tap-ownership projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS32 tap-ownership source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    invariant(row.tap_ownership && typeof row.tap_ownership === 'object' &&
      Number.isSafeInteger(row.tap_ownership.tapped_by_id) &&
      row.tap_ownership.tapped_by_id === 0,
    `${label} tap-owner initializer drifted`);
  }
}

function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
