// 调用入口：在 examples/woc/tools 目录直接执行 node wos79_pet_follow_runtime_static_guard.mjs；缺少源码契约时脚本抛错退出。
// 将锁定的跟随速度和距离与 Zr 纯规则及世界空旷地形分支对应核对，排除未实现的路径状态调用并检查文档边界。
// 这些断言只核对源码文本与元数据结构；通过并不证明运行时行为等价。

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const source = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_ai.ts');
const rules = read('scripts', 'woc_game', 'src', 'instances', 'pet_follow_rules.zr');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  'const PET_FOLLOW_DISTANCE = 3.5;',
  'PET_TELEPORT_DISTANCE,',
  'const PET_FORCE_RECOVERY_DISTANCE = 96;',
  'function petFollow(ctx: SimContext, pet: Entity, owner: Entity): void {',
  'const speed = Math.max(pet.moveSpeed, RUN_SPEED * 1.1) * ctx.moveSpeedMult(pet);',
  'ctx.moveToward(pet, aim, speed);',
]) requireText(source, expected, 'source pet follow');

for (const expected of [
  'pub followDistance(): float',
  'return 3.5;',
  'pub shouldClearPath(distanceToOwner: float): bool',
  'pub followSpeed(petMoveSpeed: float, moveSpeedMultiplier: float): float',
  'pub contractTest(): int',
]) requireText(rules, expected, 'source-locked follow rules');

for (const expected of [
  'var petFollow = %import("instances/pet_follow_rules");',
  'stepOfflineEmberkinStraightFollow(',
  'petFollow.shouldClearPath(distance)',
  'petFollow.followSpeed(',
  'state.entityPreviousX[petIndex]',
  'terrainGround.builtinGroundHeight(',
  'stepOfflineEmberkinStraightFollow(state, petIndex, ownerIndex);',
  'pub emberkinFollowStateTest(): int',
]) requireText(world, expected, 'WOS79 reducer');

// 反向扫描排除在空旷地形跟随投影中调用尚无状态支撑的寻路操作。
for (const forbidden of [
  'petFollow.shouldWarpAfterFreshPath(',
  'petFollow.shouldRecomputePath(',
  'petFollow.shouldDropReachedWaypoint(',
]) {
  if (world.includes(forbidden)) throw new Error('WOS79 must not claim unavailable path state: ' + forbidden);
}

requireText(contract, 'WOS79 completes the no-target Emberkin heel branch only for open ground', 'WOS79 contract');
requireText(contract, 'current WOS rows', 'WOS79 contract');
requireText(contract, 'intentionally absent rather than simulated with', 'WOS79 contract');

process.stdout.write('WOS79 Emberkin pet follow runtime static guard passed\n');
