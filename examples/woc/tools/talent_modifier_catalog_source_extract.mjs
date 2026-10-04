// 调用端：talent_modifier_catalog_codegen.mjs；职责：从固定版本的 Git 模块图导出已求值的天赋修正记录。
// 上层生成器通过固定版本的 wocgit 加载器运行此脚本；标准输出为机器可读的 JSON。

const [sourceModule] = process.argv.slice(2);
if (!sourceModule) throw new Error('usage: talent_modifier_catalog_source_extract.mjs <source-module>');

const { ROW_TREES, TALENTS } = await import(sourceModule);
const entries = [];
for (const [classId, talents] of Object.entries(TALENTS)) {
  const tree = ROW_TREES[classId];
  if (!tree) throw new Error(`missing row tree for ${classId}`);
  for (const spec of talents.specs) {
    entries.push({ origin: 'spec', class_id: classId, id: spec.id, effect: spec.mastery.effect });
  }
  for (const row of tree) {
    for (const option of row.options) {
      entries.push({
        origin: 'option', class_id: classId, level: row.level, id: option.id, effect: option.effect,
      });
    }
  }
}
process.stdout.write(JSON.stringify({ entries }));
