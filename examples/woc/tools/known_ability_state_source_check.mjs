// 调用入口：在 examples/woc/tools 目录直接执行 node known_ability_state_source_check.mjs；缺少源码契约时脚本抛错退出。
// 检查已知技能状态中的目录、职业、被动技能、潜行及形态准入条件，并核对代表性技能的源码回归断言。
// 这些断言只核对源码文本与元数据结构；通过并不证明运行时行为等价。

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'combat', 'known_ability_state.zr'),
  'utf8',
);

for (const needle of [
  'pub sourceAbilityAdmission(',
  'sourceCatalog.abilityExists(<int>code)',
  'sourceCatalog.abilityClassId(<int>code) != playerClass',
  'sourceCatalog.isPassive(<int>code)',
  'sourceCatalog.isKnownAt(<int>code, level, granted, committedSpec)',
  'sourceCatalog.requiresStealth(<int>code) && !stealthed',
  'sourceFormAdmission(code, activeForm, isFormToggle)',
  'sourceAbilityAdmission(barkskin, "druid", 20, false, "", "form_bear", false, false)',
  'sourceAbilityAdmission(wrath, "druid", 20, false, "", "form_bear", false, false)',
  'sourceAbilityAdmission(maul, "druid", 20, false, "", "form_cat", false, false)',
  'sourceAbilityAdmission(ambush, "rogue", 20, false, "", "", false, false)',
  'sourceAbilityAdmission(deepWounds, "warrior", 20, false, "", "", false, false)',
]) {
  invariant(projection.includes(needle), `current ability admission omitted: ${needle}`);
}

process.stdout.write('checked current known-ability admission source projection\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
