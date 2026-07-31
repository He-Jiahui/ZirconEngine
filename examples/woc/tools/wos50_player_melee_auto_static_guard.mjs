import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const state = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);
const autoAttack = gitShow('src/sim/combat/auto_attack.ts');
const questCredit = gitShow('src/sim/quests/quest_credit.ts');
const lootRoll = gitShow('src/sim/loot/loot_roll.ts');
const simTypes = gitShow('src/sim/types.ts');
const formSwing = gitShow('src/sim/combat/form_swing.ts');
const projectileTravel = gitShow('src/sim/projectile_travel.ts');
const classes = gitShow('src/sim/content/classes.ts');

for (const needle of [
  'export function updatePlayerAutoAttack(ctx: SimContext, p: Entity, meta: PlayerMeta): void {',
  'if (p.auras.some((a) => isTravelFormAuraKind(a.kind))) {',
  'p.swingTimer = Math.max(0, p.swingTimer - DT);',
  'p.offhandSwingTimer = Math.max(0, p.offhandSwingTimer - DT);',
  'if (!p.autoAttack || p.castingAbility) return;',
  'if (p.swingTimer > 0 && (!p.dualWielding || !p.offhandWeapon || p.offhandSwingTimer > 0)) return;',
  'if (facingDiff > MELEE_ARC) return;',
  'if (d > MELEE_RANGE) return;',
  'const connected = meleeSwing(ctx, p, t, bonus, abilityName, {',
  'whiteDualWieldPenalty: p.dualWielding && abilityName === null,',
  'if (p.dualWielding && p.offhandWeapon && p.offhandSwingTimer <= 0) {',
  'weaponMult: 0.5,',
  'apSwingSpeed: offhand.speed,',
  'const spikes = MOBS[target.templateId]?.thorns;',
  'if (spikes && !attacker.dead) {',
  'p.swingTimer =',
]) {
  invariant(autoAttack.includes(needle), `source player melee auto drifted: ${needle}`);
}

for (const needle of [
  'export function onMobKilledForQuests(ctx: SimContext, mob: Entity, meta: PlayerMeta): void {',
  "objective.type === 'kill' && objective.targetMobId === mob.templateId",
]) {
  invariant(questCredit.includes(needle), `source quest kill credit drifted: ${needle}`);
}

for (const needle of [
  'export function rollLoot(',
  'const questRecipients = candidates.filter((m) => needsQuestDrop(ctx, entry, m));',
  'if (questRecipients.length === 0) continue;',
  'if (!ctx.rng.chance(entry.chance)) continue;',
  'copper += ctx.rng.int(Math.ceil(entry.copper * 0.6), Math.ceil(entry.copper * 1.4));',
  'mob.loot = { copper, items };',
]) {
  invariant(lootRoll.includes(needle), `source corpse-loot order drifted: ${needle}`);
}

for (const needle of [
  'export function zeroDiff(playerLevel: number): number {',
  'if (playerLevel <= 7) return 5;',
  'if (playerLevel <= 9) return 6;',
  'if (playerLevel <= 15) return 7;',
  'export function mobXpValue(mobLevel: number, playerLevel: number): number {',
  'const base = 45 + 5 * mobLevel;',
  'return Math.round(base * (1 + 0.05 * Math.min(diff, 4)));',
  'if (-diff >= zd) return 0; // gray',
  'return Math.round(base * (1 - -diff / zd));',
]) {
  invariant(simTypes.includes(needle), `source mob XP curve drifted: ${needle}`);
}

for (const needle of [
  'export const ROGUE_BASE_SWING_SPEED',
  'const ranged = CLASSES[cls].ranged;',
  'if (!ranged) return undefined;',
]) {
  invariant(formSwing.includes(needle), `source rogue melee fallback drifted: ${needle}`);
}

for (const needle of [
  'export const PROJECTILE_SPEED = 26;',
  'export const PROJECTILE_REACH = 0.7;',
  'export const PROJECTILE_MAX_FLIGHT = 3;',
  'export function scheduleProjectile(',
  'for (const proj of ctx.pendingProjectiles) {',
  'if (!source || source.dead || !target || target.dead) continue; // fizzle',
  'const next = stepProjectile(proj.x, proj.z, target.pos.x, target.pos.z, step);',
  'proj.resolve(source, target);',
]) {
  invariant(projectileTravel.includes(needle), `source projectile travel drifted: ${needle}`);
}

for (const needle of [
  "hunter: {",
  'ranged: { min: 5, max: 9, speed: 2.3, maxRange: 35, minRange: 8 },',
]) {
  invariant(classes.includes(needle), `source hunter Auto Shot profile drifted: ${needle}`);
}

