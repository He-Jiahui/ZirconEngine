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
const codegen = text('tools/talent_proc_catalog_codegen.mjs');
const catalog = text('scripts/woc_game/src/generated/talent_proc_catalog.zr');
const procState = text('scripts/woc_game/src/combat/talent_proc_state.zr');
const state = text('scripts/woc_game/src/world/state.zr');
const fixture = text('scripts/woc_game/woc_m4_redline_draw_runtime_tests.zrp');
const entry = text('scripts/woc_game/src/world/redline_draw_runtime_test_main.zr');
const testStart = state.indexOf('pub redlineDrawRuntimeStateTest(): int {');
const testEnd = state.indexOf('\npub leanQuiverRuntimeStateTest(): int {', testStart);
const helperStart = state.indexOf('applyOfflineRedlineDrawCastCompleted(\n    state:');
const helperEnd = state.indexOf('\nspendOfflineAbilityResource(', helperStart);
const fixedTickStart = state.indexOf('pub fixedTick(');
const fixedTickEnd = state.indexOf('\npub selfTest()', fixedTickStart);
const counterShotStart = testStart >= 0
  ? state.indexOf('    // Counter Shot is an independent, immediate retained completion boundary.', testStart)
  : -1;
const counterShotEnd = counterShotStart >= 0
  ? state.indexOf('    // A short remaining cooldown follows the source delete branch.', counterShotStart)
  : -1;

invariant(
  testStart >= 0 && testEnd > testStart && helperStart >= 0 && helperEnd > helperStart &&
    fixedTickStart >= 0 && fixedTickEnd > fixedTickStart &&
    counterShotStart > testStart && counterShotEnd > counterShotStart,
  'Redline Draw WOC runtime boundaries drifted',
);
const test = state.slice(testStart, testEnd);
const counterShot = state.slice(counterShotStart, counterShotEnd);
const helper = state.slice(helperStart, helperEnd);
const fixedTick = state.slice(fixedTickStart, fixedTickEnd);

invariant(
  choices.includes("id: 'hun_r20_rapid_killing'") &&
    choices.includes("id: 'hun_redline_draw'") &&
    choices.includes("on: 'castNth'") && choices.includes('n: 3') &&
    choices.includes('abilities: hunterRangedShotAbilityIds') && choices.includes('icd: 8') &&
    choices.includes("kind: 'cooldownRefund', ability: 'rapid_fire', seconds: 5"),
  'source Redline Draw choice or refund contract drifted',
);
invariant(
  procRuntime.includes('export function tickProcState(') &&
    procRuntime.includes("case 'cooldownRefund':") &&
    procRuntime.includes('player.cooldowns.delete(response.ability)') &&
    procRuntime.includes('player.cooldowns.set(response.ability, remaining - response.seconds)') &&
    procRuntime.includes('if (wasEmpowered) return;'),
  'source Redline Draw ICD, refund, or free-cast ordering drifted',
);
invariant(
  codegen.includes('cooldown_seconds: typeof response.seconds') &&
    codegen.includes('pub responseCooldownSeconds') &&
    catalog.includes('if (index == 8) return "hun_redline_draw";') &&
    catalog.includes('if (index == 8) return <uint>53;') &&
    catalog.includes('if (index == 8) return 3;') &&
    catalog.includes('if (index == 8) return 8.0;') &&
    catalog.includes('if (index == 8 && responseIndex == 0) return "rapid_fire";') &&
    catalog.includes('if (index == 8 && responseIndex == 0) return 5.0;') &&
    procState.includes('pub var cooldownSeconds: float;') &&
    procState.includes('catalog.responseCooldownSeconds(index, responseIndex)') &&
    procState.includes('currentCooldown - response.cooldownSeconds'),
  'generated Redline Draw cooldown-seconds projection drifted',
);
invariant(
  state.includes('offlineRedlineDrawEntityIds') &&
    state.includes('offlineRedlineDrawCounters') &&
    state.includes('offlineRedlineDrawIcdRemaining') &&
    state.includes('schemaVersion != <uint>117') &&
    state.includes('schemaVersion >= <uint>117') &&
    state.includes('return encodeStateVersion(state, <uint>118);') &&
    state.includes('offlineRedlineDrawStateIsValid') &&
    helper.includes('talentProcs.onCastCompleted(') &&
    helper.includes('talentProcs.planResponse(') &&
    helper.includes('plan.cooldownAbilityId == "rapid_fire"') &&
    helper.includes('clearAbilityCooldown(state, casterIndex, rapidFire)') &&
    helper.includes('setAbilityCooldownExpiration(state, casterIndex, rapidFire, nextExpires)') &&
    helper.includes('castConsumedEmpower') &&
    !helper.includes('nextAuthoritativeRandomUnit(state)') &&
    fixedTick.includes('ageOfflineRedlineDraw(state);') &&
    state.includes('applyOfflineRedlineDrawCastCompleted(state, casterIndex, abilityCode, false);'),
  'WOC Redline Draw persistence, completion, cooldown bridge, tick, or RNG projection drifted',
);
invariant(
  test.includes('offlineRedlineDrawCounter(state, playerId) != 1') &&
    test.includes('offlineRedlineDrawCounter(state, playerId) != 2') &&
    test.includes('offlineRedlineDrawIcdRemaining(state, playerId), 8.0') &&
    test.includes('abilityCooldownExpiresAt(state, 0, rapidFire) != <uint>295000000') &&
    test.includes('while (tick < 160)') &&
    test.includes('replaceOfflineTalentAllocation(\n            restored') &&
    test.includes('state.entityKnownAbilityCodes.add(counterShot);') &&
    test.includes('abilityCooldownExpiresAt(capped, 0, rapidFire) != capped.timeMicros') &&
    test.includes('applyOfflineRedlineDrawCastCompleted(capped, 0, concussiveShot, true)') &&
    test.includes('decodeState(encodeStateVersion(new WorldState(), <uint>116))') &&
    test.includes('rngDraws != <uint>0'),
  'Redline Draw cadence, ICD, deselection, snapshot, short cooldown, historical schema, or free-cast coverage drifted',
);
invariant(
  counterShot.includes('setAbilityCooldownExpiration(capped, 0, rapidFire, capped.timeMicros + <uint>300000000);') &&
    counterShot.includes('setOfflineRedlineDrawState(capped, playerId, 2, 0.0);') &&
    counterShot.includes('startOfflineCounterShotCast(capped, 0, targetId);') &&
    counterShot.includes('offlineRedlineDrawCounter(capped, playerId) != 0') &&
    counterShot.includes('!floatMatches(offlineRedlineDrawIcdRemaining(capped, playerId), 8.0)') &&
    counterShot.includes('abilityCooldownExpiresAt(capped, 0, rapidFire) != capped.timeMicros + <uint>295000000') &&
    counterShot.includes('capped.rngDraws != <uint>0'),
  'Redline Draw Counter Shot completion coverage drifted',
);
invariant(
  fixture.trim() === 'backend = "zr_vm:project"\nentry = "src/world/redline_draw_runtime_test_main.zr"' &&
    entry.includes('return state.redlineDrawRuntimeStateTest();'),
  'Redline Draw project fixture drifted',
);

process.stdout.write('WOS245 Redline Draw source and WOC static contract guard passed\n');
