import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const spirit = gitShow('src/sim/spirit.ts');
const dungeons = gitShow('src/sim/instances/dungeons.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

invariant(entity.includes('corpseInstanceId: null,'), 'pinned corpse-instance initializer drifted');
for (const needle of [
  'p.corpseInstanceId = ctx.instanceClaimIdAt(p.pos);', 'p.corpseInstanceId = null;',
]) invariant(spirit.includes(needle), `pinned corpse-instance transition drifted: ${needle}`);
invariant(dungeons.includes('candidate.exitId === p.corpseInstanceId'),
  'pinned corpse-instance reentry guard drifted');

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

invariant(state.includes('pub var entityCorpseInstanceIds: container.Array<uint>;'),
  'WOS33 corpse-instance column is missing');
for (const needle of [
  'appendDefaultCorpseInstanceColumns(this);',
  'appendDefaultCorpseInstanceColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>33',
  'if (schemaVersion >= <uint>33) {',
  'm8FreshPlayerStats.corpseInstanceId',
  'm8EastbrookEncounter.corpseInstanceId',
  'entityState.entityCorpseInstanceIds[0] = <uint>77;',
]) invariant(state.includes(needle), `WOS33 corpse-instance projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS33 corpse-instance source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    invariant(row.corpse_instance && typeof row.corpse_instance === 'object' &&
      Number.isSafeInteger(row.corpse_instance.instance_id) && row.corpse_instance.instance_id === 0,
    `${label} corpse-instance initializer drifted`);
  }
}

function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
