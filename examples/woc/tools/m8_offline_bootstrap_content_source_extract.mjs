const classes = await import('wocgit:///src/sim/content/classes.ts');
const data = await import('wocgit:///src/sim/data.ts');

const classIds = Object.keys(classes.CLASSES);
const starterItemIds = new Set();
const projectedClasses = classIds.map((id) => {
  const definition = classes.CLASSES[id];
  for (const itemId of [
    definition.startWeapon,
    definition.startChest,
    ...definition.startItems.map((item) => item.itemId),
  ]) {
    if (typeof itemId === 'string' && itemId.length > 0) starterItemIds.add(itemId);
  }
  return {
    id: definition.id,
    name: definition.name,
    base_stats: projectStats(definition.baseStats),
    stats_per_level: projectStats(definition.statsPerLevel),
    base_hp: definition.baseHp,
    hp_per_level: definition.hpPerLevel,
    base_mana: definition.baseMana,
    mana_per_level: definition.manaPerLevel,
    resource_type: definition.resourceType,
    start_weapon: definition.startWeapon,
    start_chest: definition.startChest,
    start_items: definition.startItems.map((item) => ({ item_id: item.itemId, count: item.count })),
    ranged: definition.ranged ? {
      min: definition.ranged.min,
      max: definition.ranged.max,
      speed: definition.ranged.speed,
      max_range: definition.ranged.maxRange,
      min_range: definition.ranged.minRange,
      wand: definition.ranged.wand === true,
      school: definition.ranged.school ?? null,
    } : null,
    abilities: [...definition.abilities],
    color: definition.color,
  };
});

const starterItems = [...starterItemIds].sort().map((id) => {
  const definition = data.ITEMS[id];
  if (!definition) throw new Error(`missing starter item definition: ${id}`);
  return {
    id,
    kind: definition.kind,
    slot: definition.slot ?? null,
    armor_type: definition.armorType ?? null,
    quality: definition.quality ?? null,
    stats: projectStats(definition.stats),
    weapon: definition.weapon ? {
      min: definition.weapon.min,
      max: definition.weapon.max,
      speed: definition.weapon.speed,
    } : null,
    food_hp: definition.foodHp ?? 0,
    drink_mana: definition.drinkMana ?? 0,
  };
});

process.stdout.write(JSON.stringify({
  player_start: { x: data.PLAYER_START.x, z: data.PLAYER_START.z },
  classes: projectedClasses,
  starter_items: starterItems,
}));

function projectStats(stats) {
  return {
    str: stats?.str ?? 0,
    agi: stats?.agi ?? 0,
    sta: stats?.sta ?? 0,
    int: stats?.int ?? 0,
    spi: stats?.spi ?? 0,
    armor: stats?.armor ?? 0,
  };
}
