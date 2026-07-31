const data = await import('wocgit:///src/sim/data.ts');

const entries = Object.entries(data.NPCS).map(([id, definition]) => {
  if (!definition.pos || !Number.isFinite(definition.pos.x) || !Number.isFinite(definition.pos.z)) {
    throw new Error(`NPC ${id} has no finite source position`);
  }
  return {
    id,
    dynamic: definition.dynamic === true,
    x: definition.pos.x,
    z: definition.pos.z,
    facing: definition.facing ?? 0,
  };
});

process.stdout.write(JSON.stringify({
  entries,
  static_entries: entries.filter((entry) => !entry.dynamic),
  dynamic_ids: entries.filter((entry) => entry.dynamic).map((entry) => entry.id),
}));
