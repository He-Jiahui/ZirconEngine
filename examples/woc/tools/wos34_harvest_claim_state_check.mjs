import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const interaction = gitShow('src/sim/interaction.ts');
const lifecycle = gitShow('src/sim/mob/lifecycle.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

invariant(entity.includes('harvestClaimedBy: null,'), 'pinned harvest-claim initializer drifted');
invariant(interaction.includes('mob.harvestClaimedBy = claim.claimedBy;'),
  'pinned harvest-claim acquisition drifted');
invariant(lifecycle.includes('mob.harvestClaimedBy = null;'),
  'pinned harvest-claim reset drifted');

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

invariant(state.includes('pub var entityHarvestClaimedByIds: container.Array<uint>;'),
  'WOS34 harvest-claim column is missing');
for (const needle of [
  'appendDefaultHarvestClaimColumns(this);',
  'appendDefaultHarvestClaimColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>34',
  'if (schemaVersion >= <uint>34) {',
  'm8FreshPlayerStats.harvestClaimId',
  'm8EastbrookEncounter.harvestClaimId',
  'entityState.entityHarvestClaimedByIds[0] = <uint>900;',
]) invariant(state.includes(needle), `WOS34 harvest-claim projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS34 harvest-claim source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    invariant(row.harvest_claim && typeof row.harvest_claim === 'object' &&
      Number.isSafeInteger(row.harvest_claim.claimed_by_id) &&
      row.harvest_claim.claimed_by_id === 0,
    `${label} harvest-claim initializer drifted`);
  }
}

function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
