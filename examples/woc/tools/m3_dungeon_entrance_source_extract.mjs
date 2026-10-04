// 从固定版本 WOC 源码中提取副本入口标识、门坐标与实例槽位数，供 m3_dungeon_entrance_codegen.mjs 消费。
const data = await import('wocgit:///src/sim/data.ts');

if (!Array.isArray(data.DUNGEON_LIST) || !Number.isInteger(data.INSTANCE_SLOT_COUNT)) {
  throw new Error('dungeon list source shape drifted');
}

const dungeons = data.DUNGEON_LIST.map((dungeon) => {
  if (typeof dungeon.id !== 'string' || typeof dungeon.name !== 'string' ||
      !Number.isInteger(dungeon.index) || !dungeon.doorPos ||
      !Number.isFinite(dungeon.doorPos.x) || !Number.isFinite(dungeon.doorPos.z)) {
    throw new Error('dungeon definition has an invalid entry shape');
  }
  return {
    id: dungeon.id,
    name: dungeon.id === 'nythraxis_crypt' ? 'Abandoned Crypt' : dungeon.name,
    index: dungeon.index,
    overworld_door: dungeon.overworldDoor !== false,
    x: dungeon.doorPos.x,
    z: dungeon.doorPos.z,
  };
});

process.stdout.write(JSON.stringify({ instance_slot_count: data.INSTANCE_SLOT_COUNT, dungeons }));
