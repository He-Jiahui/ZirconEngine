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
const catalog = text('scripts/woc_game/src/generated/talent_proc_catalog.zr');
const procState = text('scripts/woc_game/src/combat/talent_proc_state.zr');
const state = text('scripts/woc_game/src/world/state.zr');
const fixture = text('scripts/woc_game/woc_m4_lean_quiver_runtime_tests.zrp');
const entry = text('scripts/woc_game/src/world/lean_quiver_runtime_test_main.zr');
const testStart = state.indexOf('pub leanQuiverRuntimeStateTest(): int {');
const testEnd = state.indexOf('\npub counterShotRuntimeStateTest(): int {', testStart);
const completionStart = state.indexOf('applyOfflineLeanQuiverCastCompleted(\n    state:');
const completionEnd = state.indexOf('\nspendOfflineAbilityResource(', completionStart);

invariant(
  testStart >= 0 && testEnd > testStart && completionStart >= 0 && completionEnd > completionStart,
  'Lean Quiver WOC runtime boundaries drifted',
);
const test = state.slice(testStart, testEnd);
const completion = state.slice(completionStart, completionEnd);

invariant(
  choices.includes("id: 'hun_r11_efficiency'") &&
    choices.includes("id: 'hun_lean_quiver'") &&
    choices.includes("on: 'castNth'") && choices.includes('n: 3') &&
    choices.includes('abilities: hunterRangedShotAbilityIds') &&
    choices.includes("responses: [{ kind: 'resource', amount: 20 }]") &&
    choices.includes("'concussive_shot'") && choices.includes("'counter_shot'"),
  'source Lean Quiver choice or ranged-shot contract drifted',
);
invariant(
  procRuntime.includes('export function onCastCompleted(') &&
    procRuntime.includes('const wasEmpowered = player.castConsumedEmpower === true;') &&
    procRuntime.includes('if (wasEmpowered) return;') &&
    procRuntime.includes("trigger.on !== 'castNth'") &&
    procRuntime.includes('procState.counters[def.id] = 0') &&
    procRuntime.includes('player.resource = Math.min(player.maxResource, player.resource + response.amount)'),
  'source cast-complete counter, free-cast exclusion, or capped resource ordering drifted',
);
invariant(
  catalog.includes('if (index == 6) return "hun_lean_quiver";') &&
    catalog.includes('if (index == 6) return <uint>44;') &&
    catalog.includes('if (index == 6) return 3;') &&
    catalog.includes('if (index == 6) return 9;') &&
    catalog.includes('if (index == 6 && responseIndex == 0) return 20.0;') &&
    procState.includes('pub onCastCompleted(') && procState.includes('pub planResponse('),
  'generated Lean Quiver proc projection drifted',
);
invariant(
  state.includes('offlineLeanQuiverEntityIds') &&
    state.includes('offlineLeanQuiverCounters') &&
    state.includes('schemaVersion != <uint>116') &&
    state.includes('schemaVersion >= <uint>116') &&
    state.includes('return encodeStateVersion(state, <uint>118);') &&
    state.includes('offlineLeanQuiverStateIsValid') &&
    state.includes('leanQuiverTalentProcIndex') &&
    completion.includes('talentProcs.onCastCompleted(') &&
    completion.includes('talentProcs.planResponse(') &&
    completion.includes('offlineCappedResourceGain(state, casterIndex, plan.resourceDelta)') &&
    completion.includes('castConsumedEmpower') &&
    state.includes('applyOfflineLeanQuiverCastCompleted(state, casterIndex, abilityCode, false);') &&
    !completion.includes('nextAuthoritativeRandomUnit(state)'),
  'WOC Lean Quiver persistence, completion, resource, or RNG projection drifted',
);
invariant(
  test.includes('offlineLeanQuiverCounter(state, playerId) != 1') &&
    test.includes('offlineLeanQuiverCounter(state, playerId) != 2') &&
    test.includes('state.entityZ[1] = 3.0') &&
    test.includes('decodeState(encodeState(state))') &&
    test.includes('offlineLeanQuiverCounter(restored, playerId) != 2') &&
    test.includes('offlineLeanQuiverCounter(restored, playerId) != 0') &&
    test.includes('setOfflineLeanQuiverCounter(capped, playerId, 2)') &&
    test.includes('startOfflineCounterShotCast(capped, 0, targetId)') &&
    test.includes('abilityCooldownExpiresAt(capped, 0, counterShot) <= capped.timeMicros') &&
    test.includes('applyOfflineLeanQuiverCastCompleted(capped, 0, wingClipAbilityCode(), false)') &&
    test.includes('applyOfflineLeanQuiverCastCompleted(capped, 0, concussiveShot, true)') &&
    test.includes('decodeState(encodeStateVersion(new WorldState(), <uint>115))') &&
    test.includes('restored.offlineProjectileSourceIds.length != 4'),
  'Lean Quiver cadence, Counter Shot, snapshot, historical schema, cap, or non-impact coverage drifted',
);
invariant(
  fixture.trim() === 'backend = "zr_vm:project"\nentry = "src/world/lean_quiver_runtime_test_main.zr"' &&
    entry.includes('return state.leanQuiverRuntimeStateTest();'),
  'Lean Quiver project fixture drifted',
);

process.stdout.write('WOS244 Lean Quiver source and WOC static contract guard passed\n');
