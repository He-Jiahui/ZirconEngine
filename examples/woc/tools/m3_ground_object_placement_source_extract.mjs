// 从固定版本 WOC 源码中提取地面物品定义及各自的世界坐标，供 m3_ground_object_placement_codegen.mjs 消费。
const data = await import('wocgit:///src/sim/data.ts');

// 将每个地面物品定义按手工位置展开为逐坐标记录。
const entries = [];
for (const definition of data.GROUND_OBJECTS) {
  if (typeof definition.itemId !== 'string' || typeof definition.name !== 'string' ||
      !Array.isArray(definition.positions)) {
    throw new Error('ground-object definition has an invalid shape');
  }
  for (const position of definition.positions) {
    if (!Number.isFinite(position.x) || !Number.isFinite(position.z)) {
      throw new Error(`ground object ${definition.itemId} has no finite position`);
    }
    entries.push({
      item_id: definition.itemId,
      name: definition.name,
      x: position.x,
      z: position.z,
    });
  }
}

process.stdout.write(JSON.stringify({ definition_count: data.GROUND_OBJECTS.length, entries }));
