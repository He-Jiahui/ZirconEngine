import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const source = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'combat', 'damage.ts');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  'const pet = ctx.petOf(e.id);',
  'if (pet) handleDeath(ctx, pet, killer);',
  'if (e.ownerId !== null) {',
  "if (MOBS[e.templateId]?.family === 'demon') e.corpseTimer = 3;",
  'return; // owned pets drop no loot/credit; demons unravel, hunters revive or abandon',
]) requireText(source, expected, 'source owned-pet death');

for (const expected of [
  'applyOfflineMobMeleePlayerDeath(state: WorldState, playerIndex: int): void',
  'var petIndex = offlineOwnedEmberkinPetIndex(state, deadPlayerId, false);',
  'beginOfflineEmberkinDemonDeath(state, petIndex);',
  'pub emberkinOwnerDeathStateTest(): int',
]) requireText(world, expected, 'WOS82 reducer');

const playerDeath = world.indexOf('applyOfflineMobMeleePlayerDeath(state: WorldState, playerIndex: int): void');
const petRetire = world.indexOf('beginOfflineEmberkinDemonDeath(state, petIndex);', playerDeath);
if (playerDeath < 0 || petRetire < playerDeath) throw new Error('WOS82 pet retirement must remain in player death');

requireText(contract, 'WOS82 connects the existing player-death reducer', 'WOS82 contract');
requireText(contract, 'short corpse interval', 'WOS82 contract');
requireText(contract, 'dead owner-bound row', 'WOS82 contract');

process.stdout.write('WOS82 Emberkin owner-death runtime static guard passed\n');
