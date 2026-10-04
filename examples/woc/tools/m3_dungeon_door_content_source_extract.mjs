// 从固定版本 WOC 源码中提取去重后的副本门位置与通行清理范围，供 m3_dungeon_door_content_codegen.mjs 消费。
const data = await import('wocgit:///src/sim/data.ts');
const locomotion = await import('wocgit:///src/sim/mob/locomotion.ts');

const seen = new Set();
// 先去重多个副本共享的门坐标，再加入源码通行清理半径。
const doors = [];
for (const dungeon of Object.values(data.DUNGEONS)) {
  const door = dungeon.doorPos;
  if (!door) continue;
  const key = `${door.x},${door.z}`;
  if (seen.has(key)) continue;
  seen.add(key);
  doors.push({ x: door.x, z: door.z });
}

process.stdout.write(JSON.stringify({
  clear_radius: locomotion.MAX_AGGRO_RADIUS,
  doors,
}));
