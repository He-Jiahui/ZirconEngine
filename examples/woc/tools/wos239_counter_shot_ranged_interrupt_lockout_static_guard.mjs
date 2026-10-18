import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (path) => readFileSync(resolve(root, path), 'utf8');
const sourceRoot = resolve(root, '..', '..', 'dev', 'world-of-claudecraft');
const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const readPinned = (path) => {
  const result = spawnSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], {
    encoding: 'utf8',
  });
  if (result.status !== 0) throw new Error(`unable to read source-pinned ${path}`);
  return result.stdout;
};
const requireText = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`missing ${label}: ${needle}`);
};

const classes = readPinned('src/sim/content/classes.ts');
const lifecycle = readPinned('src/sim/combat/casting_lifecycle.ts');
const dispatch = readPinned('src/sim/combat/effect_dispatch.ts');
const state = read('scripts/woc_game/src/world/state.zr');
const catalog = read('scripts/woc_game/src/generated/m4_ability_catalog.zr');
const effects = read('scripts/woc_game/src/generated/m4_ability_effects.zr');
const coverage = read('reference/current-head/m4_ability_projection_coverage.json');
const fixture = read('scripts/woc_game/woc_m4_counter_shot_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/counter_shot_runtime_test_main.zr');

requireText(classes, 'counter_shot: {', 'source Counter Shot identity');
requireText(classes, 'bladestorm: {', 'source uninterruptible target identity');
requireText(classes, 'uninterruptible: true', 'source uninterruptible target flag');
requireText(classes, "name: 'Hushing Shot'", 'source Counter Shot name');
requireText(classes, 'learnLevel: 10', 'source Counter Shot level');
requireText(classes, 'cost: 35', 'source Counter Shot cost');
requireText(classes, 'castTime: 0', 'source Counter Shot instant cast');
requireText(classes, 'cooldown: 20', 'source Counter Shot cooldown');
requireText(classes, 'range: 35', 'source Counter Shot range');
requireText(classes, 'minRange: 8', 'source Counter Shot minimum range');
requireText(classes, "school: 'physical'", 'source Counter Shot school');
requireText(classes, "scalesWith: 'ranged'", 'source Counter Shot scaling class');
requireText(classes, 'requiresTarget: true', 'source Counter Shot target requirement');
requireText(classes, "{ type: 'interrupt', lockout: 4 }", 'source Counter Shot effect');
requireText(lifecycle, 'if (ability.minRange && d < ability.minRange)',
  'source minimum range admission');
requireText(lifecycle, 'const firesProjectile = ability.projectile ?? ability.school !== \'physical\';',
  'source physical non-projectile dispatch');
requireText(dispatch, "case 'interrupt': {", 'source interrupt dispatch');
requireText(dispatch, 'target.castingAbility === null', 'source no-active-cast branch');
requireText(dispatch, 'target.castingAbility === FISHING_CAST_ID', 'source fishing immunity');
requireText(dispatch, "interruptedDef?.school === 'physical'", 'source physical-cast immunity');
requireText(dispatch, 'interruptedDef?.uninterruptible', 'source uninterruptible-cast immunity');
requireText(dispatch, "diminishedCrowdControlDuration(p, target, 'lockout', eff.lockout)",
  'source lockout DR dispatch');
requireText(dispatch, 'ctx.cancelCast(target)', 'source cancel before lockout');
requireText(dispatch, "kind: 'lockout'", 'source school lockout aura');

requireText(state, 'counterShotAbilityCode', 'Counter Shot catalog identity');
requireText(state, 'counterShotPayloadAbilityIsExact', 'Counter Shot typed payload');
requireText(state, 'counterShotProfileIsValid', 'Counter Shot source profile');
requireText(state, 'counterShotTargetIndex', 'Counter Shot range admission');
requireText(state, 'startOfflineCounterShotCast', 'Counter Shot reducer');
requireText(state, 'pummelTargetCastIsInterruptible(state, targetIndex)',
  'Counter Shot interruptible-target reuse');
requireText(state, 'cancelOfflineTargetCast(state, targetIndex)',
  'Counter Shot immediate cast cancellation');
requireText(state, 'applyOfflineMotionAuraWithDetailsAndSchool',
  'Counter Shot school-keyed lockout');
requireText(state, 'counterShotRuntimeStateTest', 'Counter Shot runtime test');
requireText(state, 'pummelCastIsInterruptible(true, false, OFFLINE_PROJECTILE_SCHOOL_FROST, true)',
  'Counter Shot uninterruptible-cast runtime regression');
