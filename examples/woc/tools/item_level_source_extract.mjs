// 调用端：item_level_catalog_contract_codegen.mjs；职责：通过 Git 加载器求值固定版本的物品等级定义，供上级目录生成器使用。
// 上层生成器通过固定版本的 wocgit 加载器运行此脚本；标准输出为机器可读的 JSON。

const data = await import('wocgit:///src/sim/data.ts');
const itemLevel = await import('wocgit:///src/sim/item_level.ts');

const items = Object.keys(data.ITEMS).sort().map((id) => {
  const definition = data.ITEMS[id];
  return {
    id,
    source_level: optionalInteger(itemLevel.itemSourceLevel(id)),
    from_raid: itemLevel.itemFromRaid(id),
    item_level: optionalInteger(itemLevel.itemLevel(definition)),
  };
});
const rareSource = items.find((item) =>
  item.source_level !== null && data.ITEMS[item.id].quality === 'rare');
if (!rareSource) throw new Error('item-level source contains no rare sourced item fixture');

process.stdout.write(JSON.stringify({
  items,
  rare_source_fixture: {
    id: rareSource.id,
    source_level: rareSource.source_level,
  },
}));

function optionalInteger(value) {
  if (value === undefined) return null;
  if (!Number.isInteger(value)) throw new Error('item-level source returned a non-integer: ' + value);
  return value;
}
