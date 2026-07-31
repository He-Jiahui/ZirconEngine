import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const WORLD_SEED = 20061;
const PLAYER_CLASS = 'mage';
const EXPECTED_CAMP_IDS = ['forest_wolf', 'forest_wolf', 'wild_boar', 'wild_boar'];
const EXPECTED_CAMP_COUNTS = [7, 6, 6, 5];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const outputPath = join(projectRoot, 'reference', 'current-head', 'm8_eastbrook_encounter.json');
const zrOutputPath = join(
  projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm8_eastbrook_encounter.zr',
);
const checkOnly = process.argv.includes('--check');

// This executes the pinned source tree itself. The temporary archive isolates
// extraction from a developer's dirty WOC checkout while reusing its installed
// TypeScript runtime through a junction.
const SOURCE_EXTRACTOR = `
import { Sim } from './src/sim/sim.ts';
import { BUILTIN_WORLD } from './src/sim/data.ts';

const selectedCamps = [];
for (const [sourceCampIndex, camp] of BUILTIN_WORLD.camps.entries()) {
  if (camp.mobId === 'forest_wolf' || camp.mobId === 'wild_boar') {
    selectedCamps.push({
      source_camp_index: sourceCampIndex,
      mob_id: camp.mobId,
      center_x: camp.center.x,
      center_z: camp.center.z,
      radius: camp.radius,
      count: camp.count,
    });
  }
}
const sim = new Sim({ seed: 20061, playerClass: 'mage' });
const rows = [...sim.entities.values()].filter(
  (entity) => entity.templateId === 'forest_wolf' || entity.templateId === 'wild_boar',
);
let rowIndex = 0;
const spawns = [];
function projectCombat(entity) {
  return {
    weapon_min: entity.weapon.min,
    weapon_max: entity.weapon.max,
    weapon_speed: entity.weapon.speed,
    offhand_weapon_min: entity.offhandWeapon?.min ?? 0,
    offhand_weapon_max: entity.offhandWeapon?.max ?? 0,
    offhand_weapon_speed: entity.offhandWeapon?.speed ?? 0,
    has_offhand_weapon: entity.offhandWeapon !== null,
    dual_wielding: entity.dualWielding,
    attack_power: entity.attackPower,
    ranged_power: entity.rangedPower,
    spell_power: entity.spellPower,
    armor: entity.stats.armor,
    crit_chance: entity.critChance,
    dodge_chance: entity.dodgeChance,
    hit_bonus: entity.hitBonus,
    crit_damage_physical_bonus: entity.critDmgPhysBonus,
    melee_haste: entity.meleeHaste,
    ranged_haste: entity.rangedHaste,
    swing_timer: entity.swingTimer,
    offhand_swing_timer: entity.offhandSwingTimer,
    block_chance: entity.blockChance,
    block_value: entity.blockValue,
  };
}
function projectCombatState(entity) {
  return {
    in_combat: entity.inCombat,
    combat_timer: entity.combatTimer,
    // WOS uses zero for nullable entity IDs at the plugin boundary.
    aggro_target_id: entity.aggroTargetId ?? 0,
  };
}
function projectLocomotionRecovery(entity) {
  const leashAnchor = entity.leashAnchor;
  return {
    leash_anchor_present: leashAnchor !== null,
    leash_anchor_x: leashAnchor?.x ?? 0,
    leash_anchor_y: leashAnchor?.y ?? 0,
    leash_anchor_z: leashAnchor?.z ?? 0,
    evade_stall: entity.evadeStall,
    flee_timer: entity.fleeTimer,
    flee_return_timer: entity.fleeReturnTimer,
    has_fled: entity.hasFled,
  };
}
function projectForcedTarget(entity) {
  return {
    forced_target_id: entity.forcedTargetId ?? 0,
    forced_target_timer: entity.forcedTargetTimer,
    shuffle_target_timer: entity.shuffleTargetTimer ?? 0,
  };
}
function projectResourceCooldown(entity) {
  return {
    five_second_rule: entity.fiveSecondRule,
    combo_points: entity.comboPoints,
    combo_until: entity.comboUntil,
    overpower_until: entity.overpowerUntil,
    potion_cooldown_until: entity.potionCooldownUntil,
    potion_cd_remaining: entity.potionCdRemaining,
    saved_mana: entity.savedMana,
  };
}
function projectCastChargeTarget(entity) {
  const castAim = entity.castAim;
  const queuedCastAim = entity.queuedCastAim;
  return {
    cast_aim_present: castAim !== null,
    cast_aim_x: castAim?.x ?? 0,
    cast_aim_y: castAim?.y ?? 0,
    cast_aim_z: castAim?.z ?? 0,
    queued_cast_aim_present: queuedCastAim !== null,
    queued_cast_aim_x: queuedCastAim?.x ?? 0,
    queued_cast_aim_y: queuedCastAim?.y ?? 0,
    queued_cast_aim_z: queuedCastAim?.z ?? 0,
    charge_target_id: entity.chargeTargetId ?? 0,
    charge_time_left: entity.chargeTimeLeft,
    follow_target_id: entity.followTargetId ?? 0,
  };
}
function resourceKindCode(resourceType) {
  if (resourceType === null) return 0;
  if (resourceType === 'mana') return 1;
  if (resourceType === 'rage') return 2;
  if (resourceType === 'energy') return 3;
  throw new Error(\`unknown WOC resource type: \${resourceType}\`);
}
function projectPresentationIdentity(entity) {
  return {
    scale: entity.scale,
    color: entity.color,
    skin_catalog: skinCatalogCode(entity.skinCatalog),
    skin_index: entity.skin,
  };
}
function skinCatalogCode(skinCatalog) {
  if (skinCatalog === 'class') return 1;
  if (skinCatalog === 'mech') return 2;
  throw new Error('unknown WOC skin catalog');
}
function projectActivityState(entity) {
  return {
    ai_state: aiStateCode(entity.aiState),
    sitting: entity.sitting,
    weapon_stowed: entity.weaponStowed,
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
function projectTapOwnership(entity) {
  return { tapped_by_id: entity.tappedById ?? 0 };
}
function projectCorpseInstance(entity) {
  return { instance_id: entity.corpseInstanceId ?? 0 };
}
function projectHarvestClaim(entity) {
  return { claimed_by_id: entity.harvestClaimedBy ?? 0 };
}
function projectLootFfa(entity) {
  return Number.isFinite(entity.lootFfaTimer)
    ? { timer_present: true, timer_seconds: entity.lootFfaTimer }
    : { timer_present: false, timer_seconds: 0 };
}
function projectPetRuntime(entity) {
  return {
    mode: petModeCode(entity.petMode),
    taunt_timer: entity.petTauntTimer,
    auto_taunt: optionalBoolean(entity.petAutoTaunt),
    auto_water_jet: optionalBoolean(entity.petAutoWaterJet),
    manual_taunt_pending: optionalBoolean(entity.petManualTauntPending),
    path_cooldown: entity.petPathCooldown,
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
function projectBossCadence(entity) {
  return {
    pulse_timer: entity.pulseTimer,
    stomp_timer: entity.stompTimer,
    big_cast_timer: entity.bigCastTimer,
    yelled_engage: entity.yelledEngage,
    stoneskin_timer: entity.stoneskinTimer,
  };
}
function projectBossSpecial(entity) {
  return {
    terrify_timer: entity.terrifyTimer,
    aoe_slow_timer: entity.aoeSlowTimer,
    loud_yell_timer: entity.loudYellTimer,
    loud_yell_index: entity.loudYellIndex,
    detonate_timer: Number.isFinite(entity.detonateTimer)
      ? { present: true, seconds: entity.detonateTimer }
      : { present: false, seconds: 0 },
    mend_timer: entity.mendTimer,
    ward_timer: entity.wardTimer,
    channel_timer: entity.channelTimer,
    channel_ramp: entity.channelRamp,
    rally_timer: entity.rallyTimer,
    warcry_timer: entity.warcryTimer,
    fired_summons: entity.firedSummons,
    enraged: entity.enraged,
    healed_this_pull: entity.healedThisPull,
  };
}
for (const camp of selectedCamps) {
  for (let campMemberIndex = 0; campMemberIndex < camp.count; campMemberIndex++) {
    const entity = rows[rowIndex++];
    if (!entity || entity.templateId !== camp.mob_id) {
      throw new Error('selected camp/entity ordering drifted');
    }
    spawns.push({
      source_entity_id: entity.id,
      source_camp_index: camp.source_camp_index,
      camp_member_index: campMemberIndex,
      mob_id: entity.templateId,
      level: entity.level,
      resource_type: entity.resourceType,
      resource_kind: resourceKindCode(entity.resourceType),
      presentation_identity: projectPresentationIdentity(entity),
      activity_state: projectActivityState(entity),
      tap_ownership: projectTapOwnership(entity),
      corpse_instance: projectCorpseInstance(entity),
      harvest_claim: projectHarvestClaim(entity),
      loot_ffa: projectLootFfa(entity),
      pet_runtime: projectPetRuntime(entity),
      boss_cadence: projectBossCadence(entity),
      boss_special: projectBossSpecial(entity),
      x: entity.pos.x,
      y: entity.pos.y,
      z: entity.pos.z,
      max_hp: entity.maxHp,
      move_speed: entity.moveSpeed,
      facing: entity.facing,
      wander_timer: entity.wanderTimer,
      combat: projectCombat(entity),
      combat_state: projectCombatState(entity),
      locomotion_recovery: projectLocomotionRecovery(entity),
      forced_target: projectForcedTarget(entity),
      resource_cooldown: projectResourceCooldown(entity),
      cast_charge_target: projectCastChargeTarget(entity),
    });
  }
}
if (rowIndex !== rows.length) throw new Error('selected encounter count drifted');
process.stdout.write(JSON.stringify({ camps: selectedCamps, spawns }));
`;

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  invariant(sourceManifest.source_commit === SOURCE_COMMIT, 'reference source commit drifted');
  const extracted = extractPinnedSource();
  validateExtracted(extracted);
  const catalog = {
    schema_version: 17,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m8_eastbrook_encounter_codegen.mjs',
    extraction: {
      world_seed: WORLD_SEED,
      player_class: PLAYER_CLASS,
      simulator: 'Sim builtin world constructor before any tick',
    },
    source_identities: sourceIdentities(),
    camps: extracted.camps,
    spawns: extracted.spawns,
  };
  catalog.catalog_sha256 = catalogHash(catalog);
  writeOrCheck(outputPath, `${JSON.stringify(catalog, null, 2)}\n`);
  writeOrCheck(zrOutputPath, renderZr(catalog));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} M8 Eastbrook encounter: ` +
    `${catalog.camps.length} camps, ${catalog.spawns.length} source spawns ` +
    `(${catalog.catalog_sha256.slice(0, 15)})\n`,
  );
}

function extractPinnedSource() {
  const isolatedRoot = mkdtempSync(join(tmpdir(), 'woc-m8-encounter-'));
  try {
    const archive = execFileSync(
      'git', ['-C', sourceRoot, 'archive', '--format=tar', SOURCE_COMMIT, 'src'],
      { maxBuffer: 64 * 1024 * 1024 },
    );
    const untar = spawnSync('tar', ['-xf', '-', '-C', isolatedRoot], {
      input: archive,
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
    });
    invariant(untar.status === 0, untar.stderr || 'unable to unpack pinned WOC source');
    const nodeModules = join(sourceRoot, 'node_modules');
    invariant(existsSync(nodeModules), 'pinned WOC extraction requires source node_modules');
    symlinkSync(nodeModules, join(isolatedRoot, 'node_modules'), 'junction');
    const tsxCli = join(nodeModules, 'tsx', 'dist', 'cli.mjs');
    invariant(existsSync(tsxCli), 'pinned WOC extraction requires tsx');
    const child = spawnSync(process.execPath, [tsxCli, '-e', SOURCE_EXTRACTOR], {
      cwd: isolatedRoot,
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    });
    invariant(child.status === 0, child.stderr || `encounter extractor exited ${child.status}`);
    return JSON.parse(child.stdout);
  } finally {
    rmSync(isolatedRoot, { recursive: true, force: true, maxRetries: 3 });
  }
}

function validateExtracted(extracted) {
  invariant(extracted && typeof extracted === 'object', 'encounter extraction is not an object');
  invariant(Array.isArray(extracted.camps) && Array.isArray(extracted.spawns), 'encounter data is missing');
  invariant(extracted.camps.length === EXPECTED_CAMP_IDS.length, 'selected camp count drifted');
  for (const spawn of extracted.spawns) {
    invariant(spawn.presentation_identity && typeof spawn.presentation_identity === 'object',
      'selected spawn presentation identity is missing');
    const presentation = spawn.presentation_identity;
    invariant(Number.isFinite(presentation.scale) && presentation.scale > 0 &&
      Number.isSafeInteger(presentation.color) && presentation.color >= 0 &&
      presentation.color <= 0xffffff &&
      (presentation.skin_catalog === 1 || presentation.skin_catalog === 2) &&
      Number.isSafeInteger(presentation.skin_index) && presentation.skin_index >= 0,
    'selected spawn presentation identity is invalid');
    invariant(spawn.activity_state && typeof spawn.activity_state === 'object',
      'selected spawn activity state is missing');
    const activity = spawn.activity_state;
    invariant(Number.isSafeInteger(activity.ai_state) && activity.ai_state >= 1 &&
      activity.ai_state <= 6 && typeof activity.sitting === 'boolean' &&
      typeof activity.weapon_stowed === 'boolean',
    'selected spawn activity state is invalid');
    invariant(activity.ai_state === 1 && activity.sitting === false &&
      activity.weapon_stowed === false,
    'selected spawn activity-state initializer drifted');
    invariant(spawn.tap_ownership && typeof spawn.tap_ownership === 'object' &&
      Number.isSafeInteger(spawn.tap_ownership.tapped_by_id) &&
      spawn.tap_ownership.tapped_by_id >= 0,
    'selected spawn tap ownership is invalid');
    invariant(spawn.tap_ownership.tapped_by_id === 0,
      'selected spawn tap-ownership initializer drifted');
    invariant(spawn.corpse_instance && typeof spawn.corpse_instance === 'object' &&
      Number.isSafeInteger(spawn.corpse_instance.instance_id) &&
      spawn.corpse_instance.instance_id >= 0,
    'selected spawn corpse instance is invalid');
    invariant(spawn.corpse_instance.instance_id === 0,
      'selected spawn corpse-instance initializer drifted');
    invariant(spawn.harvest_claim && typeof spawn.harvest_claim === 'object' &&
      Number.isSafeInteger(spawn.harvest_claim.claimed_by_id) &&
      spawn.harvest_claim.claimed_by_id >= 0,
    'selected spawn harvest claim is invalid');
    invariant(spawn.harvest_claim.claimed_by_id === 0,
      'selected spawn harvest-claim initializer drifted');
    invariant(spawn.loot_ffa && typeof spawn.loot_ffa === 'object' &&
      typeof spawn.loot_ffa.timer_present === 'boolean' &&
      Number.isFinite(spawn.loot_ffa.timer_seconds) && spawn.loot_ffa.timer_seconds >= 0,
    'selected spawn loot FFA state is invalid');
    invariant(spawn.loot_ffa.timer_present === false && spawn.loot_ffa.timer_seconds === 0,
      'selected spawn loot-FFA initializer drifted');
    const pet = spawn.pet_runtime;
    invariant(pet && typeof pet === 'object' && Number.isSafeInteger(pet.mode) &&
      pet.mode >= 1 && pet.mode <= 3 && Number.isFinite(pet.taunt_timer) &&
      pet.taunt_timer >= 0 && Number.isFinite(pet.path_cooldown) && pet.path_cooldown >= 0,
    'selected spawn pet runtime state is invalid');
    for (const field of ['auto_taunt', 'auto_water_jet', 'manual_taunt_pending']) {
      invariant(typeof pet[field]?.present === 'boolean' && typeof pet[field]?.value === 'boolean' &&
        (pet[field].present || pet[field].value === false),
      'selected spawn pet optional state is invalid: ' + field);
    }
    invariant(pet.mode === 2 && pet.taunt_timer === 0 && pet.path_cooldown === 0 &&
      pet.auto_taunt.present === false && pet.auto_taunt.value === false &&
      pet.auto_water_jet.present === false && pet.auto_water_jet.value === false &&
      pet.manual_taunt_pending.present === false && pet.manual_taunt_pending.value === false,
    'selected spawn pet runtime initializer drifted');
    const cadence = spawn.boss_cadence;
    invariant(cadence && typeof cadence === 'object' &&
      [cadence.pulse_timer, cadence.stomp_timer, cadence.big_cast_timer,
        cadence.stoneskin_timer].every((value) => Number.isFinite(value) && value >= 0) &&
      typeof cadence.yelled_engage === 'boolean',
    'selected spawn boss cadence state is invalid');
    invariant(cadence.pulse_timer === 0 && cadence.stomp_timer === 0 &&
      cadence.big_cast_timer === 0 && cadence.yelled_engage === false &&
      cadence.stoneskin_timer === 0,
    'selected spawn boss cadence initializer drifted');
    const special = spawn.boss_special;
    invariant(special && typeof special === 'object' &&
      [special.terrify_timer, special.aoe_slow_timer, special.loud_yell_timer,
        special.mend_timer, special.ward_timer, special.channel_timer, special.channel_ramp,
        special.rally_timer, special.warcry_timer].every((value) => Number.isFinite(value) && value >= 0) &&
      Number.isSafeInteger(special.loud_yell_index) && special.loud_yell_index >= 0 &&
      special.detonate_timer && typeof special.detonate_timer === 'object' &&
      typeof special.detonate_timer.present === 'boolean' &&
      Number.isFinite(special.detonate_timer.seconds) && special.detonate_timer.seconds >= 0 &&
      Number.isSafeInteger(special.fired_summons) && special.fired_summons >= 0 &&
      typeof special.enraged === 'boolean' && typeof special.healed_this_pull === 'boolean',
    'selected spawn boss special state is invalid');
    invariant(special.terrify_timer === 0 && special.aoe_slow_timer === 0 &&
      special.loud_yell_timer === 0 && special.loud_yell_index === 0 &&
      special.detonate_timer.present === false && special.detonate_timer.seconds === 0 &&
      special.mend_timer === 0 && special.ward_timer === 0 && special.channel_timer === 0 &&
      special.channel_ramp === 0 && special.rally_timer === 0 && special.warcry_timer === 0 &&
      special.fired_summons === 0 && special.enraged === false && special.healed_this_pull === false,
    'selected spawn boss special initializer drifted');
  }
  invariant(
    JSON.stringify(extracted.camps.map((camp) => camp.mob_id)) === JSON.stringify(EXPECTED_CAMP_IDS),
    'selected camp order drifted',
  );
  invariant(
    JSON.stringify(extracted.camps.map((camp) => camp.count)) === JSON.stringify(EXPECTED_CAMP_COUNTS),
    'selected camp membership drifted',
  );
  invariant(extracted.spawns.length === 24, 'selected spawn count drifted');
  for (const camp of extracted.camps) {
    invariant(
      Number.isInteger(camp.source_camp_index) && camp.source_camp_index >= 0 &&
      typeof camp.mob_id === 'string' && camp.mob_id.length > 0 &&
      Number.isFinite(camp.center_x) && Number.isFinite(camp.center_z) &&
      Number.isFinite(camp.radius) && camp.radius > 0 &&
      Number.isInteger(camp.count) && camp.count > 0,
      'selected camp row is invalid',
    );
  }
  for (const spawn of extracted.spawns) {
    invariant(
      Number.isInteger(spawn.source_entity_id) && spawn.source_entity_id > 0 &&
      Number.isInteger(spawn.source_camp_index) && spawn.source_camp_index >= 0 &&
      Number.isInteger(spawn.camp_member_index) && spawn.camp_member_index >= 0 &&
      (spawn.mob_id === 'forest_wolf' || spawn.mob_id === 'wild_boar') &&
      Number.isInteger(spawn.level) && spawn.level >= 1 &&
      Number.isInteger(spawn.max_hp) && spawn.max_hp > 0 &&
      [spawn.x, spawn.y, spawn.z, spawn.move_speed, spawn.facing, spawn.wander_timer].every(Number.isFinite),
      'selected spawn row is invalid',
    );
    invariant(spawn.combat && typeof spawn.combat === 'object', 'selected spawn combat is missing');
    for (const field of [
      'weapon_min', 'weapon_max', 'weapon_speed', 'offhand_weapon_min', 'offhand_weapon_max',
      'offhand_weapon_speed', 'attack_power', 'ranged_power', 'spell_power', 'armor',
      'crit_chance', 'dodge_chance', 'hit_bonus', 'crit_damage_physical_bonus', 'melee_haste',
      'ranged_haste', 'swing_timer', 'offhand_swing_timer', 'block_chance', 'block_value',
    ]) {
      invariant(Number.isFinite(spawn.combat[field]), `selected spawn combat ${field} is invalid`);
    }
    invariant(typeof spawn.combat.has_offhand_weapon === 'boolean' &&
      typeof spawn.combat.dual_wielding === 'boolean', 'selected spawn combat flags are invalid');
    invariant(spawn.combat.weapon_min > 0 &&
      spawn.combat.weapon_max >= spawn.combat.weapon_min &&
      spawn.combat.weapon_speed > 0 &&
      spawn.combat.armor >= 0,
    'selected spawn combat profile is invalid');
    invariant(spawn.combat_state && typeof spawn.combat_state === 'object',
      'selected spawn combat state is missing');
    invariant(typeof spawn.combat_state.in_combat === 'boolean' &&
      Number.isFinite(spawn.combat_state.combat_timer) &&
      Number.isSafeInteger(spawn.combat_state.aggro_target_id) &&
      spawn.combat_state.combat_timer >= 0 && spawn.combat_state.aggro_target_id >= 0,
    'selected spawn combat state is invalid');
    invariant(spawn.combat_state.in_combat === false &&
      spawn.combat_state.combat_timer === 99 && spawn.combat_state.aggro_target_id === 0,
    'selected spawn combat state initializer drifted');
    invariant(spawn.locomotion_recovery && typeof spawn.locomotion_recovery === 'object',
      'selected spawn locomotion recovery is missing');
    const recovery = spawn.locomotion_recovery;
    invariant(typeof recovery.leash_anchor_present === 'boolean' &&
      typeof recovery.has_fled === 'boolean' &&
      [recovery.leash_anchor_x, recovery.leash_anchor_y, recovery.leash_anchor_z,
        recovery.evade_stall, recovery.flee_timer, recovery.flee_return_timer].every(Number.isFinite),
    'selected spawn locomotion recovery is invalid');
    invariant(recovery.leash_anchor_present === false && recovery.leash_anchor_x === 0 &&
      recovery.leash_anchor_y === 0 && recovery.leash_anchor_z === 0 &&
      recovery.evade_stall === 0 && recovery.flee_timer === 0 &&
      recovery.flee_return_timer === 0 && recovery.has_fled === false,
    'selected spawn locomotion recovery initializer drifted');
    invariant(spawn.forced_target && typeof spawn.forced_target === 'object',
      'selected spawn forced target is missing');
    const forcedTarget = spawn.forced_target;
    invariant(Number.isSafeInteger(forcedTarget.forced_target_id) &&
      Number.isFinite(forcedTarget.forced_target_timer) &&
      Number.isFinite(forcedTarget.shuffle_target_timer) && forcedTarget.forced_target_id >= 0 &&
      forcedTarget.forced_target_timer >= 0 && forcedTarget.shuffle_target_timer >= 0,
    'selected spawn forced target is invalid');
    invariant(forcedTarget.forced_target_id === 0 && forcedTarget.forced_target_timer === 0 &&
      forcedTarget.shuffle_target_timer === 0,
    'selected spawn forced-target initializer drifted');
    invariant(spawn.resource_cooldown && typeof spawn.resource_cooldown === 'object',
      'selected spawn resource cooldown is missing');
    const resourceCooldown = spawn.resource_cooldown;
    invariant(Number.isSafeInteger(resourceCooldown.combo_points) &&
      Number.isSafeInteger(resourceCooldown.saved_mana) &&
      [resourceCooldown.five_second_rule, resourceCooldown.combo_until,
        resourceCooldown.overpower_until, resourceCooldown.potion_cooldown_until,
        resourceCooldown.potion_cd_remaining].every(Number.isFinite),
    'selected spawn resource cooldown is invalid');
    invariant(resourceCooldown.five_second_rule === 99 && resourceCooldown.combo_points === 0 &&
      resourceCooldown.combo_until === -1 && resourceCooldown.overpower_until === -1 &&
      resourceCooldown.potion_cooldown_until === -1 && resourceCooldown.potion_cd_remaining === 0 &&
      resourceCooldown.saved_mana === 0,
    'selected spawn resource-cooldown initializer drifted');
    invariant(spawn.cast_charge_target && typeof spawn.cast_charge_target === 'object',
      'selected spawn cast-charge target is missing');
    const castChargeTarget = spawn.cast_charge_target;
    invariant(typeof castChargeTarget.cast_aim_present === 'boolean' &&
      typeof castChargeTarget.queued_cast_aim_present === 'boolean' &&
      Number.isSafeInteger(castChargeTarget.charge_target_id) &&
      Number.isSafeInteger(castChargeTarget.follow_target_id) &&
      [castChargeTarget.cast_aim_x, castChargeTarget.cast_aim_y, castChargeTarget.cast_aim_z,
        castChargeTarget.queued_cast_aim_x, castChargeTarget.queued_cast_aim_y,
        castChargeTarget.queued_cast_aim_z, castChargeTarget.charge_time_left].every(Number.isFinite) &&
      castChargeTarget.charge_target_id >= 0 && castChargeTarget.follow_target_id >= 0 &&
      castChargeTarget.charge_time_left >= 0,
    'selected spawn cast-charge target is invalid');
    invariant(castChargeTarget.cast_aim_present === false && castChargeTarget.cast_aim_x === 0 &&
      castChargeTarget.cast_aim_y === 0 && castChargeTarget.cast_aim_z === 0 &&
      castChargeTarget.queued_cast_aim_present === false &&
      castChargeTarget.queued_cast_aim_x === 0 && castChargeTarget.queued_cast_aim_y === 0 &&
      castChargeTarget.queued_cast_aim_z === 0 && castChargeTarget.charge_target_id === 0 &&
      castChargeTarget.charge_time_left === 0 && castChargeTarget.follow_target_id === 0,
    'selected spawn cast-charge target initializer drifted');
    invariant(Number.isSafeInteger(spawn.resource_kind) && spawn.resource_kind >= 0 &&
      spawn.resource_kind <= 3 && spawn.resource_kind === resourceKindCode(spawn.resource_type),
    'selected spawn resource kind drifted');
  }
  invariant(
    extracted.spawns[0].source_entity_id === 32 && extracted.spawns.at(-1).source_entity_id === 56,
    'selected source entity ordering drifted',
  );
}

function sourceIdentities() {
  const paths = [
    'src/sim/sim.ts',
    'src/sim/entity.ts',
    'src/sim/types.ts',
    'src/sim/rng.ts',
    'src/sim/data.ts',
    'src/sim/content/zone1.ts',
    'src/sim/content/zone2.ts',
    'src/sim/content/zone3.ts',
    'src/sim/combat/damage.ts',
    'src/sim/mob/lifecycle.ts',
    'src/sim/spirit.ts',
    'src/sim/instances/dungeons.ts',
    'src/sim/interaction.ts',
    'src/sim/loot/loot_ffa.ts',
    'src/sim/loot/loot_roll.ts',
    'src/sim/mob/locomotion.ts',
    'src/sim/pet/pet_ai.ts',
    'src/sim/pet/pet_commands.ts',
  ];
  return {
    representation: 'git_blob_lf',
    files: paths.map((path) => textIdentity(path, gitShow(path))),
  };
}

function renderZr(catalog) {
  const lines = [
    '// Generated by examples/woc/tools/m8_eastbrook_encounter_codegen.mjs.',
    `// Source ${catalog.source_commit}; fixed-seed Sim constructor snapshot; do not edit.`,
    '',
    'pub catalogSha(): string {',
    `    return ${zrString(catalog.catalog_sha256)};`,
    '}',
    '',
    'pub campCount(): int {',
    `    return ${catalog.camps.length};`,
    '}',
    '',
    'pub spawnCount(): int {',
    `    return ${catalog.spawns.length};`,
    '}',
    '',
  ];
  renderCampValue(lines, catalog.camps, 'campMobId', 'mob_id', zrString, 'string');
  renderCampValue(lines, catalog.camps, 'campSourceIndex', 'source_camp_index', zrInteger, 'int');
  renderCampValue(lines, catalog.camps, 'campCenterX', 'center_x', zrNumber, 'float');
  renderCampValue(lines, catalog.camps, 'campCenterZ', 'center_z', zrNumber, 'float');
  renderCampValue(lines, catalog.camps, 'campRadius', 'radius', zrNumber, 'float');
  renderCampValue(lines, catalog.camps, 'campMemberCount', 'count', zrInteger, 'int');
  renderSpawnValue(lines, catalog.spawns, 'sourceEntityId', 'source_entity_id', zrInteger, 'int');
  renderSpawnValue(lines, catalog.spawns, 'sourceCampIndex', 'source_camp_index', zrInteger, 'int');
  renderSpawnValue(lines, catalog.spawns, 'campMemberIndex', 'camp_member_index', zrInteger, 'int');
  renderSpawnValue(lines, catalog.spawns, 'mobId', 'mob_id', zrString, 'string');
  renderSpawnValue(lines, catalog.spawns, 'level', 'level', zrInteger, 'int');
  renderSpawnValue(lines, catalog.spawns, 'x', 'x', zrNumber, 'float');
  renderSpawnValue(lines, catalog.spawns, 'y', 'y', zrNumber, 'float');
  renderSpawnValue(lines, catalog.spawns, 'z', 'z', zrNumber, 'float');
  renderSpawnValue(lines, catalog.spawns, 'maxHp', 'max_hp', zrInteger, 'int');
  renderSpawnValue(lines, catalog.spawns, 'moveSpeed', 'move_speed', zrNumber, 'float');
  renderSpawnValue(lines, catalog.spawns, 'facing', 'facing', zrNumber, 'float');
  renderSpawnValue(lines, catalog.spawns, 'wanderTimer', 'wander_timer', zrNumber, 'float');
  renderSpawnCombatValue(lines, catalog.spawns, 'combatInteger', [
    ['weaponMin', 'weapon_min'], ['weaponMax', 'weapon_max'],
    ['offhandWeaponMin', 'offhand_weapon_min'], ['offhandWeaponMax', 'offhand_weapon_max'],
    ['attackPower', 'attack_power'], ['rangedPower', 'ranged_power'],
    ['spellPower', 'spell_power'], ['armor', 'armor'], ['blockValue', 'block_value'],
  ], zrInteger, 'int');
  renderSpawnCombatValue(lines, catalog.spawns, 'combatDecimal', [
    ['weaponSpeed', 'weapon_speed'], ['offhandWeaponSpeed', 'offhand_weapon_speed'],
    ['critChance', 'crit_chance'], ['dodgeChance', 'dodge_chance'], ['hitBonus', 'hit_bonus'],
    ['critDamagePhysicalBonus', 'crit_damage_physical_bonus'], ['meleeHaste', 'melee_haste'],
    ['rangedHaste', 'ranged_haste'], ['swingTimer', 'swing_timer'],
    ['offhandSwingTimer', 'offhand_swing_timer'], ['blockChance', 'block_chance'],
  ], zrNumber, 'float');
  renderSpawnCombatValue(lines, catalog.spawns, 'combatFlag', [
    ['hasOffhandWeapon', 'has_offhand_weapon'], ['dualWielding', 'dual_wielding'],
  ], (value) => value ? 'true' : 'false', 'bool');
  renderSpawnCombatValue(lines, catalog.spawns, 'combatStateFlag', [
    ['inCombat', 'in_combat'],
  ], (value) => value ? 'true' : 'false', 'bool', 'combat_state');
  renderSpawnCombatValue(lines, catalog.spawns, 'combatStateDecimal', [
    ['combatTimer', 'combat_timer'],
  ], zrNumber, 'float', 'combat_state');
  renderSpawnCombatStateTargetId(lines, catalog.spawns);
  renderSpawnCombatValue(lines, catalog.spawns, 'locomotionRecoveryFlag', [
    ['leashAnchorPresent', 'leash_anchor_present'], ['hasFled', 'has_fled'],
  ], (value) => value ? 'true' : 'false', 'bool', 'locomotion_recovery');
  renderSpawnCombatValue(lines, catalog.spawns, 'locomotionRecoveryDecimal', [
    ['leashAnchorX', 'leash_anchor_x'], ['leashAnchorY', 'leash_anchor_y'],
    ['leashAnchorZ', 'leash_anchor_z'], ['evadeStall', 'evade_stall'],
    ['fleeTimer', 'flee_timer'], ['fleeReturnTimer', 'flee_return_timer'],
  ], zrNumber, 'float', 'locomotion_recovery');
  renderSpawnForcedTargetId(lines, catalog.spawns);
  renderSpawnCombatValue(lines, catalog.spawns, 'forcedTargetDecimal', [
    ['forcedTargetTimer', 'forced_target_timer'],
    ['shuffleTargetTimer', 'shuffle_target_timer'],
  ], zrNumber, 'float', 'forced_target');
  renderSpawnCombatValue(lines, catalog.spawns, 'resourceCooldownInteger', [
    ['comboPoints', 'combo_points'], ['savedMana', 'saved_mana'],
  ], zrInteger, 'int', 'resource_cooldown');
  renderSpawnCombatValue(lines, catalog.spawns, 'resourceCooldownDecimal', [
    ['fiveSecondRule', 'five_second_rule'], ['comboUntil', 'combo_until'],
    ['overpowerUntil', 'overpower_until'], ['potionCooldownUntil', 'potion_cooldown_until'],
    ['potionCooldownRemaining', 'potion_cd_remaining'],
  ], zrNumber, 'float', 'resource_cooldown');
  renderSpawnCombatValue(lines, catalog.spawns, 'castChargeTargetFlag', [
    ['castAimPresent', 'cast_aim_present'], ['queuedCastAimPresent', 'queued_cast_aim_present'],
  ], (value) => value ? 'true' : 'false', 'bool', 'cast_charge_target');
  renderSpawnCombatValue(lines, catalog.spawns, 'castChargeTargetDecimal', [
    ['castAimX', 'cast_aim_x'], ['castAimY', 'cast_aim_y'], ['castAimZ', 'cast_aim_z'],
    ['queuedCastAimX', 'queued_cast_aim_x'], ['queuedCastAimY', 'queued_cast_aim_y'],
    ['queuedCastAimZ', 'queued_cast_aim_z'], ['chargeTimeLeft', 'charge_time_left'],
  ], zrNumber, 'float', 'cast_charge_target');
  renderSpawnCastChargeTargetId(lines, catalog.spawns);
  renderSpawnResourceKind(lines, catalog.spawns);
  renderSpawnPresentationIdentity(lines, catalog.spawns);
  renderSpawnActivityState(lines, catalog.spawns);
  renderSpawnTapOwnership(lines, catalog.spawns);
  renderSpawnCorpseInstance(lines, catalog.spawns);
  renderSpawnHarvestClaim(lines, catalog.spawns);
  renderSpawnLootFfa(lines, catalog.spawns);
  renderSpawnPetRuntime(lines, catalog.spawns);
  renderSpawnBossCadence(lines, catalog.spawns);
  renderSpawnBossSpecial(lines, catalog.spawns);
  lines.push(...renderContractTest(catalog));
  return `${lines.join('\n')}\n`;
}

