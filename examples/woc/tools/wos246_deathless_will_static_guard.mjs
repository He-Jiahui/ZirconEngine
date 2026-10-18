import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');

function source(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function text(path) {
  return readFileSync(join(projectRoot, path), 'utf8');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

const choices = source('src/sim/content/choice_rows_classic.ts');
const procRuntime = source('src/sim/combat/talent_procs.ts');
const damage = source('src/sim/combat/damage.ts');
const catalog = text('scripts/woc_game/src/generated/talent_proc_catalog.zr');
const procState = text('scripts/woc_game/src/combat/talent_proc_state.zr');
const state = text('scripts/woc_game/src/world/state.zr');
const fixture = text('scripts/woc_game/woc_m4_deathless_will_runtime_tests.zrp');
const entry = text('scripts/woc_game/src/world/deathless_will_runtime_test_main.zr');

const helperStart = state.indexOf('deathlessWillTalentProcIndex(): int {');
const helperEnd = state.indexOf('\nredlineDrawTalentProcIndex(): int {', helperStart);
const testStart = state.indexOf('pub deathlessWillRuntimeStateTest(): int {');
const testEnd = state.indexOf('\npub redlineDrawRuntimeStateTest(): int {', testStart);
const arcaneStart = state.indexOf('applyOfflineArcaneEffectiveDamage(state: WorldState');
const arcaneEnd = state.indexOf('\n// Temporal Echo', arcaneStart);
const fallStart = state.indexOf('stepRetainedPlayerMovement(');
const fallEnd = state.indexOf('\nloadCastingState(', fallStart);
const directStart = state.indexOf('stepOfflineDirectMeleeAutoAttack(state: WorldState): void {');
const directEnd = state.indexOf('\n// Source aura expiry', directStart);
const swingStart = state.indexOf('resolveOfflineEastbrookMobSwingRequests(');
const swingEnd = state.indexOf('\nstepOfflineEastbrookMobMeleePursuit(', swingStart);
const fixedTickStart = state.indexOf('pub fixedTick(');
const fixedTickEnd = state.indexOf('\npub selfTest()', fixedTickStart);

invariant(
  helperStart >= 0 && helperEnd > helperStart && testStart >= 0 && testEnd > testStart &&
    arcaneStart >= 0 && arcaneEnd > arcaneStart && fallStart >= 0 && fallEnd > fallStart &&
    directStart >= 0 && directEnd > directStart && swingStart >= 0 && swingEnd > swingStart &&
    fixedTickStart >= 0 && fixedTickEnd > fixedTickStart,
  'Deathless Will WOC runtime boundaries drifted',
);

const helper = state.slice(helperStart, helperEnd);
const test = state.slice(testStart, testEnd);
const arcane = state.slice(arcaneStart, arcaneEnd);
const fall = state.slice(fallStart, fallEnd);
const direct = state.slice(directStart, directEnd);
const swing = state.slice(swingStart, swingEnd);
const fixedTick = state.slice(fixedTickStart, fixedTickEnd);

invariant(
  choices.includes("id: 'hun_r11_survival_instincts'") &&
    choices.includes("id: 'hun_deathless_will'") && choices.includes("school: 'nature'") &&
    choices.includes("trigger: { on: 'bigHitTaken', hpFrac: 0.3, icd: 30 }") &&
    choices.includes("auraKind: 'buff_speed'") && choices.includes('value: 1.4') &&
    choices.includes('duration: 4') && choices.includes("name: 'Deathless Will'"),
  'source Deathless Will selection or aura contract drifted',
);
invariant(
  procRuntime.includes("trigger.on !== 'bigHitTaken' || amount < player.maxHp * trigger.hpFrac") &&
    procRuntime.includes('procState.icds[def.id] !== undefined') &&
    procRuntime.includes('procState.icds[def.id] = trigger.icd') && procRuntime.includes('fire(ctx, player, def, player)'),
  'source Deathless Will threshold, ICD, or deterministic response ordering drifted',
);
invariant(
  damage.includes('const preHp = target.hp;') &&
    damage.includes('target.hp = guardianWardRestore || Math.max(0, target.hp - amount);') &&
    damage.includes('if (amount > 0 && !target.dead) onDamageTaken(ctx, target, amount);') &&
    damage.indexOf('onDamageTaken(ctx, target, amount)') < damage.indexOf('if (target.hp <= 0)'),
  'source Deathless Will post-settlement and pre-death ordering drifted',
);
invariant(
  catalog.includes('if (index == 7) return <uint>45;') &&
    catalog.includes('if (index == 7) return "hun_deathless_will";') &&
    catalog.includes('if (index == 7) return 5;') && catalog.includes('if (index == 7) return 30.0;') &&
    catalog.includes('if (index == 7) return 0.3;') &&
    catalog.includes('if (index == 7 && responseIndex == 0) return 6;') &&
    catalog.includes('if (index == 7 && responseIndex == 0) return "buff_speed";') &&
    catalog.includes('if (index == 7 && responseIndex == 0) return 4.0;') &&
    catalog.includes('if (index == 7 && responseIndex == 0) return "Deathless Will";') &&
    catalog.includes('if (index == 7 && responseIndex == 0) return 1.4;') &&
    procState.includes('pub onBigHitTaken(') && procState.includes('armIcd(state, definition, result);'),
  'generated Deathless Will proc projection drifted',
);
invariant(
  state.includes('offlineDeathlessWillEntityIds') && state.includes('offlineDeathlessWillIcdRemaining') &&
    state.includes('schemaVersion != <uint>118') && state.includes('schemaVersion >= <uint>118') &&
    state.includes('return encodeStateVersion(state, <uint>118);') && state.includes('offlineDeathlessWillStateIsValid') &&
    state.includes('retainedSpeedAuraProfileIsValid') &&
    state.includes('if (abilityCode == deathlessWillSelectionCode())') &&
    helper.includes('talentProcs.onBigHitTaken(') && helper.includes('talentProcs.planResponse(') &&
    helper.includes('setOfflineDeathlessWillIcd(state, ownerId') &&
    helper.includes('plan.auraKind == "buff_speed"') && helper.includes('plan.auraSchool == "nature"') &&
    helper.includes('OFFLINE_PROJECTILE_SCHOOL_NATURE') &&
    !helper.includes('nextAuthoritativeRandomUnit(state)') &&
    fixedTick.includes('ageOfflineDeathlessWill(state);'),
  'Deathless Will WOC persistence, aura, tick, or RNG projection drifted',
);
invariant(
  arcane.includes('applyOfflineDeathlessWillDamageTaken(state, targetIndex, remainingDamage);') &&
    fall.includes('applyOfflineDeathlessWillDamageTaken(state, index, transition.landingDamage);') &&
    swing.includes('applyOfflineDeathlessWillDamageTaken(state, playerIndex, dealt);') &&
    direct.includes('var playerReactionDamage = playerHpBefore - actor.hp;') &&
    direct.includes('applyOfflineDeathlessWillDamageTaken(state, playerIndex, playerReactionDamage);'),
  'Deathless Will post-settlement damage boundary coverage drifted',
);
invariant(
  test.includes('applyOfflineDeathlessWillDamageTaken(state, 0, 0);') &&
    test.includes('applyOfflineDeathlessWillDamageTaken(state, 0, 29);') &&
    test.includes('applyOfflineDeathlessWillDamageTaken(state, 0, 30);') &&
    test.includes('while (tick < 600)') && test.includes('ageOfflineDeathlessWill(restored);') &&
    test.includes('ageMotionAurasForEntity(restored, 0);') &&
    test.includes('decodeState(encodeStateVersion(new WorldState(), <uint>117))') &&
    test.includes('stepOfflineEastbrookMobMeleePursuit(actual);') &&
    test.includes('actual.rngDraws != <uint>3') && test.includes('state.rngDraws != <uint>0'),
  'Deathless Will threshold, ICD, expiry, deselection, snapshot, or actual melee coverage drifted',
);
invariant(
  fixture.trim() === 'backend = "zr_vm:project"\nentry = "src/world/deathless_will_runtime_test_main.zr"' &&
    entry.includes('return state.deathlessWillRuntimeStateTest();'),
  'Deathless Will project fixture drifted',
);

process.stdout.write('WOS246 Deathless Will source and WOC static contract guard passed\n');
