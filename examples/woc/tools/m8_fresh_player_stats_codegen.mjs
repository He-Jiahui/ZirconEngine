import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_CLASS_IDS = [
  'warrior', 'mage', 'rogue', 'paladin', 'hunter', 'priest', 'shaman', 'warlock', 'druid',
];
const EXPECTED_FRESH_VALUES = {
  warrior: { max_hp: 90, max_resource: 100, resource: 0, armor: 110, attack_power: 46 },
  mage: { max_hp: 54, max_resource: 195, resource: 195, armor: 57, spell_power: 13 },
  hunter: { max_hp: 69, max_resource: 93, resource: 93, armor: 109, ranged_power: 50 },
};
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const bootstrapCatalogPath = join(projectRoot, 'contracts', 'm8_offline_bootstrap_content.json');
const outputPath = join(projectRoot, 'contracts', 'm8_fresh_player_stats.json');
const zrOutputPath = join(
  projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm8_fresh_player_stats.zr',
);
const extractorPath = join(scriptDirectory, 'm8_fresh_player_stats_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  invariant(sourceManifest.source_commit === SOURCE_COMMIT, 'reference source commit drifted');
  const bootstrapCatalog = JSON.parse(readFileSync(bootstrapCatalogPath, 'utf8'));
  invariant(bootstrapCatalog.source_commit === SOURCE_COMMIT, 'bootstrap catalog source commit drifted');
  const child = spawnSync(process.execPath, [
    '--no-warnings', '--experimental-loader', loaderUrl, extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `fresh player extractor exited ${child.status}`);
  const extracted = JSON.parse(child.stdout);
  validateExtracted(extracted, bootstrapCatalog);
  const catalog = {
    schema_version: 17,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m8_fresh_player_stats_codegen.mjs',
    source_identities: sourceIdentities(),
    players: extracted.players,
  };
  catalog.catalog_sha256 = catalogHash(catalog);
  const json = `${JSON.stringify(catalog, null, 2)}\n`;
  writeOrCheck(outputPath, json);
  writeOrCheck(zrOutputPath, renderZr(catalog));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} M8 fresh player stats: ` +
    `${catalog.players.length} classes (${catalog.catalog_sha256.slice(0, 15)})\n`,
  );
}

function validateExtracted(extracted, bootstrapCatalog) {
  invariant(extracted && typeof extracted === 'object', 'fresh player extraction is not an object');
  invariant(Array.isArray(extracted.players), 'fresh player list is missing');
  invariant(JSON.stringify(extracted.players.map((player) => player.class_id)) ===
    JSON.stringify(EXPECTED_CLASS_IDS), 'fresh player class order drifted');
  invariant(Array.isArray(bootstrapCatalog.classes), 'bootstrap classes are missing');
  for (const [index, player] of extracted.players.entries()) {
    const bootstrap = bootstrapCatalog.classes[index];
    invariant(bootstrap?.id === player.class_id, `bootstrap class mismatch: ${player.class_id}`);
    invariant(player.level === 1, `fresh player level is not one: ${player.class_id}`);
    invariant(player.resource_type === bootstrap.resource_type,
      `resource type mismatch: ${player.class_id}`);
    invariant(player.color === bootstrap.color, `color mismatch: ${player.class_id}`);
    invariant(player.equipment?.mainhand === bootstrap.start_weapon &&
      player.equipment?.chest === bootstrap.start_chest,
    `equipment mismatch: ${player.class_id}`);
    invariant(JSON.stringify(player.start_items) === JSON.stringify(bootstrap.start_items),
      `starter ration mismatch: ${player.class_id}`);
    invariant(player.mainhand_item_id === player.equipment.mainhand &&
      player.equipped_items?.mainhand === player.equipment.mainhand &&
      player.equipped_items?.chest === player.equipment.chest,
    `equipped entity mirror mismatch: ${player.class_id}`);
    invariant(player.hp === player.max_hp && player.max_hp > 0,
      `fresh hp is not full: ${player.class_id}`);
    const expectedResource = player.resource_type === 'mana'
      ? player.max_resource
      : player.resource_type === 'energy' ? 100 : 0;
    invariant(player.resource === expectedResource && player.max_resource > 0,
      `fresh resource mismatch: ${player.class_id}`);
    for (const value of Object.values(player.stats ?? {})) {
      invariant(Number.isFinite(value), `non-finite stat: ${player.class_id}`);
    }
    for (const value of [
      player.weapon?.min, player.weapon?.max, player.weapon?.speed, player.attack_power,
      player.ranged_power, player.spell_power, player.crit_chance, player.dodge_chance,
      player.move_speed,
    ]) {
      invariant(Number.isFinite(value), `non-finite derived value: ${player.class_id}`);
    }
    invariant(player.weapon.min > 0 && player.weapon.max >= player.weapon.min &&
      player.weapon.speed > 0 && player.move_speed > 0,
    `invalid fresh weapon or movement: ${player.class_id}`);
    invariant(player.combat && typeof player.combat === 'object',
      `fresh combat projection is missing: ${player.class_id}`);
    for (const field of [
      'weapon_min', 'weapon_max', 'weapon_speed', 'offhand_weapon_min', 'offhand_weapon_max',
      'offhand_weapon_speed', 'attack_power', 'ranged_power', 'spell_power', 'armor',
      'crit_chance', 'dodge_chance', 'hit_bonus', 'crit_damage_physical_bonus', 'melee_haste',
      'ranged_haste', 'swing_timer', 'offhand_swing_timer', 'block_chance', 'block_value',
    ]) {
      invariant(Number.isFinite(player.combat[field]),
        `non-finite fresh combat value ${field}: ${player.class_id}`);
    }
    invariant(typeof player.combat.has_offhand_weapon === 'boolean' &&
      typeof player.combat.dual_wielding === 'boolean',
    `invalid fresh combat flags: ${player.class_id}`);
    invariant(player.combat.weapon_min === player.weapon.min &&
      player.combat.weapon_max === player.weapon.max &&
      player.combat.weapon_speed === player.weapon.speed &&
      player.combat.attack_power === player.attack_power &&
      player.combat.ranged_power === player.ranged_power &&
      player.combat.spell_power === player.spell_power &&
      player.combat.armor === player.stats.armor &&
      player.combat.crit_chance === player.crit_chance &&
      player.combat.dodge_chance === player.dodge_chance,
    `fresh combat mirror mismatch: ${player.class_id}`);
    invariant(player.combat_state && typeof player.combat_state === 'object',
      `fresh combat state is missing: ${player.class_id}`);
    invariant(typeof player.combat_state.in_combat === 'boolean' &&
      Number.isFinite(player.combat_state.combat_timer) &&
      Number.isSafeInteger(player.combat_state.aggro_target_id) &&
      player.combat_state.combat_timer >= 0 && player.combat_state.aggro_target_id >= 0,
    `invalid fresh combat state: ${player.class_id}`);
    invariant(player.combat_state.in_combat === false && player.combat_state.combat_timer === 99 &&
      player.combat_state.aggro_target_id === 0,
    `fresh combat state initializer drifted: ${player.class_id}`);
    invariant(player.locomotion_recovery && typeof player.locomotion_recovery === 'object',
      `fresh locomotion recovery is missing: ${player.class_id}`);
    const recovery = player.locomotion_recovery;
    invariant(typeof recovery.leash_anchor_present === 'boolean' &&
      typeof recovery.has_fled === 'boolean' &&
      [recovery.leash_anchor_x, recovery.leash_anchor_y, recovery.leash_anchor_z,
        recovery.evade_stall, recovery.flee_timer, recovery.flee_return_timer].every(Number.isFinite),
    `invalid fresh locomotion recovery: ${player.class_id}`);
    invariant(recovery.leash_anchor_present === false && recovery.leash_anchor_x === 0 &&
      recovery.leash_anchor_y === 0 && recovery.leash_anchor_z === 0 &&
      recovery.evade_stall === 0 && recovery.flee_timer === 0 &&
      recovery.flee_return_timer === 0 && recovery.has_fled === false,
    `fresh locomotion recovery initializer drifted: ${player.class_id}`);
    invariant(player.forced_target && typeof player.forced_target === 'object',
      `fresh forced target is missing: ${player.class_id}`);
    const forcedTarget = player.forced_target;
    invariant(Number.isSafeInteger(forcedTarget.forced_target_id) &&
      Number.isFinite(forcedTarget.forced_target_timer) &&
      Number.isFinite(forcedTarget.shuffle_target_timer) && forcedTarget.forced_target_id >= 0 &&
      forcedTarget.forced_target_timer >= 0 && forcedTarget.shuffle_target_timer >= 0,
    `invalid fresh forced target: ${player.class_id}`);
    invariant(forcedTarget.forced_target_id === 0 && forcedTarget.forced_target_timer === 0 &&
      forcedTarget.shuffle_target_timer === 0,
    `fresh forced-target initializer drifted: ${player.class_id}`);
    invariant(player.resource_cooldown && typeof player.resource_cooldown === 'object',
      `fresh resource cooldown is missing: ${player.class_id}`);
    const resourceCooldown = player.resource_cooldown;
    invariant(Number.isSafeInteger(resourceCooldown.combo_points) &&
      Number.isSafeInteger(resourceCooldown.saved_mana) &&
      [resourceCooldown.five_second_rule, resourceCooldown.combo_until,
        resourceCooldown.overpower_until, resourceCooldown.potion_cooldown_until,
        resourceCooldown.potion_cd_remaining].every(Number.isFinite),
    `invalid fresh resource cooldown: ${player.class_id}`);
    invariant(resourceCooldown.five_second_rule === 99 && resourceCooldown.combo_points === 0 &&
      resourceCooldown.combo_until === -1 && resourceCooldown.overpower_until === -1 &&
      resourceCooldown.potion_cooldown_until === -1 && resourceCooldown.potion_cd_remaining === 0 &&
      resourceCooldown.saved_mana === 0,
    `fresh resource-cooldown initializer drifted: ${player.class_id}`);
    invariant(player.cast_charge_target && typeof player.cast_charge_target === 'object',
      `fresh cast-charge target is missing: ${player.class_id}`);
    const castChargeTarget = player.cast_charge_target;
    invariant(typeof castChargeTarget.cast_aim_present === 'boolean' &&
      typeof castChargeTarget.queued_cast_aim_present === 'boolean' &&
      Number.isSafeInteger(castChargeTarget.charge_target_id) &&
      Number.isSafeInteger(castChargeTarget.follow_target_id) &&
      [castChargeTarget.cast_aim_x, castChargeTarget.cast_aim_y, castChargeTarget.cast_aim_z,
        castChargeTarget.queued_cast_aim_x, castChargeTarget.queued_cast_aim_y,
        castChargeTarget.queued_cast_aim_z, castChargeTarget.charge_time_left].every(Number.isFinite) &&
      castChargeTarget.charge_target_id >= 0 && castChargeTarget.follow_target_id >= 0 &&
      castChargeTarget.charge_time_left >= 0,
    `invalid fresh cast-charge target: ${player.class_id}`);
    invariant(castChargeTarget.cast_aim_present === false && castChargeTarget.cast_aim_x === 0 &&
      castChargeTarget.cast_aim_y === 0 && castChargeTarget.cast_aim_z === 0 &&
      castChargeTarget.queued_cast_aim_present === false &&
      castChargeTarget.queued_cast_aim_x === 0 && castChargeTarget.queued_cast_aim_y === 0 &&
      castChargeTarget.queued_cast_aim_z === 0 && castChargeTarget.charge_target_id === 0 &&
      castChargeTarget.charge_time_left === 0 && castChargeTarget.follow_target_id === 0,
    `fresh cast-charge target initializer drifted: ${player.class_id}`);
    invariant(Number.isSafeInteger(player.resource_kind) && player.resource_kind >= 0 &&
      player.resource_kind <= 3 && player.resource_kind === resourceKindCode(player.resource_type),
    `fresh resource kind drifted: ${player.class_id}`);
  }
  for (const player of extracted.players) {
    invariant(player.presentation_identity && typeof player.presentation_identity === 'object',
      'fresh presentation identity is missing: ' + player.class_id);
    const presentation = player.presentation_identity;
    invariant(Number.isFinite(presentation.scale) && presentation.scale > 0 &&
      Number.isSafeInteger(presentation.color) && presentation.color >= 0 &&
      presentation.color <= 0xffffff &&
      (presentation.skin_catalog === 1 || presentation.skin_catalog === 2) &&
      Number.isSafeInteger(presentation.skin_index) && presentation.skin_index >= 0,
    'invalid fresh presentation identity: ' + player.class_id);
    invariant(presentation.scale === 1 && presentation.color === player.color &&
      presentation.skin_catalog === 1 && presentation.skin_index === 0,
    'fresh presentation identity initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    invariant(player.activity_state && typeof player.activity_state === 'object',
      'fresh activity state is missing: ' + player.class_id);
    const activity = player.activity_state;
    invariant(Number.isSafeInteger(activity.ai_state) && activity.ai_state >= 1 &&
      activity.ai_state <= 6 && typeof activity.sitting === 'boolean' &&
      typeof activity.weapon_stowed === 'boolean',
    'invalid fresh activity state: ' + player.class_id);
    invariant(activity.ai_state === 1 && activity.sitting === false &&
      activity.weapon_stowed === false,
    'fresh activity-state initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    invariant(player.tap_ownership && typeof player.tap_ownership === 'object' &&
      Number.isSafeInteger(player.tap_ownership.tapped_by_id) &&
      player.tap_ownership.tapped_by_id >= 0,
    'invalid fresh tap ownership: ' + player.class_id);
    invariant(player.tap_ownership.tapped_by_id === 0,
      'fresh tap-ownership initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    invariant(player.corpse_instance && typeof player.corpse_instance === 'object' &&
      Number.isSafeInteger(player.corpse_instance.instance_id) &&
      player.corpse_instance.instance_id >= 0,
    'invalid fresh corpse instance: ' + player.class_id);
    invariant(player.corpse_instance.instance_id === 0,
      'fresh corpse-instance initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    invariant(player.harvest_claim && typeof player.harvest_claim === 'object' &&
      Number.isSafeInteger(player.harvest_claim.claimed_by_id) &&
      player.harvest_claim.claimed_by_id >= 0,
    'invalid fresh harvest claim: ' + player.class_id);
    invariant(player.harvest_claim.claimed_by_id === 0,
      'fresh harvest-claim initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    invariant(player.loot_ffa && typeof player.loot_ffa === 'object' &&
      typeof player.loot_ffa.timer_present === 'boolean' &&
      Number.isFinite(player.loot_ffa.timer_seconds) && player.loot_ffa.timer_seconds >= 0,
    'invalid fresh loot FFA state: ' + player.class_id);
    invariant(player.loot_ffa.timer_present === false && player.loot_ffa.timer_seconds === 0,
      'fresh loot-FFA initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    const pet = player.pet_runtime;
    invariant(pet && typeof pet === 'object' && Number.isSafeInteger(pet.mode) &&
      pet.mode >= 1 && pet.mode <= 3 && Number.isFinite(pet.taunt_timer) &&
      pet.taunt_timer >= 0 && Number.isFinite(pet.path_cooldown) && pet.path_cooldown >= 0,
    'invalid fresh pet runtime state: ' + player.class_id);
    for (const field of ['auto_taunt', 'auto_water_jet', 'manual_taunt_pending']) {
      invariant(typeof pet[field]?.present === 'boolean' && typeof pet[field]?.value === 'boolean' &&
        (pet[field].present || pet[field].value === false),
      'invalid fresh pet optional state: ' + player.class_id + ' ' + field);
    }
    invariant(pet.mode === 2 && pet.taunt_timer === 0 && pet.path_cooldown === 0 &&
      pet.auto_taunt.present === false && pet.auto_taunt.value === false &&
      pet.auto_water_jet.present === false && pet.auto_water_jet.value === false &&
      pet.manual_taunt_pending.present === false && pet.manual_taunt_pending.value === false,
    'fresh pet runtime initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    const cadence = player.boss_cadence;
    invariant(cadence && typeof cadence === 'object' &&
      [cadence.pulse_timer, cadence.stomp_timer, cadence.big_cast_timer,
        cadence.stoneskin_timer].every((value) => Number.isFinite(value) && value >= 0) &&
      typeof cadence.yelled_engage === 'boolean',
    'invalid fresh boss cadence state: ' + player.class_id);
    invariant(cadence.pulse_timer === 0 && cadence.stomp_timer === 0 &&
      cadence.big_cast_timer === 0 && cadence.yelled_engage === false &&
      cadence.stoneskin_timer === 0,
    'fresh boss cadence initializer drifted: ' + player.class_id);
  }
  for (const player of extracted.players) {
    const special = player.boss_special;
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
    'invalid fresh boss special state: ' + player.class_id);
    invariant(special.terrify_timer === 0 && special.aoe_slow_timer === 0 &&
      special.loud_yell_timer === 0 && special.loud_yell_index === 0 &&
      special.detonate_timer.present === false && special.detonate_timer.seconds === 0 &&
      special.mend_timer === 0 && special.ward_timer === 0 && special.channel_timer === 0 &&
      special.channel_ramp === 0 && special.rally_timer === 0 && special.warcry_timer === 0 &&
      special.fired_summons === 0 && special.enraged === false && special.healed_this_pull === false,
    'fresh boss special initializer drifted: ' + player.class_id);
  }
  for (const [classId, expected] of Object.entries(EXPECTED_FRESH_VALUES)) {
    const player = extracted.players.find((entry) => entry.class_id === classId);
    invariant(player, `expected fresh player is missing: ${classId}`);
    for (const [field, value] of Object.entries(expected)) {
      const actual = field === 'armor' ? player.stats.armor : player[field];
      invariant(actual === value, `fresh ${classId} ${field} drifted: ${actual}`);
    }
  }
}

function sourceIdentities() {
  const paths = [
    'src/sim/entity.ts',
    'src/sim/sim.ts',
    'src/sim/types.ts',
    'src/sim/data.ts',
    'src/sim/item_level_req.ts',
    'src/sim/pvp/index.ts',
    'src/sim/content/classes.ts',
    'src/sim/content/items.ts',
    'src/sim/content/weapon_skin_rules.ts',
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
    '// Generated by examples/woc/tools/m8_fresh_player_stats_codegen.mjs.',
    `// Source ${catalog.source_commit}; fresh Sim.addPlayer derived facts only; do not edit.`,
    '',
    'pub catalogSha(): string {',
    `    return ${zrString(catalog.catalog_sha256)};`,
    '}',
    '',
    'pub classCount(): int {',
    `    return ${catalog.players.length};`,
    '}',
    '',
    'pub classId(index: int): string {',
  ];
  catalog.players.forEach((player, index) => lines.push(
    `    if (index == ${index}) { return ${zrString(player.class_id)}; }`,
  ));
  lines.push('    throw "unknown WOC fresh player class index";', '}', '',
    'pub classIndex(id: string): int {');
  catalog.players.forEach((player, index) => lines.push(
    `    if (id == ${zrString(player.class_id)}) { return ${index}; }`,
  ));
  lines.push('    return -1;', '}', '');
  lines.push('// Class ids are join keys into m8_offline_bootstrap_content; raw class facts live there.', '');
  renderClassInteger(lines, catalog.players);
  lines.push('');
  renderClassDecimal(lines, catalog.players);
  lines.push('');
  renderCombatInteger(lines, catalog.players);
  lines.push('');
  renderCombatDecimal(lines, catalog.players);
  lines.push('');
  renderCombatFlag(lines, catalog.players);
  lines.push('');
  renderCombatStateFlag(lines, catalog.players);
  lines.push('');
  renderCombatStateDecimal(lines, catalog.players);
  lines.push('');
  renderCombatStateTargetId(lines, catalog.players);
  lines.push('');
  renderLocomotionRecoveryFlag(lines, catalog.players);
  lines.push('');
  renderLocomotionRecoveryDecimal(lines, catalog.players);
  lines.push('');
  renderForcedTargetId(lines, catalog.players);
  lines.push('');
  renderForcedTargetDecimal(lines, catalog.players);
  lines.push('');
  renderResourceCooldownInteger(lines, catalog.players);
  lines.push('');
  renderResourceCooldownDecimal(lines, catalog.players);
  lines.push('');
  renderCastChargeTargetFlag(lines, catalog.players);
  lines.push('');
  renderCastChargeTargetDecimal(lines, catalog.players);
  lines.push('');
  renderCastChargeTargetId(lines, catalog.players);
  lines.push('');
  renderResourceKind(lines, catalog.players);
  lines.push('');
  renderPresentationIdentity(lines, catalog.players);
  lines.push('');
  renderActivityState(lines, catalog.players);
  lines.push('');
  renderTapOwnership(lines, catalog.players);
  lines.push('');
  renderCorpseInstance(lines, catalog.players);
  lines.push('');
  renderHarvestClaim(lines, catalog.players);
  lines.push('');
  renderLootFfa(lines, catalog.players);
  lines.push('');
  renderPetRuntime(lines, catalog.players);
  lines.push('');
  renderBossCadence(lines, catalog.players);
  lines.push('');
  renderBossSpecial(lines, catalog.players);
  lines.push('', ...renderContractTest(catalog));
  return `${lines.join('\n')}\n`;
}

