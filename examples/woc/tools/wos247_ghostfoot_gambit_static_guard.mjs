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
const casting = source('src/sim/combat/casting_lifecycle.ts');
const catalog = text('scripts/woc_game/src/generated/talent_proc_catalog.zr');
const procState = text('scripts/woc_game/src/combat/talent_proc_state.zr');
const state = text('scripts/woc_game/src/world/state.zr');
const fixture = text('scripts/woc_game/woc_m4_ghostfoot_gambit_runtime_tests.zrp');
const entry = text('scripts/woc_game/src/world/ghostfoot_gambit_runtime_test_main.zr');

const helperStart = state.indexOf('ghostfootGambitTalentProcIndex(): int {');
const helperEnd = state.indexOf('\ndeathlessWillTalentProcIndex(): int {', helperStart);
const evasionStart = state.indexOf('startOfflineEvasionCast(state: WorldState, casterIndex: int): void {');
const evasionEnd = state.indexOf('\nstartOfflineDeterrenceCast(', evasionStart);
const hemorrhageStart = state.indexOf('startOfflineHemorrhageCast(state: WorldState, casterIndex: int, targetId: uint): void {');
const hemorrhageEnd = state.indexOf('\nclawTargetIndex(', hemorrhageStart);
const testStart = state.indexOf('pub ghostfootGambitRuntimeStateTest(): int {');
const testEnd = state.indexOf('\npub deathlessWillRuntimeStateTest(): int {', testStart);

invariant(
  helperStart >= 0 && helperEnd > helperStart && evasionStart >= 0 && evasionEnd > evasionStart &&
    hemorrhageStart >= 0 && hemorrhageEnd > hemorrhageStart && testStart >= 0 && testEnd > testStart,
  'Ghostfoot Gambit WOC runtime boundaries drifted',
);

const helper = state.slice(helperStart, helperEnd);
const evasion = state.slice(evasionStart, evasionEnd);
const hemorrhage = state.slice(hemorrhageStart, hemorrhageEnd);
const test = state.slice(testStart, testEnd);

invariant(
  choices.includes("id: 'rog_r17_improved_evasion'") &&
    choices.includes("id: 'rog_improved_evasion'") &&
    choices.includes("trigger: { on: 'castNth', n: 1, abilities: ['evasion'] }") &&
    choices.includes("{ kind: 'resource', amount: 30 }") &&
    choices.includes("aura: 'next_cast_cheap'") && choices.includes('duration: 8') &&
    choices.includes('costPct: 0.5'),
  'source Ghostfoot Gambit selection or response contract drifted',
);
invariant(
  casting.includes('const cheapMultiplier = nextCastCheapMultiplier(p, ability.id);') &&
    casting.includes('Math.ceil(res.cost * cheapMultiplier)') &&
    casting.includes('const cheap = consumeNextCastCheap(ctx, p, ability.id);'),
  'source cheap-cost admission or successful-cast consumption ordering drifted',
);
invariant(
  catalog.includes('if (index == 17) return <uint>68;') &&
    catalog.includes('if (index == 17) return "rog_improved_evasion";') &&
    catalog.includes('if (index == 17 && responseIndex == 1) return "next_cast_cheap";') &&
    catalog.includes('if (index == 17 && responseIndex == 1) return 8.0;') &&
    catalog.includes('if (index == 17 && responseIndex == 1) return 0.5;') &&
    procState.includes('pub onCastCompleted(') && procState.includes('pub planResponse('),
  'generated Ghostfoot Gambit proc projection drifted',
);
invariant(
  helper.includes('talentProcs.onCastCompleted(') && helper.includes('talentProcs.planResponse(') &&
    helper.includes('offlineCappedResourceGain(state, casterIndex, plan.resourceDelta);') &&
    helper.includes('nextCastModifiers.cheapCostKind()') &&
    helper.includes('setNextCastModifier(') && helper.includes('plan.auraDuration == 8.0') &&
    helper.includes('plan.auraValue == 0.5') && !helper.includes('nextAuthoritativeRandomUnit(state)'),
  'Ghostfoot Gambit response order, modifier bridge, or deterministic RNG contract drifted',
);
invariant(
  evasion.includes('applyOfflineGhostfootGambitCastCompleted(state, casterIndex);') &&
    evasion.indexOf('applyOfflineGhostfootGambitCastCompleted(state, casterIndex);') >
      evasion.indexOf('applyOfflineMotionAuraWithDetails(') &&
    hemorrhage.includes('nextCastModifierValueForAbility(') &&
    hemorrhage.includes('terrainGradient.javascriptCeil(') &&
    hemorrhage.includes('consumeNextCastModifierForAbility(') &&
    hemorrhage.indexOf('consumeNextCastModifierForAbility(') > hemorrhage.indexOf('spendOfflineAbilityResource('),
  'Ghostfoot Gambit Evasion completion or Hemorrhage cheap-cost boundary drifted',
);
invariant(
  test.includes('startOfflineEvasionCast(state, 0);') &&
    test.includes('startOfflineHemorrhageCast(state, 0, targetId);') &&
    test.includes('state.entityResources[0] != 18') &&
    test.includes('ageOfflineNextCastModifiers(restored);') &&
    test.includes('while (tick < 160)') && test.includes('state.rngDraws != <uint>0'),
  'Ghostfoot Gambit gate, refund, cost, expiry, or no-RNG coverage drifted',
);
invariant(
  fixture.trim() === 'backend = "zr_vm:project"\nentry = "src/world/ghostfoot_gambit_runtime_test_main.zr"' &&
    entry.includes('return state.ghostfootGambitRuntimeStateTest();'),
  'Ghostfoot Gambit project fixture drifted',
);

process.stdout.write('WOS247 Ghostfoot Gambit source and WOC static contract guard passed\n');
