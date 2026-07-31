const { CLASSES } = await import('wocgit:///src/sim/content/classes.ts');
const { BASE_ITEMS } = await import('wocgit:///src/sim/content/items.ts');
const { createPlayer, recalcPlayerStats } = await import('wocgit:///src/sim/entity.ts');
const { MAX_LEVEL } = await import('wocgit:///src/sim/types.ts');

const classIds = Object.keys(CLASSES);
const classes = classIds.map((classId) => classBaseline(classId));

process.stdout.write(JSON.stringify({ max_level: MAX_LEVEL, classes }));

function classBaseline(classId) {
  const definition = CLASSES[classId];
  const equipment = { mainhand: definition.startWeapon, chest: definition.startChest };
  const levels = [];

  for (let level = 1; level <= MAX_LEVEL; level += 1) {
    const player = createPlayer(1, classId, { x: 2, y: 1.5, z: -2 }, 'Baseline');
    player.level = level;
    recalcPlayerStats(player, classId, equipment, undefined, {});
    player.hp = player.maxHp;
    player.resource = definition.resourceType === 'mana'
      ? player.maxResource
      : definition.resourceType === 'energy'
        ? 100
        : 0;
    levels.push({
      ...projectLevel(player),
      pre_form: preFormInput(definition, equipment, level),
    });
  }

  return {
    class_id: classId,
    resource_type: definition.resourceType,
    equipment,
    equipment_contributions: {
      mainhand: equipmentContribution(equipment.mainhand),
      chest: equipmentContribution(equipment.chest),
    },
    start_items: definition.startItems.map((item) => ({ item_id: item.itemId, count: item.count })),
    levels,
  };
}

// Keep the initial gear's source contribution separate from the already
// equipped baseline. A later WOS equipment identity can replace the mainhand
// by subtracting this exact contribution before applying its catalog item.
function equipmentContribution(itemId) {
  const item = BASE_ITEMS[itemId];
  if (!item) throw new Error(`missing M5 starting item ${itemId}`);
  return {
    item_id: itemId,
    stats: {
      str: item.stats?.str ?? 0,
      agi: item.stats?.agi ?? 0,
      sta: item.stats?.sta ?? 0,
      int: item.stats?.int ?? 0,
      armor: item.stats?.armor ?? 0,
    },
    spell_power: item.spellPower ?? 0,
    crit_rating: item.critRating ?? 0,
    haste_rating: item.hasteRating ?? 0,
    hit_rating: item.hitRating ?? 0,
    weapon: {
      min: item.weapon?.min ?? 0,
      max: item.weapon?.max ?? 0,
      speed: item.weapon?.speed ?? 0,
    },
  };
}

// This is the exact pre-form point in recalcPlayerStats for the M5 starting
// equipment subset. It intentionally excludes set, talent, percent and aura
// contributions, which are separate future contribution inputs.
function preFormInput(definition, equipment, level) {
  const stats = {
    str: definition.baseStats.str + definition.statsPerLevel.str * (level - 1),
    agi: definition.baseStats.agi + definition.statsPerLevel.agi * (level - 1),
    sta: definition.baseStats.sta + definition.statsPerLevel.sta * (level - 1),
    int: definition.baseStats.int + definition.statsPerLevel.int * (level - 1),
    armor: definition.baseStats.armor + definition.statsPerLevel.armor * (level - 1),
  };
  let bonusSpellPower = 0;
  for (const itemId of [equipment.mainhand, equipment.chest]) {
    const item = BASE_ITEMS[itemId];
    if (!item) throw new Error(`missing M5 starting item ${itemId}`);
    bonusSpellPower += item.spellPower ?? 0;
    if (!item.stats) continue;
    stats.str += item.stats.str ?? 0;
    stats.agi += item.stats.agi ?? 0;
    stats.sta += item.stats.sta ?? 0;
    stats.int += item.stats.int ?? 0;
    stats.armor += item.stats.armor ?? 0;
  }
  return {
    strength: stats.str,
    agility: stats.agi,
    stamina: stats.sta,
    intellect: stats.int,
    armor_before_agility: stats.armor,
    bonus_attack_power: 0,
    bonus_spell_power: bonusSpellPower,
    base_hp: definition.baseHp,
    hp_per_level: definition.hpPerLevel,
  };
}

function projectLevel(player) {
  return {
    level: player.level,
    stats: {
      str: player.stats.str,
      agi: player.stats.agi,
      sta: player.stats.sta,
      int: player.stats.int,
      spi: player.stats.spi,
      armor: player.stats.armor,
      pvp_offense: player.stats.pvpOffense,
      pvp_defense: player.stats.pvpDefense,
    },
    weapon: {
      min: player.weapon.min,
      max: player.weapon.max,
      speed: player.weapon.speed,
    },
    max_hp: player.maxHp,
    hp: player.hp,
    max_resource: player.maxResource,
    resource: player.resource,
    attack_power: player.attackPower,
    ranged_power: player.rangedPower,
    spell_power: player.spellPower,
    crit_chance: player.critChance,
    dodge_chance: player.dodgeChance,
    move_speed: player.moveSpeed,
  };
}
