// 从固定版本 WOC 源码中提取墓地坐标与共用灵魂医者标识，供 m3_spirit_healer_placement_codegen.mjs 消费。
const graveyards = await import('wocgit:///src/sim/content/graveyards.ts');

if (!Array.isArray(graveyards.OVERWORLD_GRAVEYARDS) ||
    typeof graveyards.SPIRIT_HEALER_NPC_ID !== 'string' || !graveyards.SPIRIT_HEALER) {
  throw new Error('spirit healer source shape drifted');
}

// 从源码内容读取墓地坐标与共用医者标识。
const entries = graveyards.OVERWORLD_GRAVEYARDS.map((entry) => {
  if (typeof entry.id !== 'string' || typeof entry.name !== 'string' ||
      !Number.isFinite(entry.x) || !Number.isFinite(entry.z)) {
    throw new Error('graveyard entry has an invalid shape');
  }
  return { id: entry.id, name: entry.name, x: entry.x, z: entry.z };
});
const healer = graveyards.SPIRIT_HEALER;
if (healer.id !== graveyards.SPIRIT_HEALER_NPC_ID || typeof healer.name !== 'string' ||
    !Number.isFinite(healer.facing) || !Number.isInteger(healer.color) || healer.dynamic !== true) {
  throw new Error('spirit healer NPC source shape drifted');
}

process.stdout.write(JSON.stringify({
  graveyards: entries,
  healer: { id: healer.id, name: healer.name, facing: healer.facing, color: healer.color },
}));