function renderCampValue(lines, camps, functionName, field, formatter, type) {
  lines.push(`pub ${functionName}(campIndex: int): ${type} {`);
  for (const [index, camp] of camps.entries()) {
    lines.push(`    if (campIndex == ${index}) { return ${formatter(camp[field])}; }`);
  }
  lines.push('    throw "unknown WOC M8 encounter camp";', '}', '');
}

function renderSpawnValue(lines, spawns, functionName, field, formatter, type) {
  lines.push(`pub ${functionName}(spawnIndex: int): ${type} {`);
  for (const [index, spawn] of spawns.entries()) {
    lines.push(`    if (spawnIndex == ${index}) { return ${formatter(spawn[field])}; }`);
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnCombatValue(lines, spawns, functionName, fields, formatter, type, sourceProperty = 'combat') {
  lines.push(`pub ${functionName}(spawnIndex: int, field: string): ${type} {`);
  for (const [index, spawn] of spawns.entries()) {
    lines.push(`    if (spawnIndex == ${index}) {`);
    for (const [name, sourceField] of fields) {
      lines.push(`        if (field == ${zrString(name)}) { return ${formatter(spawn[sourceProperty][sourceField])}; }`);
    }
    lines.push(`        return ${type === 'bool' ? 'false' : (type === 'int' || type === 'uint') ? '0' : '0.0'};`, '    }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnCombatStateTargetId(lines, spawns) {
  lines.push('pub combatStateTargetId(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push(`    if (spawnIndex == ${index}) { return <uint>${zrInteger(spawn.combat_state.aggro_target_id)}; }`);
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnForcedTargetId(lines, spawns) {
  lines.push('pub forcedTargetId(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push(`    if (spawnIndex == ${index}) { return <uint>${zrInteger(spawn.forced_target.forced_target_id)}; }`);
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnCastChargeTargetId(lines, spawns) {
  lines.push('pub castChargeTargetId(spawnIndex: int, field: string): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push(`    if (spawnIndex == ${index}) {`);
    const values = {
      chargeTargetId: spawn.cast_charge_target.charge_target_id,
      followTargetId: spawn.cast_charge_target.follow_target_id,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return <uint>${zrInteger(value)}; }`);
    }
    lines.push('        return <uint>0;', '    }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnResourceKind(lines, spawns) {
  lines.push('pub resourceKind(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push(`    if (spawnIndex == ${index}) { return <uint>${zrInteger(spawn.resource_kind)}; }`);
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnPresentationIdentity(lines, spawns) {
  lines.push('pub presentationIdentityScale(spawnIndex: int): float {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return ' +
      zrNumber(spawn.presentation_identity.scale) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  lines.push('pub presentationIdentityColor(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.presentation_identity.color) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  lines.push('pub presentationIdentitySkinCatalog(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.presentation_identity.skin_catalog) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  lines.push('pub presentationIdentitySkinIndex(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.presentation_identity.skin_index) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
}

function renderSpawnActivityState(lines, spawns) {
  lines.push('pub activityStateAiState(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.activity_state.ai_state) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  lines.push('pub activityStateSitting(spawnIndex: int): bool {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return ' +
      (spawn.activity_state.sitting ? 'true' : 'false') + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  lines.push('pub activityStateWeaponStowed(spawnIndex: int): bool {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return ' +
      (spawn.activity_state.weapon_stowed ? 'true' : 'false') + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}');
}

function renderSpawnTapOwnership(lines, spawns) {
  lines.push('pub tapOwnershipId(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.tap_ownership.tapped_by_id) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}');
}

function renderSpawnCorpseInstance(lines, spawns) {
  lines.push('pub corpseInstanceId(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.corpse_instance.instance_id) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}');
}

function renderSpawnHarvestClaim(lines, spawns) {
  lines.push('pub harvestClaimId(spawnIndex: int): uint {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return <uint>' +
      zrInteger(spawn.harvest_claim.claimed_by_id) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}');
}

function renderSpawnLootFfa(lines, spawns) {
  lines.push('pub lootFfaTimerPresent(spawnIndex: int): bool {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return ' +
      (spawn.loot_ffa.timer_present ? 'true' : 'false') + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  lines.push('pub lootFfaTimerSeconds(spawnIndex: int): float {');
  for (const [index, spawn] of spawns.entries()) {
    lines.push('    if (spawnIndex == ' + index + ') { return ' +
      zrNumber(spawn.loot_ffa.timer_seconds) + '; }');
  }
  lines.push('    throw "unknown WOC M8 encounter spawn";', '}');
}

function renderSpawnPetRuntime(lines, spawns) {
  const fields = [
    ['petMode', 'mode', '<uint>', (value) => '<uint>' + zrInteger(value)],
    ['petTauntTimerSeconds', 'taunt_timer', '', zrNumber],
    ['petAutoTauntPresent', 'auto_taunt.present', '', (value) => value ? 'true' : 'false'],
    ['petAutoTaunt', 'auto_taunt.value', '', (value) => value ? 'true' : 'false'],
    ['petAutoWaterJetPresent', 'auto_water_jet.present', '', (value) => value ? 'true' : 'false'],
    ['petAutoWaterJet', 'auto_water_jet.value', '', (value) => value ? 'true' : 'false'],
    ['petManualTauntPendingPresent', 'manual_taunt_pending.present', '', (value) => value ? 'true' : 'false'],
    ['petManualTauntPending', 'manual_taunt_pending.value', '', (value) => value ? 'true' : 'false'],
    ['petPathCooldownSeconds', 'path_cooldown', '', zrNumber],
  ];
  for (const [functionName, field, cast, format] of fields) {
    const isBoolean = functionName.includes('Present') || functionName === 'petAutoTaunt' ||
      functionName === 'petAutoWaterJet' || functionName === 'petManualTauntPending';
    const type = isBoolean ? 'bool' : functionName === 'petMode' ? 'uint' : 'float';
    lines.push('pub ' + functionName + '(spawnIndex: int): ' + type + ' {');
    for (const [index, spawn] of spawns.entries()) {
      const [outer, inner] = field.split('.');
      const value = inner ? spawn.pet_runtime[outer][inner] : spawn.pet_runtime[outer];
      lines.push('    if (spawnIndex == ' + index + ') { return ' + cast + format(value) + '; }');
    }
    lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  }
  lines.pop();
}

function renderSpawnBossCadence(lines, spawns) {
  const fields = [
    ['bossPulseTimerSeconds', 'pulse_timer', 'float', zrNumber],
    ['bossStompTimerSeconds', 'stomp_timer', 'float', zrNumber],
    ['bossBigCastTimerSeconds', 'big_cast_timer', 'float', zrNumber],
    ['bossYelledEngage', 'yelled_engage', 'bool', (value) => value ? 'true' : 'false'],
    ['bossStoneskinTimerSeconds', 'stoneskin_timer', 'float', zrNumber],
  ];
  for (const [functionName, field, type, format] of fields) {
    lines.push('pub ' + functionName + '(spawnIndex: int): ' + type + ' {');
    for (const [index, spawn] of spawns.entries()) {
      lines.push('    if (spawnIndex == ' + index + ') { return ' +
        format(spawn.boss_cadence[field]) + '; }');
    }
    lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  }
  lines.pop();
}

function renderSpawnBossSpecial(lines, spawns) {
  const fields = [
    ['bossTerrifyTimerSeconds', 'terrify_timer', 'float', zrNumber],
    ['bossAoeSlowTimerSeconds', 'aoe_slow_timer', 'float', zrNumber],
    ['bossLoudYellTimerSeconds', 'loud_yell_timer', 'float', zrNumber],
    ['bossLoudYellIndex', 'loud_yell_index', 'int', zrInteger],
    ['bossDetonateTimerPresent', 'detonate_timer.present', 'bool', (value) => value ? 'true' : 'false'],
    ['bossDetonateTimerSeconds', 'detonate_timer.seconds', 'float', zrNumber],
    ['bossMendTimerSeconds', 'mend_timer', 'float', zrNumber],
    ['bossWardTimerSeconds', 'ward_timer', 'float', zrNumber],
    ['bossChannelTimerSeconds', 'channel_timer', 'float', zrNumber],
    ['bossChannelRamp', 'channel_ramp', 'float', zrNumber],
    ['bossRallyTimerSeconds', 'rally_timer', 'float', zrNumber],
    ['bossWarcryTimerSeconds', 'warcry_timer', 'float', zrNumber],
    ['bossFiredSummons', 'fired_summons', 'int', zrInteger],
    ['bossEnraged', 'enraged', 'bool', (value) => value ? 'true' : 'false'],
    ['bossHealedThisPull', 'healed_this_pull', 'bool', (value) => value ? 'true' : 'false'],
  ];
  for (const [functionName, field, type, format] of fields) {
    lines.push('pub ' + functionName + '(spawnIndex: int): ' + type + ' {');
    for (const [index, spawn] of spawns.entries()) {
      const [outer, inner] = field.split('.');
      const value = inner ? spawn.boss_special[outer][inner] : spawn.boss_special[outer];
      lines.push('    if (spawnIndex == ' + index + ') { return ' + format(value) + '; }');
    }
    lines.push('    throw "unknown WOC M8 encounter spawn";', '}', '');
  }
  lines.pop();
}

function renderContractTest(catalog) {
  const first = catalog.spawns[0];
  const firstBoar = catalog.spawns.find((spawn) => spawn.mob_id === 'wild_boar');
  const last = catalog.spawns.at(-1);
  return [
    'pub contractTest(): int {',
    `    if (catalogSha() != ${zrString(catalog.catalog_sha256)} || campCount() != 4 || spawnCount() != 24) { return -1; }`,
    `    if (campMobId(0) != "forest_wolf" || campMemberCount(0) != 7 || campSourceIndex(0) != 0 ||`,
    `        campMobId(3) != "wild_boar" || campMemberCount(3) != 5 || campSourceIndex(3) != 4) { return -2; }`,
    `    if (sourceEntityId(0) != ${first.source_entity_id} || mobId(0) != "forest_wolf" || level(0) != ${first.level} ||`,
    `        x(0) != ${zrNumber(first.x)} || y(0) != ${zrNumber(first.y)} || z(0) != ${zrNumber(first.z)} ||`,
    `        maxHp(0) != ${first.max_hp} || moveSpeed(0) != ${zrNumber(first.move_speed)} ||`,
    `        resourceKind(0) != <uint>${zrInteger(first.resource_kind)} ||`,
    `        facing(0) != ${zrNumber(first.facing)} || wanderTimer(0) != ${zrNumber(first.wander_timer)} ||`,
    `        combatInteger(0, "weaponMin") != ${zrInteger(first.combat.weapon_min)} ||`,
    `        combatInteger(0, "armor") != ${zrInteger(first.combat.armor)} ||`,
    `        combatDecimal(0, "weaponSpeed") != ${zrNumber(first.combat.weapon_speed)} ||`,
    `        combatFlag(0, "dualWielding") || combatStateFlag(0, "inCombat") ||`,
    `        combatStateDecimal(0, "combatTimer") != ${zrNumber(first.combat_state.combat_timer)} ||`,
    `        combatStateTargetId(0) != <uint>${zrInteger(first.combat_state.aggro_target_id)} ||`,
    `        locomotionRecoveryFlag(0, "leashAnchorPresent") ||`,
    `        locomotionRecoveryFlag(0, "hasFled") ||`,
    `        locomotionRecoveryDecimal(0, "fleeTimer") != ${zrNumber(first.locomotion_recovery.flee_timer)} ||`,
    `        forcedTargetId(0) != <uint>${zrInteger(first.forced_target.forced_target_id)} ||`,
    `        forcedTargetDecimal(0, "forcedTargetTimer") != ${zrNumber(first.forced_target.forced_target_timer)} ||`,
    `        resourceCooldownInteger(0, "comboPoints") != ${zrInteger(first.resource_cooldown.combo_points)} ||`,
    `        resourceCooldownDecimal(0, "fiveSecondRule") != ${zrNumber(first.resource_cooldown.five_second_rule)} ||`,
    `        castChargeTargetFlag(0, "castAimPresent") ||`,
    `        castChargeTargetDecimal(0, "chargeTimeLeft") != ${zrNumber(first.cast_charge_target.charge_time_left)} ||`,
    `        castChargeTargetId(0, "chargeTargetId") != <uint>${zrInteger(first.cast_charge_target.charge_target_id)}) { return -3; }`,
    `    if (sourceEntityId(13) != ${firstBoar.source_entity_id} || mobId(13) != "wild_boar" || level(13) != ${firstBoar.level} ||`,
    `        maxHp(13) != ${firstBoar.max_hp} || moveSpeed(13) != ${zrNumber(firstBoar.move_speed)} ||`,
    `        combatInteger(13, "weaponMax") != ${zrInteger(firstBoar.combat.weapon_max)} ||`,
    `        combatDecimal(13, "critChance") != ${zrNumber(firstBoar.combat.crit_chance)}) { return -4; }`,
    `    if (sourceEntityId(23) != ${last.source_entity_id} || sourceCampIndex(23) != ${last.source_camp_index} ||`,
    `        campMemberIndex(23) != ${last.camp_member_index} || mobId(23) != "wild_boar") { return -5; }`,
    '    var index = 0;',
    '    while (index < spawnCount()) {',
    '        if (level(index) < 1 || maxHp(index) < 1 || moveSpeed(index) <= 0.0 ||',
    '            campMemberIndex(index) < 0 || sourceCampIndex(index) < 0) { return -6; }',
    '        index = index + 1;',
    '    }',
    '    return 1;',
    '}',
  ];
}

function catalogHash(catalog) {
  return hashText(JSON.stringify({
    extraction: catalog.extraction,
    camps: catalog.camps,
    spawns: catalog.spawns,
  }));
}

function resourceKindCode(resourceType) {
  if (resourceType === null) return 0;
  if (resourceType === 'mana') return 1;
  if (resourceType === 'rage') return 2;
  if (resourceType === 'energy') return 3;
  throw new Error(`unknown WOC resource type: ${resourceType}`);
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function textIdentity(path, text) {
  return { path, bytes: Buffer.byteLength(text, 'utf8'), sha256: hashText(text) };
}

function writeOrCheck(path, content) {
  if (checkOnly) {
    invariant(existsSync(path), `${path} is missing; run npm run generate:m8-eastbrook-encounter`);
    invariant(readFileSync(path, 'utf8') === content, `${path} is stale; run npm run generate:m8-eastbrook-encounter`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, 'utf8');
}

function zrString(value) {
  return JSON.stringify(value);
}

function zrInteger(value) {
  invariant(Number.isInteger(value), `non-integer Zr value ${value}`);
  return String(value);
}

function zrNumber(value) {
  invariant(Number.isFinite(value), `non-finite Zr number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function hashText(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
