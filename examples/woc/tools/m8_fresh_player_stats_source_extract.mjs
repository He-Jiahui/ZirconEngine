const { CLASSES } = await import('wocgit:///src/sim/content/classes.ts');
const { createPlayer, recalcPlayerStats } = await import('wocgit:///src/sim/entity.ts');

const classIds = Object.keys(CLASSES);
const players = classIds.map((classId) => freshPlayer(classId));

process.stdout.write(JSON.stringify({ players }));

function freshPlayer(classId) {
  const definition = CLASSES[classId];
  const equipment = { mainhand: definition.startWeapon, chest: definition.startChest };
  const player = createPlayer(1, classId, { x: 2, y: 1.5, z: -2 }, 'Bootstrap');
  recalcPlayerStats(player, classId, equipment, undefined, {});
  player.hp = player.maxHp;
  player.resource = definition.resourceType === 'mana'
    ? player.maxResource
    : definition.resourceType === 'energy'
      ? 100
      : 0;

  return {
    class_id: classId,
    level: player.level,
    resource_type: player.resourceType,
    resource_kind: resourceKindCode(player.resourceType),
    presentation_identity: projectPresentationIdentity(player),
    activity_state: projectActivityState(player),
    tap_ownership: projectTapOwnership(player),
    corpse_instance: projectCorpseInstance(player),
    harvest_claim: projectHarvestClaim(player),
    loot_ffa: projectLootFfa(player),
    pet_runtime: projectPetRuntime(player),
    boss_cadence: projectBossCadence(player),
    boss_special: projectBossSpecial(player),
    color: player.color,
    equipment,
    start_items: definition.startItems.map((item) => ({ item_id: item.itemId, count: item.count })),
    stats: projectStats(player.stats),
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
    mainhand_item_id: player.mainhandItemId,
    equipped_items: player.equippedItems,
    combat: projectCombat(player),
    combat_state: projectCombatState(player),
    locomotion_recovery: projectLocomotionRecovery(player),
    forced_target: projectForcedTarget(player),
    resource_cooldown: projectResourceCooldown(player),
    cast_charge_target: projectCastChargeTarget(player),
  };
}

function projectCombat(player) {
  return {
    weapon_min: player.weapon.min,
    weapon_max: player.weapon.max,
    weapon_speed: player.weapon.speed,
    offhand_weapon_min: player.offhandWeapon?.min ?? 0,
    offhand_weapon_max: player.offhandWeapon?.max ?? 0,
    offhand_weapon_speed: player.offhandWeapon?.speed ?? 0,
    has_offhand_weapon: player.offhandWeapon !== null,
    dual_wielding: player.dualWielding,
    attack_power: player.attackPower,
    ranged_power: player.rangedPower,
    spell_power: player.spellPower,
    armor: player.stats.armor,
    crit_chance: player.critChance,
    dodge_chance: player.dodgeChance,
    hit_bonus: player.hitBonus,
    crit_damage_physical_bonus: player.critDmgPhysBonus,
    melee_haste: player.meleeHaste,
    ranged_haste: player.rangedHaste,
    swing_timer: player.swingTimer,
    offhand_swing_timer: player.offhandSwingTimer,
    block_chance: player.blockChance,
    block_value: player.blockValue,
  };
}

function projectCombatState(player) {
  return {
    in_combat: player.inCombat,
    combat_timer: player.combatTimer,
    // WOS uses zero for nullable entity IDs at the plugin boundary.
    aggro_target_id: player.aggroTargetId ?? 0,
  };
}

function projectLocomotionRecovery(player) {
  const leashAnchor = player.leashAnchor;
  return {
    leash_anchor_present: leashAnchor !== null,
    leash_anchor_x: leashAnchor?.x ?? 0,
    leash_anchor_y: leashAnchor?.y ?? 0,
    leash_anchor_z: leashAnchor?.z ?? 0,
    evade_stall: player.evadeStall,
    flee_timer: player.fleeTimer,
    flee_return_timer: player.fleeReturnTimer,
    has_fled: player.hasFled,
  };
}

function projectForcedTarget(player) {
  return {
    forced_target_id: player.forcedTargetId ?? 0,
    forced_target_timer: player.forcedTargetTimer,
    shuffle_target_timer: player.shuffleTargetTimer ?? 0,
  };
}

function projectResourceCooldown(player) {
  return {
    five_second_rule: player.fiveSecondRule,
    combo_points: player.comboPoints,
    combo_until: player.comboUntil,
    overpower_until: player.overpowerUntil,
    potion_cooldown_until: player.potionCooldownUntil,
    potion_cd_remaining: player.potionCdRemaining,
    saved_mana: player.savedMana,
  };
}

