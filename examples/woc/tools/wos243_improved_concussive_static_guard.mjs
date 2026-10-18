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
const classes = source('src/sim/content/classes.ts');
const effectDispatch = source('src/sim/combat/effect_dispatch.ts');
const auraStacking = source('src/sim/combat/aura_stacking.ts');
const playerMotion = source('src/sim/player_motion.ts');
const modifiers = text('scripts/woc_game/src/generated/talent_modifier_catalog.zr');
const selections = text('scripts/woc_game/src/generated/current_talent_selection_catalog.zr');
const addedEffects = text('scripts/woc_game/src/combat/talent_added_effect_state.zr');
const motion = text('scripts/woc_game/src/world/motion_aura_state.zr');
const motionTest = text('scripts/woc_game/src/world/motion_aura_state_test_main.zr');
const state = text('scripts/woc_game/src/world/state.zr');
const fixture = text('scripts/woc_game/woc_m4_improved_concussive_runtime_tests.zrp');
const entry = text('scripts/woc_game/src/world/improved_concussive_runtime_test_main.zr');
const testStart = state.indexOf('pub improvedConcussiveRuntimeStateTest(): int {');
const testEnd = state.indexOf('\npub counterShotRuntimeStateTest', testStart);

invariant(
  choices.includes("id: 'hun_r8_improved_concussive'") &&
    choices.includes("ability: 'concussive_shot'") &&
    choices.includes("type: 'slow', mult: 0.3, duration: 4"),
  'source Improved Concussive choice contract drifted',
);
invariant(
  classes.includes('entry.effects = [...entry.effects, ...am.addEffects.map') &&
    effectDispatch.includes('id: `${ability.id}_slow`') &&
    auraStacking.includes('if (existing.id !== aura.id) continue;') &&
    auraStacking.includes('existing.sourceId === aura.sourceId') &&
    playerMotion.includes("a.kind === 'slow' || a.kind === 'stealth'") &&
    playerMotion.includes('slow = Math.min(slow, a.value)'),
  'source Concussive appended-slow replacement semantics drifted',
);
invariant(
  selections.includes('id == "hun_r8_improved_concussive") return <uint>42') &&
    modifiers.includes('if (index == 50 && abilityIndex == 0) return "concussive_shot";') &&
    modifiers.includes('if (index == 50 && abilityIndex == 0 && effectIndex == 0) return 2;') &&
    modifiers.includes('if (index == 50 && abilityIndex == 0 && effectIndex == 0 && field == 1) return 4.0;') &&
    modifiers.includes('if (index == 50 && abilityIndex == 0 && effectIndex == 0 && field == 2) return 0.3;'),
  'generated Improved Concussive nested slow DTO drifted',
);
invariant(
  addedEffects.includes('effectSlow(): int { return 2; }') &&
    addedEffects.includes('pub isSlow(') &&
    addedEffects.includes('pub slowDuration(') &&
    addedEffects.includes('pub slowMult(') &&
    addedEffects.includes('hun_r8_improved_concussive') === false &&
    addedEffects.includes('concussive.playerLevel = 8') &&
    addedEffects.includes('slowDuration(effects, 0) != 4.0') &&
    addedEffects.includes('slowMult(effects, 0) != 0.3'),
  'typed Improved Concussive effect projection drifted',
);
invariant(
  motion.includes('var rowSlow = <float>values[index];') &&
    motion.includes('if (!(rowSlow > 0.0) || rowSlow > 1.0) { rowSlow = slowMultiplier; }') &&
    motion.includes('if (rowSlow < slow) { slow = rowSlow; }') &&
    motionTest.includes('values.add(0.3);') &&
    motionTest.includes(') != 0.3)') &&
    motionTest.includes('values[4] = 0.0;'),
  'motion slow-value aggregation drifted',
);
invariant(
  state.includes('applyOfflineConcussiveShotAddedEffects') &&
    state.includes('talentAddedEffects.resolveForAbility(modifiers, "concussive_shot", true, effects);') &&
    state.includes('if (duration == 4.0 && mult == 0.3)') &&
    state.includes('applyOfflineConcussiveShotAddedEffects(state, projectileIndex, casterIndex, targetIndex);') &&
    state.includes('var slow = ccContract.motionAuraKindCode("slow");') &&
    state.includes('kindCode != bleedVulnerability && kindCode != slow') &&
    testStart >= 0 && testEnd > testStart,
  'Improved Concussive landing projection drifted',
);
const improvedTest = state.slice(testStart, testEnd);
invariant(
  improvedTest.includes('concussiveShotRuntimeStateTest() != 1') &&
    improvedTest.includes('hun_r8_improved_concussive') &&
    improvedTest.includes('offlineProjectileTalentRowOptionCodes[1] != improved') &&
    improvedTest.includes('replaceOfflineTalentAllocation(\n            state, 0, <uint>0, <uint>0, <uint>0') &&
    improvedTest.includes('!floatMatches(<float>restored.entityMotionAuraValues[0], 0.3)') &&
    improvedTest.includes('motionAuras.movementMultiplier(') &&
    improvedTest.includes('fizzled.entityDead[1] = true') &&
    improvedTest.includes('var otherSourceId = restored.spawnPlayer(') &&
    improvedTest.includes('<uint>restored.entityMotionAuraSourceIds[1] != otherSourceId') &&
    improvedTest.includes('!floatMatches(<float>restored.entityMotionAuraValues[1], 0.4)') &&
    improvedTest.includes('appendTypedCastTargetCommandForTest(') &&
    improvedTest.includes('!floatMatches(<float>typed.entityMotionAuraValues[0], 0.3)'),
  'Improved Concussive launch-snapshot or command coverage drifted',
);
invariant(
  fixture.trim() === 'backend = "zr_vm:project"\nentry = "src/world/improved_concussive_runtime_test_main.zr"' &&
    entry.includes('return state.improvedConcussiveRuntimeStateTest();'),
  'Improved Concussive project fixture drifted',
);

process.stdout.write('WOS243 Improved Concussive source and WOC static contract guard passed\n');
