// 调用入口：在 examples/woc/tools 目录直接执行 node wos50_idle_proximity_aggro_static_guard.mjs；缺少源码契约时脚本抛错退出。
// 检查世界状态中 Eastbrook 怪物的空闲目标选择、模板感知半径和稀有标记、进入仇恨的调用、帧更新及自测。
// 这些断言只核对源码文本与元数据结构；通过并不证明运行时行为等价。

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const state = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);

for (const needle of [
  'var idleAggro = %import("world/mob_idle_aggro_state");',
  'enterIdleMobAggro(state: WorldState, primaryIndex: int, targetId: uint): bool',
  'stepOfflineEastbrookMobIdleAggro(state: WorldState): void',
  'idleAggro.selectIdleAggroTarget(',
  'campMobCore.metric(templateIndex, "aggro_radius", true)',
  'campMobCore.flag(templateIndex, "elite", true)',
  'campMobCore.flag(templateIndex, "rare", true)',
  'campMobCore.flag(templateIndex, "boss", true)',
  'enterIdleMobAggro(state, index, selection.targetId)',
  'stepOfflineEastbrookMobIdleAggro(state);',
  'pub offlineIdleProximityAggroStateTest(): int',
  'offlineIdleProximityAggroStateTest() != 1',
]) {
  invariant(state.includes(needle), `WOS idle proximity aggro is missing: ${needle}`);
}

process.stdout.write('checked WOS50 idle proximity aggro source\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