function renderClassInteger(lines, players) {
  lines.push('pub classInteger(index: int, field: string): int {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      level: player.level,
      maxHp: player.max_hp,
      hp: player.hp,
      maxResource: player.max_resource,
      resource: player.resource,
      statStr: player.stats.str,
      statAgi: player.stats.agi,
      statSta: player.stats.sta,
      statInt: player.stats.int,
      statSpi: player.stats.spi,
      statArmor: player.stats.armor,
      weaponMin: player.weapon.min,
      weaponMax: player.weapon.max,
      attackPower: player.attack_power,
      rangedPower: player.ranged_power,
      spellPower: player.spell_power,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrInt(value)}; }`);
    }
    lines.push('        return 0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderClassDecimal(lines, players) {
  lines.push('pub classDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      weaponSpeed: player.weapon.speed,
      critChance: player.crit_chance,
      dodgeChance: player.dodge_chance,
      moveSpeed: player.move_speed,
      pvpOffense: player.stats.pvp_offense,
      pvpDefense: player.stats.pvp_defense,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCombatInteger(lines, players) {
  lines.push('pub combatInteger(index: int, field: string): int {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      weaponMin: player.combat.weapon_min,
      weaponMax: player.combat.weapon_max,
      offhandWeaponMin: player.combat.offhand_weapon_min,
      offhandWeaponMax: player.combat.offhand_weapon_max,
      attackPower: player.combat.attack_power,
      rangedPower: player.combat.ranged_power,
      spellPower: player.combat.spell_power,
      armor: player.combat.armor,
      blockValue: player.combat.block_value,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrInt(value)}; }`);
    }
    lines.push('        return 0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCombatDecimal(lines, players) {
  lines.push('pub combatDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      weaponSpeed: player.combat.weapon_speed,
      offhandWeaponSpeed: player.combat.offhand_weapon_speed,
      critChance: player.combat.crit_chance,
      dodgeChance: player.combat.dodge_chance,
      hitBonus: player.combat.hit_bonus,
      critDamagePhysicalBonus: player.combat.crit_damage_physical_bonus,
      meleeHaste: player.combat.melee_haste,
      rangedHaste: player.combat.ranged_haste,
      swingTimer: player.combat.swing_timer,
      offhandSwingTimer: player.combat.offhand_swing_timer,
      blockChance: player.combat.block_chance,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCombatFlag(lines, players) {
  lines.push('pub combatFlag(index: int, field: string): bool {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      hasOffhandWeapon: player.combat.has_offhand_weapon,
      dualWielding: player.combat.dual_wielding,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${value ? 'true' : 'false'}; }`);
    }
    lines.push('        return false;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCombatStateFlag(lines, players) {
  lines.push('pub combatStateFlag(index: int, field: string): bool {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    lines.push(`        if (field == "inCombat") { return ${player.combat_state.in_combat ? 'true' : 'false'}; }`);
    lines.push('        return false;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCombatStateDecimal(lines, players) {
  lines.push('pub combatStateDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    lines.push(`        if (field == "combatTimer") { return ${zrNumber(player.combat_state.combat_timer)}; }`);
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCombatStateTargetId(lines, players) {
  lines.push('pub combatStateTargetId(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) { return <uint>${zrInt(player.combat_state.aggro_target_id)}; }`);
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderLocomotionRecoveryFlag(lines, players) {
  lines.push('pub locomotionRecoveryFlag(index: int, field: string): bool {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    for (const [field, value] of Object.entries({
      leashAnchorPresent: player.locomotion_recovery.leash_anchor_present,
      hasFled: player.locomotion_recovery.has_fled,
    })) {
      lines.push(`        if (field == ${zrString(field)}) { return ${value ? 'true' : 'false'}; }`);
    }
    lines.push('        return false;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderLocomotionRecoveryDecimal(lines, players) {
  lines.push('pub locomotionRecoveryDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      leashAnchorX: player.locomotion_recovery.leash_anchor_x,
      leashAnchorY: player.locomotion_recovery.leash_anchor_y,
      leashAnchorZ: player.locomotion_recovery.leash_anchor_z,
      evadeStall: player.locomotion_recovery.evade_stall,
      fleeTimer: player.locomotion_recovery.flee_timer,
      fleeReturnTimer: player.locomotion_recovery.flee_return_timer,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderForcedTargetId(lines, players) {
  lines.push('pub forcedTargetId(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) { return <uint>${zrInt(player.forced_target.forced_target_id)}; }`);
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderForcedTargetDecimal(lines, players) {
  lines.push('pub forcedTargetDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      forcedTargetTimer: player.forced_target.forced_target_timer,
      shuffleTargetTimer: player.forced_target.shuffle_target_timer,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderResourceCooldownInteger(lines, players) {
  lines.push('pub resourceCooldownInteger(index: int, field: string): int {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      comboPoints: player.resource_cooldown.combo_points,
      savedMana: player.resource_cooldown.saved_mana,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrInt(value)}; }`);
    }
    lines.push('        return 0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderResourceCooldownDecimal(lines, players) {
  lines.push('pub resourceCooldownDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      fiveSecondRule: player.resource_cooldown.five_second_rule,
      comboUntil: player.resource_cooldown.combo_until,
      overpowerUntil: player.resource_cooldown.overpower_until,
      potionCooldownUntil: player.resource_cooldown.potion_cooldown_until,
      potionCooldownRemaining: player.resource_cooldown.potion_cd_remaining,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCastChargeTargetFlag(lines, players) {
  lines.push('pub castChargeTargetFlag(index: int, field: string): bool {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      castAimPresent: player.cast_charge_target.cast_aim_present,
      queuedCastAimPresent: player.cast_charge_target.queued_cast_aim_present,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${value ? 'true' : 'false'}; }`);
    }
    lines.push('        return false;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCastChargeTargetDecimal(lines, players) {
  lines.push('pub castChargeTargetDecimal(index: int, field: string): float {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      castAimX: player.cast_charge_target.cast_aim_x,
      castAimY: player.cast_charge_target.cast_aim_y,
      castAimZ: player.cast_charge_target.cast_aim_z,
      queuedCastAimX: player.cast_charge_target.queued_cast_aim_x,
      queuedCastAimY: player.cast_charge_target.queued_cast_aim_y,
      queuedCastAimZ: player.cast_charge_target.queued_cast_aim_z,
      chargeTimeLeft: player.cast_charge_target.charge_time_left,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCastChargeTargetId(lines, players) {
  lines.push('pub castChargeTargetId(index: int, field: string): uint {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      chargeTargetId: player.cast_charge_target.charge_target_id,
      followTargetId: player.cast_charge_target.follow_target_id,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return <uint>${zrInt(value)}; }`);
    }
    lines.push('        return <uint>0;', '    }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderResourceKind(lines, players) {
  lines.push('pub resourceKind(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push(`    if (index == ${index}) { return <uint>${zrInt(player.resource_kind)}; }`);
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderPresentationIdentity(lines, players) {
  lines.push('pub presentationIdentityScale(index: int): float {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return ' +
      zrNumber(player.presentation_identity.scale) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  lines.push('pub presentationIdentityColor(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.presentation_identity.color) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  lines.push('pub presentationIdentitySkinCatalog(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.presentation_identity.skin_catalog) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  lines.push('pub presentationIdentitySkinIndex(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.presentation_identity.skin_index) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderActivityState(lines, players) {
  lines.push('pub activityStateAiState(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.activity_state.ai_state) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  lines.push('pub activityStateSitting(index: int): bool {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return ' +
      (player.activity_state.sitting ? 'true' : 'false') + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  lines.push('pub activityStateWeaponStowed(index: int): bool {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return ' +
      (player.activity_state.weapon_stowed ? 'true' : 'false') + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderTapOwnership(lines, players) {
  lines.push('pub tapOwnershipId(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.tap_ownership.tapped_by_id) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderCorpseInstance(lines, players) {
  lines.push('pub corpseInstanceId(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.corpse_instance.instance_id) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderHarvestClaim(lines, players) {
  lines.push('pub harvestClaimId(index: int): uint {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return <uint>' +
      zrInt(player.harvest_claim.claimed_by_id) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderLootFfa(lines, players) {
  lines.push('pub lootFfaTimerPresent(index: int): bool {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return ' +
      (player.loot_ffa.timer_present ? 'true' : 'false') + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  lines.push('pub lootFfaTimerSeconds(index: int): float {');
  for (const [index, player] of players.entries()) {
    lines.push('    if (index == ' + index + ') { return ' +
      zrNumber(player.loot_ffa.timer_seconds) + '; }');
  }
  lines.push('    throw "unknown WOC fresh player class index";', '}');
}

function renderPetRuntime(lines, players) {
  const fields = [
    ['petMode', 'mode', '<uint>', (value) => '<uint>' + zrInt(value)],
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
    lines.push('pub ' + functionName + '(index: int): ' + type + ' {');
    for (const [index, player] of players.entries()) {
      const [outer, inner] = field.split('.');
      const value = inner ? player.pet_runtime[outer][inner] : player.pet_runtime[outer];
      lines.push('    if (index == ' + index + ') { return ' + cast + format(value) + '; }');
    }
    lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  }
  lines.pop();
}

function renderBossCadence(lines, players) {
  const fields = [
    ['bossPulseTimerSeconds', 'pulse_timer', 'float', zrNumber],
    ['bossStompTimerSeconds', 'stomp_timer', 'float', zrNumber],
    ['bossBigCastTimerSeconds', 'big_cast_timer', 'float', zrNumber],
    ['bossYelledEngage', 'yelled_engage', 'bool', (value) => value ? 'true' : 'false'],
    ['bossStoneskinTimerSeconds', 'stoneskin_timer', 'float', zrNumber],
  ];
  for (const [functionName, field, type, format] of fields) {
    lines.push('pub ' + functionName + '(index: int): ' + type + ' {');
    for (const [index, player] of players.entries()) {
      lines.push('    if (index == ' + index + ') { return ' +
        format(player.boss_cadence[field]) + '; }');
    }
    lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  }
  lines.pop();
}

function renderBossSpecial(lines, players) {
  const fields = [
    ['bossTerrifyTimerSeconds', 'terrify_timer', 'float', zrNumber],
    ['bossAoeSlowTimerSeconds', 'aoe_slow_timer', 'float', zrNumber],
    ['bossLoudYellTimerSeconds', 'loud_yell_timer', 'float', zrNumber],
    ['bossLoudYellIndex', 'loud_yell_index', 'int', zrInt],
    ['bossDetonateTimerPresent', 'detonate_timer.present', 'bool', (value) => value ? 'true' : 'false'],
    ['bossDetonateTimerSeconds', 'detonate_timer.seconds', 'float', zrNumber],
    ['bossMendTimerSeconds', 'mend_timer', 'float', zrNumber],
    ['bossWardTimerSeconds', 'ward_timer', 'float', zrNumber],
    ['bossChannelTimerSeconds', 'channel_timer', 'float', zrNumber],
    ['bossChannelRamp', 'channel_ramp', 'float', zrNumber],
    ['bossRallyTimerSeconds', 'rally_timer', 'float', zrNumber],
    ['bossWarcryTimerSeconds', 'warcry_timer', 'float', zrNumber],
    ['bossFiredSummons', 'fired_summons', 'int', zrInt],
    ['bossEnraged', 'enraged', 'bool', (value) => value ? 'true' : 'false'],
    ['bossHealedThisPull', 'healed_this_pull', 'bool', (value) => value ? 'true' : 'false'],
  ];
  for (const [functionName, field, type, format] of fields) {
    lines.push('pub ' + functionName + '(index: int): ' + type + ' {');
    for (const [index, player] of players.entries()) {
      const [outer, inner] = field.split('.');
      const value = inner ? player.boss_special[outer][inner] : player.boss_special[outer];
      lines.push('    if (index == ' + index + ') { return ' + format(value) + '; }');
    }
    lines.push('    throw "unknown WOC fresh player class index";', '}', '');
  }
  lines.pop();
}

function renderContractTest(catalog) {
  const warrior = catalog.players.find((player) => player.class_id === 'warrior');
  const mage = catalog.players.find((player) => player.class_id === 'mage');
  const hunter = catalog.players.find((player) => player.class_id === 'hunter');
  return [
    'pub contractTest(): int {',
    `    if (catalogSha() != ${zrString(catalog.catalog_sha256)} || classCount() != 9) { return -1; }`,
    `    if (classIndex("warrior") != 0 || classIndex("druid") != 8 || classIndex("missing") != -1 ||`,
    `        classInteger(0, "maxHp") != ${zrInt(warrior.max_hp)} ||`,
    `        classInteger(0, "statArmor") != ${zrInt(warrior.stats.armor)} ||`,
    `        classInteger(0, "attackPower") != ${zrInt(warrior.attack_power)} ||`,
    `        resourceKind(0) != <uint>${zrInt(warrior.resource_kind)}) { return -2; }`,
    '    var mageIndex = classIndex("mage");',
    `    if (mageIndex != 1 ||`,
    `        classInteger(mageIndex, "maxResource") != ${zrInt(mage.max_resource)} ||`,
    `        classInteger(mageIndex, "spellPower") != ${zrInt(mage.spell_power)} ||`,
    `        classDecimal(mageIndex, "weaponSpeed") != ${zrNumber(mage.weapon.speed)} ||`,
    `        combatInteger(mageIndex, "armor") != ${zrInt(mage.combat.armor)} ||`,
    `        combatDecimal(mageIndex, "meleeHaste") != ${zrNumber(mage.combat.melee_haste)} ||`,
    `        combatFlag(mageIndex, "dualWielding") || combatStateFlag(mageIndex, "inCombat") ||`,
    `        combatStateDecimal(mageIndex, "combatTimer") != ${zrNumber(mage.combat_state.combat_timer)} ||`,
    `        combatStateTargetId(mageIndex) != <uint>${zrInt(mage.combat_state.aggro_target_id)} ||`,
    `        locomotionRecoveryFlag(mageIndex, "leashAnchorPresent") ||`,
    `        locomotionRecoveryFlag(mageIndex, "hasFled") ||`,
    `        locomotionRecoveryDecimal(mageIndex, "fleeTimer") != ${zrNumber(mage.locomotion_recovery.flee_timer)} ||`,
    `        forcedTargetId(mageIndex) != <uint>${zrInt(mage.forced_target.forced_target_id)} ||`,
    `        forcedTargetDecimal(mageIndex, "forcedTargetTimer") != ${zrNumber(mage.forced_target.forced_target_timer)} ||`,
    `        resourceCooldownInteger(mageIndex, "comboPoints") != ${zrInt(mage.resource_cooldown.combo_points)} ||`,
    `        resourceCooldownDecimal(mageIndex, "fiveSecondRule") != ${zrNumber(mage.resource_cooldown.five_second_rule)} ||`,
    `        castChargeTargetFlag(mageIndex, "castAimPresent") ||`,
    `        castChargeTargetDecimal(mageIndex, "chargeTimeLeft") != ${zrNumber(mage.cast_charge_target.charge_time_left)} ||`,
    `        castChargeTargetId(mageIndex, "chargeTargetId") != <uint>${zrInt(mage.cast_charge_target.charge_target_id)}) { return -3; }`,
    `    if (classInteger(4, "rangedPower") != ${zrInt(hunter.ranged_power)} ||`,
    `        classDecimal(4, "moveSpeed") != ${zrNumber(hunter.move_speed)}) { return -4; }`,
    '    var index = 0;',
    '    while (index < classCount()) {',
    '        if (classIndex(classId(index)) != index) { return -5; }',
    '        index = index + 1;',
    '    }',
    '    return 1;',
    '}',
  ];
}

function catalogHash(catalog) {
  return hashText(JSON.stringify({ players: catalog.players }));
}

function resourceKindCode(resourceType) {
  if (resourceType === null) return 0;
  if (resourceType === 'mana') return 1;
  if (resourceType === 'rage') return 2;
  if (resourceType === 'energy') return 3;
  throw new Error(`unknown WOC resource type: ${resourceType}`);
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function textIdentity(path, text) {
  return { path, bytes: Buffer.byteLength(text, 'utf8'), sha256: hashText(text) };
}

function writeOrCheck(path, content) {
  if (checkOnly) {
    invariant(existsSync(path), `${path} is missing; run npm run generate:m8-fresh-player-stats`);
    invariant(readFileSync(path, 'utf8') === content,
      `${path} is stale; run npm run generate:m8-fresh-player-stats`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, 'utf8');
}

function zrString(value) {
  return JSON.stringify(value);
}

function zrInt(value) {
  invariant(Number.isSafeInteger(value), `non-integer Zr value ${value}`);
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
