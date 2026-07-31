const data = await import('wocgit:///src/sim/data.ts');
const valeCup = await import('wocgit:///src/sim/social/vale_cup.ts');
const pvpHonor = await import('wocgit:///src/sim/content/pvp_honor.ts');

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
