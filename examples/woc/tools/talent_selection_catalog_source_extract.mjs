// 调用端：talent_selection_catalog_codegen.mjs；职责：从固定版本的 Git 模块图导出已求值的专精和天赋行记录。
// 上层生成器通过固定版本的 wocgit 加载器运行此脚本；标准输出为机器可读的 JSON。

const [sourceModule] = process.argv.slice(2);
if (!sourceModule) {
  throw new Error('usage: talent_selection_catalog_source_extract.mjs <source-module>');
}

const { ROW_TREES, TALENTS } = await import(sourceModule);
const classes = Object.entries(ROW_TREES).map(([id, tree]) => {
  const talents = TALENTS[id];
  if (!talents) throw new Error(`missing talent definitions for ${id}`);
  return {
    id,
    specs: talents.specs.map((spec) => ({ id: spec.id, signature: spec.signature })),
    rows: tree.map((row) => ({
      level: row.level,
      options: row.options.map((option) => ({
        id: option.id,
        grant_ability: option.effect?.grant?.ability ?? null,
      })),
    })),
  };
});

process.stdout.write(JSON.stringify({ classes }));
