// 调用入口：在 examples/woc/tools 目录直接执行 node wos82_pet_owner_death_runtime_static_guard.mjs；缺少源码契约时脚本抛错退出。
// 将锁定的有主宠物死亡语义同世界玩家死亡时的宠物退场、自测和合同文本对应核对。
// 这些断言只核对源码文本与元数据结构；通过并不证明运行时行为等价。

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

// 文本位置检查要求宠物退场调用位于玩家死亡归约器声明之后。
const playerDeath = world.indexOf('applyOfflineMobMeleePlayerDeath(state: WorldState, playerIndex: int): void');
const petRetire = world.indexOf('beginOfflineEmberkinDemonDeath(state, petIndex);', playerDeath);
if (playerDeath < 0 || petRetire < playerDeath) throw new Error('WOS82 pet retirement must remain in player death');

requireText(contract, 'WOS82 connects the existing player-death reducer', 'WOS82 contract');
requireText(contract, 'short corpse interval', 'WOS82 contract');
requireText(contract, 'dead owner-bound row', 'WOS82 contract');

process.stdout.write('WOS82 Emberkin owner-death runtime static guard passed\n');