for (const needle of [
  'var autoAttackState = %import("combat/auto_attack_state");',
  'var projectileTravel = %import("world/projectile_travel_state");',
  'var m5CampMobLoot = %import("generated/m5_camp_mob_loot");',
  'pub var entityCorpseQuestItemCodes: container.Array<uint>;',
  'pub var entityCorpseQuestItemCounts: container.Array<uint>;',
  'pub var entityCorpseCopper: container.Array<int>;',
  'pub var entityCorpseSharedItemCodes: container.Array<uint>;',
  'pub var entityCorpseSharedItemCounts: container.Array<uint>;',
  'pub var offlineProjectileSourceIds: container.Array<uint>;',
  'pub var offlineProjectileTargetIds: container.Array<uint>;',
  'pub var offlineProjectileX: container.Array<float>;',
  'pub var offlineProjectileZ: container.Array<float>;',
  'pub var offlineProjectileTtls: container.Array<float>;',
  'pub var offlineProjectileWands: container.Array<bool>;',
  'pub var offlineProjectileMinimums: container.Array<float>;',
  'pub var offlineProjectileMaximums: container.Array<float>;',
  'pub var offlineProjectileSpeeds: container.Array<float>;',
  'offlineProjectileStateIsValid(state: WorldState): bool',
  'stepOfflineEastbrookProjectiles(state: WorldState): void',
  'stepOfflineHunterAutoShot(state: WorldState): void',
  'configureOfflineHunterAutoShot(actor: autoAttackState.AutoActor): void',
  'actor.rangedMinimumRange = 8.0;',
  'actor.rangedMaximumRange = 35.0;',
  'actor.rangedMinimum = 5.0;',
  'actor.rangedMaximum = 9.0;',
  'actor.rangedSpeed = 2.3;',
  'projectileTravel.advanceProjectile(',
  'autoAttackState.landNextProjectile(actor, target, events);',
  'stepOfflineEastbrookProjectiles(state);',
  'stepOfflineHunterAutoShot(state);',
  'pub offlineHunterAutoShotStateTest(): int',
  'state.offlinePlayerClass = <uint>4;',
  'corpseSharedLootSlotCount(): int',
  'clearCorpseSharedLoot(state: WorldState, index: int): void',
  'rollOfflineCorpseCopper(state: WorldState, copper: int): int',
  'hasOfflineCorpseLoot(state: WorldState, mobIndex: int): bool',
  'offlineMobXpZeroDiff(playerLevel: int): int',
  'offlineMobXpValue(mobLevel: int, playerLevel: int): uint',
  'offlineMobXpRoundPositive(numerator: int, denominator: int): uint',
  'return <uint>((numerator * 2 + denominator) / (denominator * 2));',
  'var base = 45 + 5 * mobLevel;',
  'if (aboveBonus > 4) {',
  'if (belowDifference >= zeroDifference) {',
  'base * (zeroDifference - belowDifference), zeroDifference',
  'grantOfflineMobKillExperience(',
  'grantOfflineQuestExperience(state, actorIndex, experience);',
  'rollOfflineEastbrookCorpseLoot(state: WorldState, mobIndex: int, playerIndex: int): void',
  'm5CampMobLoot.lootEntryCount(',
  'm5CampMobLoot.entryMetric(',
  'm5CampMobLoot.entryText(',
  'applyOfflineCorpseLootCommand(',
  'payloads.lootCommandId(true)',
  'm5InventoryCanAddItem(state, actorIndex, itemCode, 1)',
  'grantM5InventoryItem(state, actorIndex, itemCode, 1);',
  'state.entityInventoryCopper[actorIndex] =',
  'state.entityCorpseCopper[targetIndex] = 0;',
  'while (sharedSlot < corpseSharedLootSlotCount()) {',
  'state.entityCorpseQuestItemCounts[targetIndex] = <uint>0;',
  'offlineDirectMeleeAutoProfileEnabled(state: WorldState, playerIndex: int): bool',
  '!forms.isTravelFormAbilityCode(',
  'classId == "warrior" || classId == "rogue" || classId == "paladin" ||',
  'classId == "shaman") {',
  'classId == "druid" && (',
  'forms.formAbilityCode("form_bear")',
  'forms.formAbilityCode("form_cat")',
  'stepOfflineDirectMeleeAutoAttack(state: WorldState): void',
  'var classId = knownAbilityCatalog.classId(<int>state.offlinePlayerClass);',
  'if (!offlineDirectMeleeAutoProfileEnabled(state, playerIndex)) {',
  'var actor = new autoAttackState.AutoActor();',
  'actor.offhandWeaponMinimum = <float>state.entityOffhandWeaponMinimum[playerIndex];',
  'actor.offhandWeaponMaximum = <float>state.entityOffhandWeaponMaximum[playerIndex];',
  'actor.offhandWeaponSpeed = <float>state.entityOffhandWeaponSpeed[playerIndex];',
  'actor.hasOffhandWeapon = <bool>state.entityHasOffhandWeapon[playerIndex];',
  'actor.dualWielding = <bool>state.entityDualWielding[playerIndex];',
  'actor.offhandSwingTimer = <float>state.entityOffhandSwingTimer[playerIndex];',
  'var target = new autoAttackState.AutoTarget();',
  'target.spikedHideDamage = 0;',
  'offlineMobTemplateId(state, targetIndex) == "wild_boar"',
  'target.spikedHideDamage = 2;',
  'autoAttackState.initializeAuthoritativeRng(',
  'autoAttackState.fixedTick(actor, target, events);',
  'state.entityHp[targetIndex] = target.hp > 0 ? target.hp : 0;',
  'state.entityOffhandSwingTimer[playerIndex] = actor.offhandSwingTimer;',
  'if (target.hp <= 0) {',
  'grantOfflineMobKillExperience(state, targetIndex, playerIndex);',
  'if (offlineMobTemplateId(state, targetIndex) == "forest_wolf") {',
  'creditOfflineQuestWolfKill(state, playerIndex);',
  'rollOfflineEastbrookCorpseLoot(state, targetIndex, playerIndex);',
  'state.rngState = autoAttackState.authoritativeRngState(events, true);',
  'stepOfflineDirectMeleeAutoAttack(state);',
  'stepOfflineEastbrookMobLifecycle(state);',
  'stepOfflineDirectMeleeAutoAttack(playerMelee);',
  'stepOfflineEastbrookMobLifecycle(playerMelee);',
  'playerMelee.offlinePlayerClass = <uint>2;',
  'offlinePlayerClass = <uint>2;',
  'offlinePlayerClass = <uint>3;',
  'offlinePlayerClass = <uint>6;',
  'offlinePlayerClass = <uint>8;',
  'var playerDualMelee = new WorldState();',
  'playerDualMelee.entityHasOffhandWeapon[0] = true;',
  'playerDualMelee.entityDualWielding[0] = true;',
  'playerDualMelee.entityOffhandSwingTimer[0] != 1.5',
  'var travelAttack = new WorldState();',
  'forms.formAbilityCode("form_travel")',
  'travelAttack.entityWeaponStowed[0] = true;',
  'travelAttack.entityAggroTargetIds[1] != <uint>0',
  'var playerBoarMelee = new WorldState();',
  'stepOfflineDirectMeleeAutoAttack(playerBoarMelee);',
  'var playerQuestMelee = new WorldState();',
  'playerQuestMelee.offlineQuestWolvesObjectiveCount != <uint>1',
]) {
  invariant(state.includes(needle), `WOS player melee auto integration is missing: ${needle}`);
}

