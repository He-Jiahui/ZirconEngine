const data = await import('wocgit:///src/sim/data.ts');
const locomotion = await import('wocgit:///src/sim/mob/locomotion.ts');

const seen = new Set();
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
