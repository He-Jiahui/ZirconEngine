import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const auras = gitShow('src/sim/combat/auras.ts');
const casting = gitShow('src/sim/combat/casting_lifecycle.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

for (const needle of [
  'fiveSecondRule: 99,', 'comboPoints: 0,', 'comboUntil: -1,',
  'overpowerUntil: -1,', 'potionCooldownUntil: -1,',
  'potionCdRemaining: 0,', 'savedMana: 0,',
]) {
  invariant(entity.includes(needle), `pinned Entity resource-cooldown field drifted: ${needle}`);
}
for (const needle of ['p.fiveSecondRule += DT;', 'p.potionCdRemaining = Math.max', 'p.comboPoints = 0;']) {
  invariant(auras.includes(needle), `pinned resource-cooldown tick drifted: ${needle}`);
}
invariant(casting.includes('p.savedMana = Math.max'), 'pinned saved-mana transition drifted');

assertCatalog(encounter, 'Eastbrook', 24, (row) => row.resource_cooldown);
assertCatalog(freshPlayers, 'fresh player', 9, (row) => row.resource_cooldown);

for (const field of [
  'entityFiveSecondRules: container.Array<float>;',
  'entityComboPoints: container.Array<int>;',
  'entityComboUntil: container.Array<float>;',
  'entityOverpowerUntil: container.Array<float>;',
  'entityPotionCooldownUntil: container.Array<float>;',
  'entityPotionCooldownRemaining: container.Array<float>;',
  'entitySavedMana: container.Array<int>;',
]) {
  invariant(state.includes(`pub var ${field}`), `WOS27 resource-cooldown column is missing: ${field}`);
}
for (const needle of [
  'appendDefaultResourceCooldownColumns(this);', 'appendDefaultResourceCooldownColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>27',
  'if (schemaVersion >= <uint>27) {', 'm8FreshPlayerStats.resourceCooldownDecimal',
  'm8EastbrookEncounter.resourceCooldownInteger', 'entityState.entityComboPoints[0] = 5;',
  'entityState.entityPotionCooldownUntil[0] = 90.0;',
]) {
  invariant(state.includes(needle), `WOS27 resource-cooldown projection omitted: ${needle}`);
}
invariant((state.match(/entityComboPoints/g) ?? []).length >= 9,
  'WOS27 combo points lack persistence coverage');
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the WOS38 snapshot version');

process.stdout.write(`checked WOS27 resource-cooldown source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function assertCatalog(catalog, label, count, project) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, `${label} catalog drifted`);
  for (const row of rows) {
    const resource = project(row);
    invariant(resource && typeof resource === 'object', `${label} resource cooldown is missing`);
    invariant(resource.five_second_rule === 99 && resource.combo_points === 0 &&
      resource.combo_until === -1 && resource.overpower_until === -1 &&
      resource.potion_cooldown_until === -1 && resource.potion_cd_remaining === 0 &&
      resource.saved_mana === 0, `${label} resource-cooldown initializer drifted`);
  }
}

function readCatalog(relativePath) {
  return JSON.parse(readFileSync(resolve(wocRoot, relativePath), 'utf8'));
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