invariant(
  state.includes('writer.u16(<uint>67, 1, 1);'),
  'WOS queued-on-swing persistence must remain in the current WOS envelope',
);

const projectilePrologue = state.indexOf('stepOfflineEastbrookProjectiles(state);');
const retainedPlayers = state.indexOf('stepRetainedPlayerTicks(state);');
const hunterLaunch = state.indexOf('stepOfflineHunterAutoShot(state);');
invariant(
  projectilePrologue >= 0 && retainedPlayers > projectilePrologue && hunterLaunch > retainedPlayers,
  'WOS projectile flight must advance before player swings and Hunter Auto Shot launch',
);

const meleeTick = state.indexOf('autoAttackState.fixedTick(actor, target, events);');
const committedCursor = state.indexOf(
  'state.rngState = autoAttackState.authoritativeRngState(events, true);',
  meleeTick,
);
const corpseRoll = state.indexOf(
  'rollOfflineEastbrookCorpseLoot(state, targetIndex, playerIndex);',
  meleeTick,
);
invariant(
  meleeTick >= 0 && committedCursor > meleeTick && corpseRoll > committedCursor,
  'WOS death loot must consume the RNG cursor committed by the white swing',
);

const killExperience = state.indexOf(
  'grantOfflineMobKillExperience(state, targetIndex, playerIndex);',
  meleeTick,
);
const questCreditAtDeath = state.indexOf('creditOfflineQuestWolfKill(state, playerIndex);', meleeTick);
invariant(
  killExperience > committedCursor && questCreditAtDeath > killExperience && corpseRoll > questCreditAtDeath,
  'WOS single-player kill XP must precede source quest credit and corpse-loot rolls',
);

process.stdout.write(`checked WOS50 player melee auto source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
