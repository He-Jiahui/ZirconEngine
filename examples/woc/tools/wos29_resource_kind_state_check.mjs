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
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

invariant(types.includes("export type ResourceType = 'rage' | 'mana' | 'energy';"),
  'pinned resource-type union drifted');
for (const needle of ['resourceType: null,', 'e.resourceType = def.resourceType;',
  "e.resourceType = 'mana';", 'e.resourceType = formResource;']) {
  invariant(entity.includes(needle), `pinned resource-kind behavior drifted: ${needle}`);
}

assertCatalog(encounter, 'Eastbrook', 24, 0);
assertCatalog(freshPlayers, 'fresh player', 9, undefined);
const expectedFreshKinds = new Map([
  ['warrior', 2], ['mage', 1], ['rogue', 3], ['paladin', 1], ['hunter', 1],
  ['priest', 1], ['shaman', 1], ['warlock', 1], ['druid', 1],
]);
for (const player of freshPlayers.players) {
  invariant(player.resource_kind === expectedFreshKinds.get(player.class_id),
    `fresh resource kind drifted: ${player.class_id}`);
}

for (const needle of [
  'pub var entityResourceKinds: container.Array<uint>;',
  'appendDefaultResourceKindColumns(this);', 'appendDefaultResourceKindColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>29',
  'if (schemaVersion >= <uint>29) {', 'm8FreshPlayerStats.resourceKind',
  'm8EastbrookEncounter.resourceKind', 'entityState.entityResourceKinds[0] = <uint>3;',
]) invariant(state.includes(needle), `WOS29 resource-kind projection omitted: ${needle}`);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write(`checked WOS29 resource-kind source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count, expectedKind) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    invariant(Number.isSafeInteger(row.resource_kind) && row.resource_kind >= 0 && row.resource_kind <= 3,
      `${label} resource kind is invalid`);
    if (expectedKind !== undefined) invariant(row.resource_kind === expectedKind,
      `${label} resource kind initializer drifted`);
  }
}
function readCatalog(relativePath) { return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8')); }
function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}
function invariant(condition, message) { if (!condition) throw new Error(message); }