const counterShotTestStart = state.indexOf('pub counterShotRuntimeStateTest(): int {');
const counterShotTestEnd = state.indexOf('\n// Hemorrhage', counterShotTestStart);
if (counterShotTestStart < 0 || counterShotTestEnd < 0) {
  throw new Error('unable to isolate Counter Shot runtime test');
}
const counterShotTest = state.slice(counterShotTestStart, counterShotTestEnd);
const physicalImmunityStart = counterShotTest.indexOf('noActive.entityCastingAbility[1] = pummel');
const physicalImmunityEnd = counterShotTest.indexOf('noActive.entityCastingAbility[1] = frostbolt', physicalImmunityStart);
const physicalImmunity = counterShotTest.slice(physicalImmunityStart, physicalImmunityEnd);
requireText(physicalImmunity, 'entityCastGcdRemaining[0], 1.5',
  'physical cast immunity still applies Counter Shot GCD');
requireText(physicalImmunity, 'noActive.timeMicros + <uint>20000000',
  'physical cast immunity still arms Counter Shot cooldown');
const fishingImmunityStart = counterShotTest.indexOf('fishing.fishing = true');
const fishingImmunityEnd = counterShotTest.indexOf('fishing.fishing = false', fishingImmunityStart);
const fishingImmunity = counterShotTest.slice(fishingImmunityStart, fishingImmunityEnd);
requireText(fishingImmunity, 'entityCastGcdRemaining[0], 1.5',
  'fishing immunity still applies Counter Shot GCD');
requireText(fishingImmunity, 'noActive.timeMicros + <uint>20000000',
  'fishing immunity still arms Counter Shot cooldown');
const uninterruptibleStart = counterShotTest.indexOf('noActive.entityCastingAbility[1] = bladestorm');
const uninterruptibleEnd = counterShotTest.indexOf('noActive.entityZ[1] = 7.0', uninterruptibleStart);
const uninterruptibleImmunity = counterShotTest.slice(uninterruptibleStart, uninterruptibleEnd);
requireText(uninterruptibleImmunity, 'entityCastGcdRemaining[0], 1.5',
  'uninterruptible cast still applies Counter Shot GCD');
requireText(uninterruptibleImmunity, 'noActive.timeMicros + <uint>20000000',
  'uninterruptible cast still arms Counter Shot cooldown');
requireText(uninterruptibleImmunity, 'entityCastingAbility[1] != bladestorm',
  'uninterruptible cast remains active after Counter Shot');
const reducerStart = state.indexOf('startOfflineCounterShotCast(');
const reducerEnd = state.indexOf('\nwingClipAbilityCode()', reducerStart);
if (reducerStart < 0 || reducerEnd < 0) {
  throw new Error('unable to isolate Counter Shot reducer');
}
const reducer = state.slice(reducerStart, reducerEnd);
const spend = reducer.indexOf('spendOfflineAbilityResource');
const cooldown = reducer.indexOf('setAbilityCooldownExpiration');
const interruptible = reducer.indexOf('pummelTargetCastIsInterruptible');
const cancel = reducer.indexOf('cancelOfflineTargetCast');
const lockout = reducer.indexOf('applyOfflineMotionAuraWithDetailsAndSchool');
if (spend < 0 || cooldown < spend || interruptible < cooldown || cancel < interruptible ||
    lockout < cancel) {
  throw new Error('Counter Shot must spend and arm cooldown before interrupt resolution');
}
if (reducer.includes('appendOfflineAbilityProjectile')) {
  throw new Error('Counter Shot must resolve as an immediate physical non-projectile');
}
requireText(state, 'if (abilityCode == counterShotAbilityCode())', 'Counter Shot slot route');
requireText(state, 'if (counterShotPayloadAbilityIsExact(payloadBytes, abilityOffset, abilityLength))',
  'Counter Shot typed route');
requireText(catalog, 'if (id == "counter_shot") { return 112; }',
  'Counter Shot catalog index');
requireText(catalog, 'if (id == "bladestorm") { return 113; }',
  'source uninterruptible target catalog index');
requireText(catalog, 'if (index == 113) {\n        if (field == "requiresTarget") { return false; }',
  'source uninterruptible target catalog row');
requireText(catalog, 'if (field == "uninterruptible") { return true; }',
  'source uninterruptible target catalog flag');
requireText(effects, 'if (index == 112)', 'Counter Shot effect projection');
requireText(coverage, '"id": "counter_shot"', 'Counter Shot coverage record');
requireText(coverage, '"m4_index": 112', 'Counter Shot coverage index');
requireText(coverage, '"id": "bladestorm"', 'source uninterruptible target coverage record');
requireText(coverage, '"m4_index": 113', 'source uninterruptible target coverage index');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/counter_shot_runtime_test_main.zr"') {
  throw new Error('WOS239 fixture must use only zr_vm:project');
}
requireText(runtime, 'counterShotRuntimeStateTest()', 'WOS239 runtime entry');
console.log('WOS239 Counter Shot static guard passed');
