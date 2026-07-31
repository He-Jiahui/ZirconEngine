const data = await import('wocgit:///src/sim/data.ts');

const entries = Object.entries(data.NPCS).map(([id, definition]) => {
  if (!definition.pos || !Number.isFinite(definition.pos.x) || !Number.isFinite(definition.pos.z)) {
    throw new Error(`NPC ${id} has no finite source position`);
  }
  for (const field of ['name', 'title', 'greeting']) {
    if (typeof definition[field] !== 'string') {
      throw new Error(`NPC ${id} has no string ${field}`);
    }
  }
  if (!Number.isFinite(definition.facing) || !Number.isInteger(definition.color) ||
      !Array.isArray(definition.questIds) ||
      (definition.vendorItems !== undefined && !Array.isArray(definition.vendorItems))) {
    throw new Error(`NPC ${id} has an invalid initialization field`);
  }
  return {
    id,
    dynamic: definition.dynamic === true,
    name: definition.name,
    title: definition.title,
    greeting: definition.greeting,
    x: definition.pos.x,
    z: definition.pos.z,
    facing: definition.facing,
    color: definition.color,
    quest_ids: definition.questIds,
    vendor_items: definition.vendorItems ?? [],
    dev_vendor: definition.devVendor === true,
    market: definition.market === true,
    banker: definition.banker === true,
    heroic_vendor: definition.heroicVendor === true,
    card_master: definition.cardMaster === true,
  };
});

process.stdout.write(JSON.stringify({ entries }));
