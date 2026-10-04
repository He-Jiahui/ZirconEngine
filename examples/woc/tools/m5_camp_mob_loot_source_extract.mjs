// 从固定版本 WOC 源码中提取营地怪物掉落项与组件标签，供 m5_camp_mob_loot_codegen.mjs 消费。
const data = await import('wocgit:///src/sim/data.ts');

// 按营地首次出现的顺序去重怪物模板，保持生成索引稳定。
const ids = [];
const seen = new Set();
for (const camp of data.BUILTIN_WORLD.camps) {
  if (!seen.has(camp.mobId)) {
    seen.add(camp.mobId);
    ids.push(camp.mobId);
  }
}

const mobs = ids.map((id) => {
  const template = data.MOBS[id];
  if (!template) throw new Error(`missing camp mob template ${id}`);
  return {
    id: template.id,
    loot_entries: template.loot.map((entry) => ({
      has_item_id: typeof entry.itemId === 'string',
      item_id: entry.itemId ?? '',
      has_copper: typeof entry.copper === 'number',
      copper: entry.copper ?? 0,
      chance: entry.chance,
      has_quest_id: typeof entry.questId === 'string',
      quest_id: entry.questId ?? '',
      has_roll_group: typeof entry.rollGroup === 'string',
      roll_group: entry.rollGroup ?? '',
    })),
    component_tags: [...(template.componentTags ?? [])],
  };
});

process.stdout.write(JSON.stringify({ mobs }));
