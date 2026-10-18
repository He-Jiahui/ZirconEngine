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

const talent = source('src/sim/content/talent_abilities_v2_a.ts');
const choices = source('src/sim/content/choice_rows_classic.ts');
const trapRuntime = source('src/sim/combat/hunter_trap.ts');
const roster = source('src/sim/entity_roster.ts');
const contracts = JSON.parse(text('contracts/m4_abilities.json'));
const catalog = text('scripts/woc_game/src/generated/m4_ability_catalog.zr');
const effects = text('scripts/woc_game/src/generated/m4_ability_effects.zr');
const state = text('scripts/woc_game/src/world/state.zr');
const fixture = text('scripts/woc_game/woc_m4_frost_trap_runtime_tests.zrp');
const entry = text('scripts/woc_game/src/world/frost_trap_runtime_test_main.zr');
const frostTrapTestStart = state.indexOf('pub frostTrapRuntimeStateTest(): int {');
const frostTrapTestEnd = state.indexOf('\n// Hemorrhage', frostTrapTestStart);

invariant(
  frostTrapTestStart >= 0 && frostTrapTestEnd > frostTrapTestStart,
  'Frost Trap runtime test boundary drifted',
);
const frostTrapTest = state.slice(frostTrapTestStart, frostTrapTestEnd);

invariant(
  talent.includes("id: 'frost_trap'") &&
    talent.includes("class: 'hunter'") &&
    talent.includes('learnLevel: 10') && talent.includes('cost: 35') &&
    talent.includes('castTime: 0') && talent.includes('cooldown: 30') &&
    talent.includes('range: 0') && talent.includes("school: 'frost'") &&
    talent.includes('requiresTarget: false') &&
    talent.includes("type: 'aoeRoot'") && talent.includes('duration: 3') &&
    talent.includes('radius: 3') && talent.includes('min: 0') && talent.includes('max: 0') &&
    talent.includes('stun: true') && talent.includes('armTime: 1.5') &&
    talent.includes('lifetime: 60'),
  'source Frost Trap ability contract drifted',
);
invariant(
  choices.includes("id: 'hun_r8_frost_trap'") &&
    choices.includes("ability: 'frost_trap'"),
  'source Frost Trap choice grant drifted',
);
invariant(
  trapRuntime.includes('spawnHunterTrap') &&
    trapRuntime.includes('armRemaining: effect.trap.armTime') &&
    trapRuntime.includes('freezeDuration: effect.duration') &&
    trapRuntime.includes('segmentTouchesAnnulus') &&
    trapRuntime.includes("'controlledStun'") && trapRuntime.includes("kind: 'stun'") &&
    trapRuntime.includes('trap.triggered = true'),
  'source Frost Trap arm/contact/stun transition drifted',
);
invariant(
  roster.includes('effect.remaining -= DT') &&
    roster.includes('effect.remaining <= CAST_COMPLETE_EPS || effect.hunterTrap.triggered') &&
    roster.includes('ctx.groundAoEs.splice(i, 1)'),
  'source Frost Trap lifetime or consumption ordering drifted',
);

const frostTrap = contracts.entries.find((entryValue) => entryValue.id === 'frost_trap');
invariant(
  contracts.entries.length === 117 && frostTrap?.index === 116 &&
    frostTrap.source_owner ===
      'src/sim/content/talent_abilities_v2_a.ts#TALENT_ABILITIES_V2_A.frost_trap' &&
    frostTrap.definition.effects.length === 1 &&
    frostTrap.definition.effects[0].type === 'aoeRoot' &&
    frostTrap.definition.effects[0].stun === true &&
    frostTrap.definition.effects[0].trap?.armTime === 1.5 &&
    frostTrap.definition.effects[0].trap?.lifetime === 60,
  'generated Frost Trap contract drifted',
);
invariant(
  catalog.includes('pub count(): int {\n    return 117;') &&
    catalog.includes('if (id == "frost_trap") { return 116; }') &&
    effects.includes('if (field == "trapArmTime") { return 1.5; }') &&
    effects.includes('if (field == "trapLifetime") { return 60.0; }') &&
    effects.includes('if (field == "stun") { return true; }'),
  'generated Frost Trap scalar projection drifted',
);
invariant(
  state.includes('%import("combat/ring_of_frost_geometry")') &&
    state.includes('offlineHunterTrapSourceIds') &&
    state.includes('schemaVersion != <uint>115') &&
    state.includes('schemaVersion >= <uint>115') &&
    state.includes('return encodeStateVersion(state, <uint>118);') &&
    state.includes('frostTrapProfileIsValid') &&
    state.includes('effects.metric(abilityIndex, 1, 0, "trapArmTime") == 1.5') &&
    state.includes('effects.metric(abilityIndex, 1, 0, "trapLifetime") == 60.0') &&
    state.includes('startOfflineFrostTrapCast') &&
    state.includes('stepOfflineHunterTraps') &&
    state.includes('ringOfFrostGeometry.segmentTouchesAnnulus') &&
    state.includes('ccContract.motionAuraKindCode("stun")') &&
    state.includes('OFFLINE_PROJECTILE_SCHOOL_FROST') &&
    state.includes('frostTrapPayloadAbilityIsExact') &&
    state.includes('frostTrapRuntimeStateTest') &&
    !state.includes('nextAuthoritativeRandomUnit(state);\n    state.offlineHunterTrap'),
  'WOC Frost Trap state projection drifted',
);
invariant(
  frostTrapTest.includes('encodeStateVersion(state, <uint>114)') &&
    frostTrapTest.includes('schema114.offlineHunterTrapSourceIds.length != 0') &&
    frostTrapTest.includes('fullIndex < <int>OFFLINE_DOT_PENDING_MAX') &&
    frostTrapTest.includes('fullIndex == 0 ? fullPlayerId : <uint>(1000 + fullIndex)') &&
    frostTrapTest.includes('preservedIndex < <int>OFFLINE_DOT_PENDING_MAX - 1') &&
    frostTrapTest.includes('<uint>(1001 + preservedIndex)') &&
    frostTrapTest.includes('<uint>full.offlineHunterTrapSourceIds[63] != fullPlayerId') &&
    frostTrapTest.includes('abilityCooldownExpiresAt(full, 0, frostTrap) != <uint>30000000'),
  'Frost Trap migration or full-capacity replacement regression coverage drifted',
);
invariant(
  fixture.trim() === 'backend = "zr_vm:project"\nentry = "src/world/frost_trap_runtime_test_main.zr"' &&
    entry.includes('return state.frostTrapRuntimeStateTest();'),
  'Frost Trap project fixture drifted',
);

process.stdout.write('WOS242 Frost Trap source and WOC static contract guard passed\n');
