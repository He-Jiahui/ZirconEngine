const [scopeJson] = process.argv.slice(2);
if (!scopeJson) throw new Error('M5 content scope JSON is required');

const scope = JSON.parse(scopeJson);
const data = await import('wocgit:///src/sim/data.ts');
const classes = await import('wocgit:///src/sim/content/classes.ts');
const talents = await import('wocgit:///src/sim/content/talents.ts');
const types = await import('wocgit:///src/sim/types.ts');
const bank = await import('wocgit:///src/sim/bank.ts');

const quests = selectDefinitions(data.QUESTS, scope.quest_ids, 'quest');
const mobs = selectDefinitions(data.MOBS, scope.mob_ids, 'mob');
const questNpcIds = Object.values(quests).map((quest) => quest.giverNpcId);
const questItemIds = Object.values(quests)
  .flatMap((quest) => quest.objectives ?? [])
  .map((objective) => objective.itemId)
  .filter((id) => typeof id === 'string');
const mobItemIds = Object.values(mobs)
  .flatMap((mob) => mob.loot ?? [])
  .map((entry) => entry.itemId)
  .filter((id) => typeof id === 'string');
const bankerIds = Object.entries(data.NPCS)
  .filter(([, definition]) => definition.banker === true)
  .map(([id]) => id)
  .sort();
const npcIds = [...new Set([...scope.npc_ids, ...questNpcIds, ...bankerIds])].sort();
const npcs = selectDefinitions(data.NPCS, npcIds, 'NPC');
const vendorItemIds = [...new Set(
  Object.values(npcs)
    .flatMap((definition) => definition.vendorItems ?? [])
    .filter((itemId) => typeof itemId === 'string'),
)].sort();
const specs = selectDefinitions(
  Object.fromEntries(talents.TALENTS.warrior.specs.map((spec) => [spec.id, spec])),
  scope.spec_ids,
  'specialization',
);
const specAbilityIds = Object.values(specs).map((spec) => spec.signature);
const classStartingEquipmentItemIds = [...new Set(
  Object.values(classes.CLASSES).flatMap((definition) => [
    definition.startWeapon,
    definition.startOffhand,
    definition.startChest,
  ]).filter((itemId) => typeof itemId === 'string'),
)].sort();
const talentOptions = selectTalentOptions(
  talents.ROW_TREES.warrior,
  scope.talent_option_ids,
  'warrior talent row option',
);

const result = {
  items: selectDefinitions(
    data.ITEMS,
    [...new Set([
      ...scope.item_ids,
      ...questItemIds,
      ...mobItemIds,
      ...classStartingEquipmentItemIds,
      ...vendorItemIds,
    ])].sort(),
    'item',
  ),
  quests,
  mobs,
  npcs,
  talent_options: talentOptions,
  specs,
  abilities: selectDefinitions(
    classes.ABILITIES,
    [...new Set([...scope.ability_ids, ...specAbilityIds])].sort(),
    'ability',
  ),
  banker_ids: bankerIds,
  class_starting_equipment_item_ids: classStartingEquipmentItemIds,
  quest_item_ids: [...new Set(questItemIds)].sort(),
  mob_item_ids: [...new Set(mobItemIds)].sort(),
  vendor_item_ids: vendorItemIds,
  constants: {
    max_level: types.MAX_LEVEL,
    prestige_xp_per_rank: types.PRESTIGE_XP_PER_RANK,
    bank_expansion_prices: [...bank.BANK_EXPANSION_PRICES],
    xp_for_level: Array.from(
      { length: types.MAX_LEVEL },
      (_, index) => types.xpForLevel(index + 1),
    ),
  },
};

process.stdout.write(JSON.stringify(result));

function selectDefinitions(table, ids, label) {
  const result = {};
  for (const id of ids) {
    if (!(id in table)) throw new Error(`unknown M5 ${label} id: ${id}`);
    result[id] = table[id];
  }
  return result;
}

function selectTalentOptions(rows, ids, label) {
  const byId = new Map(rows.flatMap((row) => row.options.map((option) => [option.id, {
    ...option,
    row_level: row.level,
  }])));
  const result = {};
  for (const id of ids) {
    const definition = byId.get(id);
    if (!definition) throw new Error(`unknown M5 ${label} id: ${id}`);
    result[id] = definition;
  }
  return result;
}
