import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const source = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_ai.ts');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  'pet.petTauntTimer = Math.max(0, pet.petTauntTimer - DT);',
  'if (!pet.inCombat && ctx.tickCount % 40 === 0 && pet.hp < pet.maxHp) {',
  'pet.hp = Math.min(pet.maxHp, pet.hp + Math.max(1, Math.round(pet.maxHp * 0.02)));',
  'let target = pet.aggroTargetId',
]) requireText(source, expected, 'source pet maintenance');

for (const expected of [
  'stepOfflineEmberkinMaintenance(state: WorldState, petIndex: int): void',
  'state.entityPetTauntTimers[petIndex] = tauntTimer > 0.0 ? tauntTimer : 0.0;',
  'state.tick % <uint>40 == <uint>0',
  'math.round(<float>state.entityMaxHp[petIndex] * 0.02)',
  'stepOfflineEmberkinMaintenance(state, petIndex);',
  'pub emberkinMaintenanceStateTest(): int',
]) requireText(world, expected, 'WOS80 reducer');

const maintenance = world.indexOf('stepOfflineEmberkinMaintenance(state, petIndex);');
const targetResolve = world.indexOf('var targetIndex = partyEntityIndexById(', maintenance);
if (maintenance < 0 || targetResolve < maintenance) {
  throw new Error('WOS80 maintenance must remain before target resolution');
}

requireText(contract, 'WOS80 retains the source `updatePet` scalar maintenance', 'WOS80 contract');
requireText(contract, 'tick divisible by 40', 'WOS80 contract');
requireText(contract, 'Water Jet channel ownership', 'WOS80 contract');

process.stdout.write('WOS80 Emberkin pet maintenance runtime static guard passed\n');
