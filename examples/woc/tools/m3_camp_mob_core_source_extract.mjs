const data = await import('wocgit:///src/sim/data.ts');

const ids = [];
const seen = new Set();
for (const camp of data.BUILTIN_WORLD.camps) {
  if (!seen.has(camp.mobId)) {
    seen.add(camp.mobId);
    ids.push(camp.mobId);
  }
}

const mobs = ids.map((id) => {
  const template = data.MOBS[id];
  if (!template) throw new Error(`missing camp mob template ${id}`);
  return {
    id: template.id,
    name: template.name,
    family: template.family,
    min_level: template.minLevel,
    max_level: template.maxLevel,
    hp_base: template.hpBase,
    hp_per_level: template.hpPerLevel,
    dmg_base: template.dmgBase,
    dmg_per_level: template.dmgPerLevel,
    attack_speed: template.attackSpeed,
    armor_per_level: template.armorPerLevel,
    move_speed: template.moveSpeed,
    aggro_radius: template.aggroRadius,
    scale: template.scale,
    color: template.color,
    elite: template.elite === true,
    boss: template.boss === true,
    rare: template.rare === true,
    dummy: template.dummy === true,
    can_swim: template.canSwim === true,
    has_respawn_seconds: template.respawnSeconds !== undefined,
    respawn_seconds: template.respawnSeconds ?? 0,
    has_respawn_mult: template.respawnMult !== undefined,
    respawn_mult: template.respawnMult ?? 0,
  };
});

process.stdout.write(JSON.stringify({ mobs }));
