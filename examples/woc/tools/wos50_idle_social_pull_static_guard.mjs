// 调用入口：在 examples/woc/tools 目录直接执行 node wos50_idle_social_pull_static_guard.mjs；缺少源码契约时脚本抛错退出。
// 检查世界状态中的群体仇恨导入、模板和仇恨查询、拴缚点及目标更新、战斗计时和自测标记。
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
  'var socialAggro = %import("world/fleeing_social_aggro_state");',
  'offlineMobTemplateId(state: WorldState, index: int): string',
  'stateThreatValue(state: WorldState, entityIndex: int, targetId: uint): float',
  'applyOfflineIdleSocialPull(state: WorldState, primaryIndex: int, targetId: uint): void',
  'socialAggro.normalSocialPull(',
  'state.entityLeashAnchorPresent[primaryIndex] = true;',
  'state.entityAggroTargetIds[primaryIndex] = targetId;',
  'state.entityCombatTimers[actorIndex] = 0.0;',
  'applyOfflineIdleSocialPull(state, primaryIndex, targetId);',
  'state.entityThreatValues[0] != 2.0',
]) {
  invariant(state.includes(needle), `WOS idle social pull is missing: ${needle}`);
}

process.stdout.write('checked WOS50 idle social pull source\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
