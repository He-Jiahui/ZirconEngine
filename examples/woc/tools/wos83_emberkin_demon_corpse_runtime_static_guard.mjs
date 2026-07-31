import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const source = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'combat', 'damage.ts');
const commands = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_commands.ts');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  "if (MOBS[e.templateId]?.family === 'demon') e.corpseTimer = 3;",
  'return; // owned pets drop no loot/credit; demons unravel, hunters revive or abandon',
]) requireText(source, expected, 'source demon death');

for (const expected of [
  'if (!pet.dead) {',
  'pet.corpseTimer = 0;',
  'pet.respawnTimer = 0;',
  'pet.hp = Math.max(1, Math.round(pet.maxHp * 0.35));',
]) requireText(commands, expected, 'source pet revive');

for (const expected of [
  'var OFFLINE_EMBERKIN_DEMON_CORPSE_MICROS: uint = <uint>3000000;',
  'var deadIndex = -1;',
  'beginOfflineEmberkinDemonDeath(state: WorldState, petIndex: int): void',
  'ageOfflineEmberkinDemonCorpses(state: WorldState): void',
  'ageOfflineEmberkinDemonCorpses(state);',
  '<uint>state.entityCorpseMicros[petIndex] == <uint>0',
  'pub emberkinDemonCorpseStateTest(): int',
]) requireText(world, expected, 'WOS83 reducer');

const begin = world.indexOf('beginOfflineEmberkinDemonDeath(state, petIndex);');
const age = world.indexOf('ageOfflineEmberkinDemonCorpses(state);');
if (begin < 0 || age < 0) throw new Error('WOS83 demon corpse lifecycle is not wired');

requireText(contract, 'WOS83 makes that demon-death projection time-bounded.', 'WOS83 contract');
requireText(contract, 'three-second corpse period', 'WOS83 contract');
requireText(contract, 'prefers a living', 'WOS83 contract');

process.stdout.write('WOS83 Emberkin demon corpse runtime static guard passed\n');