function projectCastChargeTarget(player) {
  const castAim = player.castAim;
  const queuedCastAim = player.queuedCastAim;
  return {
    cast_aim_present: castAim !== null,
    cast_aim_x: castAim?.x ?? 0,
    cast_aim_y: castAim?.y ?? 0,
    cast_aim_z: castAim?.z ?? 0,
    queued_cast_aim_present: queuedCastAim !== null,
    queued_cast_aim_x: queuedCastAim?.x ?? 0,
    queued_cast_aim_y: queuedCastAim?.y ?? 0,
    queued_cast_aim_z: queuedCastAim?.z ?? 0,
    charge_target_id: player.chargeTargetId ?? 0,
    charge_time_left: player.chargeTimeLeft,
    follow_target_id: player.followTargetId ?? 0,
  };
}

function resourceKindCode(resourceType) {
  if (resourceType === null) return 0;
  if (resourceType === 'mana') return 1;
  if (resourceType === 'rage') return 2;
  if (resourceType === 'energy') return 3;
  throw new Error(`unknown WOC resource type: ${resourceType}`);
}

function projectPresentationIdentity(player) {
  return {
    scale: player.scale,
    color: player.color,
    skin_catalog: skinCatalogCode(player.skinCatalog),
    skin_index: player.skin,
  };
}

function skinCatalogCode(skinCatalog) {
  if (skinCatalog === 'class') return 1;
  if (skinCatalog === 'mech') return 2;
  throw new Error('unknown WOC skin catalog');
}

function projectActivityState(player) {
  return {
    ai_state: aiStateCode(player.aiState),
    sitting: player.sitting,
    weapon_stowed: player.weaponStowed,
  };
}

function aiStateCode(aiState) {
  if (aiState === 'idle') return 1;
  if (aiState === 'chase') return 2;
  if (aiState === 'attack') return 3;
  if (aiState === 'flee') return 4;
  if (aiState === 'evade') return 5;
  if (aiState === 'dead') return 6;
  throw new Error('unknown WOC AI state');
}

function projectTapOwnership(player) {
  return { tapped_by_id: player.tappedById ?? 0 };
}

function projectCorpseInstance(player) {
  return { instance_id: player.corpseInstanceId ?? 0 };
}

function projectHarvestClaim(player) {
  return { claimed_by_id: player.harvestClaimedBy ?? 0 };
}

function projectLootFfa(player) {
  return Number.isFinite(player.lootFfaTimer)
    ? { timer_present: true, timer_seconds: player.lootFfaTimer }
    : { timer_present: false, timer_seconds: 0 };
}

function projectPetRuntime(player) {
  return {
    mode: petModeCode(player.petMode),
    taunt_timer: player.petTauntTimer,
    auto_taunt: optionalBoolean(player.petAutoTaunt),
    auto_water_jet: optionalBoolean(player.petAutoWaterJet),
    manual_taunt_pending: optionalBoolean(player.petManualTauntPending),
    path_cooldown: player.petPathCooldown,
  };
}

function optionalBoolean(value) {
  return { present: value !== undefined, value: value ?? false };
}

function petModeCode(mode) {
  if (mode === 'passive') return 1;
  if (mode === 'defensive') return 2;
  if (mode === 'aggressive') return 3;
  throw new Error('unknown WOC pet mode');
}

function projectBossCadence(player) {
  return {
    pulse_timer: player.pulseTimer,
    stomp_timer: player.stompTimer,
    big_cast_timer: player.bigCastTimer,
    yelled_engage: player.yelledEngage,
    stoneskin_timer: player.stoneskinTimer,
  };
}

function projectBossSpecial(player) {
  return {
    terrify_timer: player.terrifyTimer,
    aoe_slow_timer: player.aoeSlowTimer,
    loud_yell_timer: player.loudYellTimer,
    loud_yell_index: player.loudYellIndex,
    detonate_timer: Number.isFinite(player.detonateTimer)
      ? { present: true, seconds: player.detonateTimer }
      : { present: false, seconds: 0 },
    mend_timer: player.mendTimer,
    ward_timer: player.wardTimer,
    channel_timer: player.channelTimer,
    channel_ramp: player.channelRamp,
    rally_timer: player.rallyTimer,
    warcry_timer: player.warcryTimer,
    fired_summons: player.firedSummons,
    enraged: player.enraged,
    healed_this_pull: player.healedThisPull,
  };
}

function projectStats(stats) {
  return {
    str: stats.str,
    agi: stats.agi,
    sta: stats.sta,
    int: stats.int,
    spi: stats.spi,
    armor: stats.armor,
    pvp_offense: stats.pvpOffense,
    pvp_defense: stats.pvpDefense,
  };
}
