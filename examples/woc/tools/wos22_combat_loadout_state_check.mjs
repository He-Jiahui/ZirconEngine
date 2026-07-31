import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const entity = gitShow('src/sim/entity.ts');
const autoAttack = gitShow('src/sim/combat/auto_attack.ts');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const fresh = JSON.parse(readFileSync(resolve(wocRoot, 'contracts', 'm8_fresh_player_stats.json'), 'utf8'));
const encounter = JSON.parse(readFileSync(
  resolve(wocRoot, 'reference', 'current-head', 'm8_eastbrook_encounter.json'), 'utf8',
));

for (const needle of [
  'weapon: { min: 1, max: 2, speed: 2 },',
  'offhandWeapon: null,',
  'attackPower: 0,',
  'rangedPower: 0,',
  'spellPower: 0,',
  'meleeHaste: 0,',
  'rangedHaste: 0,',
  'critChance: 0.05,',
  'hitBonus: 0,',
  'critDmgPhysBonus: 0,',
  'dodgeChance: 0.05,',
  'blockChance: 0,',
  'blockValue: 0,',
  'swingTimer: 0,',
  'offhandSwingTimer: 0,',
  'e.weapon = {',
  'e.stats.armor = Math.round(template.armorPerLevel * (level - 1));',
]) {
  invariant(entity.includes(needle), `pinned Entity combat field drifted: ${needle}`);
}

for (const needle of [
  'p.swingTimer = Math.max(0, p.swingTimer - DT);',
  'const ranged = rangedAutoProfile(p, meta.cls);',
  'const weapon = opts.weapon ?? attacker.weapon;',
  '(ctx.effectiveAttackPower(attacker) / 14) * apSwingSpeed',
  'attacker.critDmgPhysBonus',
]) {
  invariant(autoAttack.includes(needle), `pinned auto-attack dependency drifted: ${needle}`);
}

const combatFields = [
  'weapon_min', 'weapon_max', 'weapon_speed', 'offhand_weapon_min', 'offhand_weapon_max',
  'offhand_weapon_speed', 'has_offhand_weapon', 'dual_wielding', 'attack_power', 'ranged_power',
  'spell_power', 'armor', 'crit_chance', 'dodge_chance', 'hit_bonus',
  'crit_damage_physical_bonus', 'melee_haste', 'ranged_haste', 'swing_timer',
  'offhand_swing_timer', 'block_chance', 'block_value',
];
invariant(fresh.schema_version === 17 && fresh.players.length === 9, 'fresh-player combat catalog drifted');
invariant(encounter.schema_version === 17 && encounter.spawns.length === 24,
  'Eastbrook combat catalog drifted');
for (const player of fresh.players) {
  validateCombat(player.combat, `fresh ${player.class_id}`);
  invariant(player.combat.weapon_min === player.weapon.min &&
    player.combat.weapon_max === player.weapon.max &&
    player.combat.weapon_speed === player.weapon.speed &&
    player.combat.attack_power === player.attack_power &&
    player.combat.ranged_power === player.ranged_power &&
    player.combat.spell_power === player.spell_power &&
    player.combat.armor === player.stats.armor,
  `fresh combat mirror drifted: ${player.class_id}`);
}
for (const spawn of encounter.spawns) validateCombat(spawn.combat, `Eastbrook ${spawn.source_entity_id}`);

for (const needle of [
  'writer.u16(<uint>38, 1, 1);',
  'schemaVersion != <uint>22',
  'if (schemaVersion >= <uint>22) {',
  'appendDefaultCombatLoadoutColumns(this);',
  'appendDefaultCombatLoadoutColumns(state);',
  'combatLoadoutStateIsValid(state: WorldState, index: int): bool',
  'applyFreshPlayerCombatLoadout(state, playerIndex, classIndex);',
  'applyEastbrookCombatLoadout(state, entityIndex, spawnIndex);',
  'pub selfTest(): int',
]) {
  invariant(state.includes(needle), `WOS22 combat-loadout projection omitted: ${needle}`);
}
for (const field of [
  'entityWeaponMinimum', 'entityWeaponMaximum', 'entityWeaponSpeed',
  'entityOffhandWeaponMinimum', 'entityOffhandWeaponMaximum', 'entityOffhandWeaponSpeed',
  'entityHasOffhandWeapon', 'entityDualWielding', 'entityAttackPower', 'entityRangedPower',
  'entitySpellPower', 'entityArmor', 'entityCritChance', 'entityDodgeChance', 'entityHitBonus',
  'entityCritDamagePhysicalBonus', 'entityMeleeHaste', 'entityRangedHaste', 'entitySwingTimer',
  'entityOffhandSwingTimer', 'entityBlockChance', 'entityBlockValue',
]) {
  invariant(state.includes(`pub var ${field}:`), `WOS22 state column is missing: ${field}`);
  invariant(state.includes(`this.${field} = new container.Array`),
    `WOS22 column is not initialized: ${field}`);
  invariant((state.match(new RegExp(field, 'g')) ?? []).length >= 5,
    `WOS22 column lacks encode/decode coverage: ${field}`);
}
invariant(main.includes('\\"world_state\\":\\"WOS38\\",'),
  'package stateSchema must expose the current WOS38 snapshot version');

process.stdout.write(`checked WOS22 combat-loadout source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function validateCombat(combat, label) {
  invariant(combat && typeof combat === 'object', `combat row is missing: ${label}`);
  for (const field of combatFields) {
    const value = combat[field];
    invariant(typeof value === 'boolean' || Number.isFinite(value),
      `combat field is invalid (${field}): ${label}`);
  }
  invariant(combat.weapon_min > 0 && combat.weapon_max >= combat.weapon_min &&
    combat.weapon_speed > 0 && combat.armor >= 0,
  `combat weapon profile is invalid: ${label}`);
  if (combat.has_offhand_weapon) {
    invariant(combat.offhand_weapon_min > 0 &&
      combat.offhand_weapon_max >= combat.offhand_weapon_min &&
      combat.offhand_weapon_speed > 0,
    `combat offhand profile is invalid: ${label}`);
  } else {
    invariant(!combat.dual_wielding && combat.offhand_weapon_min === 0 &&
      combat.offhand_weapon_max === 0 && combat.offhand_weapon_speed === 0,
    `combat offhand absence drifted: ${label}`);
  }
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
