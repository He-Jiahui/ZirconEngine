const data = await import('wocgit:///src/sim/data.ts');

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
