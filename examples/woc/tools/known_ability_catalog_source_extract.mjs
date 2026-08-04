const [sourceModule] = process.argv.slice(2);
if (!sourceModule) {
  throw new Error('usage: known_ability_catalog_source_extract.mjs <source-module>');
}

const { ABILITIES, CLASSES } = await import(sourceModule);
const abilities = Object.values(ABILITIES)
  .map((definition) => {
    const primarySelfBuff = (definition.effects ?? []).find((effect) => effect.type === 'selfBuff');
    return {
    id: definition.id,
    learn_level: definition.learnLevel,
    specs: definition.specs ?? [],
    exclude_specs: definition.excludeSpecs ?? [],
    exclude_specs_at_level: definition.excludeSpecsAtLevel ?? 0,
    passive: Boolean(definition.passive),
    class_id: definition.class,
    base_cost: definition.cost,
    base_cast_time: definition.castTime,
    base_cooldown: definition.cooldown,
    school: definition.school,
    exclusive_group: definition.exclusiveGroup ?? '',
    requires_form: definition.requiresForm ?? '',
    requires_stealth: Boolean(definition.requiresStealth),
    usable_in_form: Boolean(definition.usableInForm),
    cast_while_moving: Boolean(definition.castWhileMoving),
    primary_self_buff_kind: primarySelfBuff?.kind ?? '',
    primary_self_buff_value: primarySelfBuff?.value ?? 0,
    // Preserve declaration order: abilitiesKnownAt resolves every qualifying
    // rank in that order rather than sorting rank thresholds independently.
    ranks: (definition.ranks ?? []).map((rank) => ({ rank: rank.rank, level: rank.level })),
  };
  })
  .sort((left, right) => left.id.localeCompare(right.id));
const classes = Object.entries(CLASSES).map(([id, definition]) => ({
  id,
  abilities: [...definition.abilities],
}));
process.stdout.write(JSON.stringify({ abilities, classes }));
