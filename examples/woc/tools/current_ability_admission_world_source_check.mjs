// 调用入口：在 examples/woc/tools 目录直接执行 node current_ability_admission_world_source_check.mjs；缺少源码契约时脚本抛错退出。
// 检查世界状态中的职业与已提交专精解析、已知技能准入和时间逆转准入回归标记，确保当前技能目录仍接入命令路径。
// 这些断言只核对源码文本与元数据结构；通过并不证明运行时行为等价。

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const source = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);

for (const needle of [
  'var knownAbilities = %import("combat/known_ability_state");',
  'temporalReversalCatalogAdmission(state: WorldState, casterIndex: int, abilityCode: uint): bool',
  'var classIndex = <int><uint>state.entityTemplates[casterIndex] - 1;',
  'var specCode = <uint>state.entityTalentSpecCodes[casterIndex];',
  'var specId = specCode == <uint>0 ? "" : talentSelectionCatalog.specId(specCode);',
  'knownAbilities.sourceAbilityAdmission(',
  'if (!temporalReversalCatalogAdmission(state, casterIndex, abilityCode) ||',
  'var arcaneCode = talentSelectionCatalog.specCode("mage", "arcane");',
  'state.entityTalentSpecCodes[0] = arcaneCode;',
  'state.entityTalentSpecCodes[1] = arcaneCode;',
  'if (temporalReversalCatalogAdmission(state, 0, code)) { return -7; }',
  'if (!temporalReversalCatalogAdmission(state, 0, code)) { return -8; }',
]) {
  invariant(source.includes(needle), `world ability admission omitted: ${needle}`);
}

process.stdout.write('checked current ability admission world integration source projection\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
