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
const petAi = gitShow('src/sim/pet/pet_ai.ts');
const petCommands = gitShow('src/sim/pet/pet_commands.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const encounter = readCatalog('reference/current-head/m8_eastbrook_encounter.json');
const freshPlayers = readCatalog('contracts/m8_fresh_player_stats.json');

invariant(entity.includes("petMode: 'defensive',") && entity.includes('petTauntTimer: 0,') &&
  entity.includes('petPath: [],') && entity.includes('petPathCooldown: 0,'),
'pinned pet runtime initializer drifted');
invariant(types.includes("export type PetMode = 'passive' | 'defensive' | 'aggressive';"),
'pinned pet mode union drifted');
invariant(petAi.includes("if (pet.petMode === 'passive') return null;") &&
  petAi.includes("pet.petMode === 'aggressive'"),
'pinned pet-mode behavior routing drifted');
invariant(petCommands.includes("pet.petAutoTaunt = state.autoTaunt ?? false;") &&
  petCommands.includes("pet.petAutoWaterJet = state.autoWaterJet ?? false;") &&
  petCommands.includes('pet.petManualTauntPending = false;'),
'pinned pet optional-flag restore policy drifted');

assertCatalog(encounter, 'Eastbrook', 24);
assertCatalog(freshPlayers, 'fresh player', 9);

for (const field of [
  'entityPetModes: container.Array<uint>;',
  'entityPetTauntTimers: container.Array<float>;',
  'entityPetAutoTauntPresent: container.Array<bool>;',
  'entityPetAutoTaunt: container.Array<bool>;',
  'entityPetAutoWaterJetPresent: container.Array<bool>;',
  'entityPetAutoWaterJet: container.Array<bool>;',
  'entityPetManualTauntPendingPresent: container.Array<bool>;',
  'entityPetManualTauntPending: container.Array<bool>;',
  'entityPetPathCooldowns: container.Array<float>;',
]) invariant(state.includes('pub var ' + field), 'WOS36 pet runtime column is missing: ' + field);
for (const needle of [
  'appendDefaultPetRuntimeColumns(this);',
  'appendDefaultPetRuntimeColumns(state);',
  'writer.u16(<uint>38, 1, 1);', 'schemaVersion != <uint>36',
  'if (schemaVersion >= <uint>36) {',
  'm8FreshPlayerStats.petMode',
  'm8EastbrookEncounter.petMode',
  'entityState.entityPetModes[0] = <uint>3;',
  'entityState.entityPetPathCooldowns[0] = 0.75;',
]) invariant(state.includes(needle), 'WOS36 pet runtime projection omitted: ' + needle);
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'), 'package stateSchema must expose WOS38');

process.stdout.write('checked WOS36 pet runtime source projection: ' + SOURCE_COMMIT.slice(0, 15) + '\n');

function assertCatalog(catalog, label, count) {
  const rows = catalog.spawns ?? catalog.players;
  invariant(catalog.schema_version === 17 && rows.length === count, label + ' catalog drifted');
  for (const row of rows) {
    const pet = row.pet_runtime;
    invariant(pet && pet.mode === 2 && pet.taunt_timer === 0 && pet.path_cooldown === 0,
      label + ' pet runtime initializer drifted');
    for (const field of ['auto_taunt', 'auto_water_jet', 'manual_taunt_pending']) {
      invariant(pet[field]?.present === false && pet[field]?.value === false,
        label + ' pet optional initializer drifted: ' + field);
    }
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
