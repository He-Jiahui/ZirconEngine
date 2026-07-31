import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const sim = gitShow('src/sim/sim.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'scale: 1,', 'color: 0xffffff,', "skinCatalog: 'class',", 'skin: 0,',
  'e.color = def.color;', 'e.scale = template.scale;', 'e.color = template.color;',
]) invariant(entity.includes(needle), `pinned presentation identity drifted: ${needle}`);
for (const needle of [
  'player.skinCatalog = meta.skinCatalog;', 'player.skin = meta.skin;',
  'e.skin = idx;', "e.skinCatalog = catalog;",
]) invariant(sim.includes(needle), `pinned skin mirror behavior drifted: ${needle}`);

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);
for (const player of freshPlayers.players) {
  invariant(player.presentation_identity.color === player.color &&
    player.presentation_identity.scale === 1 && player.presentation_identity.skin_catalog === 1 &&
    player.presentation_identity.skin_index === 0,
  `fresh presentation initializer drifted: ${player.class_id}`);
}

for (const field of [
  'entityPresentationScales: container.Array<float>;',
  'entityPresentationColors: container.Array<uint>;',
  'entitySkinCatalogs: container.Array<uint>;',
  'entitySkinIndices: container.Array<uint>;',
]) invariant(state.includes(`pub var ${field}`), `WOS30 presentation column is missing: ${field}`);
for (const needle of [
  'appendDefaultPresentationIdentityColumns(this);',
  'appendDefaultPresentationIdentityColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>30', 'schemaVersion != <uint>31',
  'schemaVersion != <uint>32', 'schemaVersion != <uint>33',
  'if (schemaVersion >= <uint>30) {',
  'm8FreshPlayerStats.presentationIdentityColor',
  'm8EastbrookEncounter.presentationIdentityScale',
  'state.entitySkinIndices[entityIndex] = skinVariant;',
  'entityState.entityPresentationColors[0] = <uint>1193046;',
]) invariant(state.includes(needle), `WOS30 presentation projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS30 presentation-identity source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    const value = row.presentation_identity;
    invariant(value && typeof value === 'object' &&
      Number.isFinite(value.scale) && value.scale > 0 &&
      Number.isSafeInteger(value.color) && value.color >= 0 && value.color <= 0xffffff &&
      (value.skin_catalog === 1 || value.skin_catalog === 2) &&
      Number.isSafeInteger(value.skin_index) && value.skin_index >= 0,
    `${label} presentation identity is invalid`);
  }
}
function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
