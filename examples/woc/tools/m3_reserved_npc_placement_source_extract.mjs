// 从固定版本 WOC 源码中提取Vale Cup 与 Fury NPC 的保留实体 ID 和位置，供 m3_reserved_npc_placement_codegen.mjs 消费。
const data = await import('wocgit:///src/sim/data.ts');
const valeCup = await import('wocgit:///src/sim/social/vale_cup.ts');
const pvpHonor = await import('wocgit:///src/sim/content/pvp_honor.ts');

// 将保留实体 ID 绑定到手工编写的动态 NPC 定义。
const entries = [
  { role: 'groundskeeper_bram', npc_id: 'groundskeeper_bram', entity_id: valeCup.VALE_CUP_BRAM_ID },
  { role: 'fury', npc_id: pvpHonor.FURY_NPC_ID, entity_id: pvpHonor.FURY_ENTITY_ID },
].map((entry) => {
  const definition = data.NPCS[entry.npc_id];
  if (!definition || definition.dynamic !== true || !definition.pos ||
      !Number.isFinite(definition.pos.x) || !Number.isFinite(definition.pos.z) ||
      !Number.isFinite(definition.facing) || !Number.isSafeInteger(entry.entity_id)) {
    throw new Error(`reserved NPC ${entry.role} source shape drifted`);
  }
  return {
    ...entry,
    name: definition.name,
    x: definition.pos.x,
    z: definition.pos.z,
    facing: definition.facing,
  };
});

process.stdout.write(JSON.stringify({ entries }));
