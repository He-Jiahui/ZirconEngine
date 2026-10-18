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

const talents = readPinned('src/sim/content/talent_abilities_v2_a.ts');
const choices = readPinned('src/sim/content/choice_rows_classic.ts');
const lifecycle = readPinned('src/sim/combat/casting_lifecycle.ts');
const dispatch = readPinned('src/sim/combat/effect_dispatch.ts');
const state = read('scripts/woc_game/src/world/state.zr');
const catalog = read('scripts/woc_game/src/generated/m4_ability_catalog.zr');
const effects = read('scripts/woc_game/src/generated/m4_ability_effects.zr');
const coverage = read('reference/current-head/m4_ability_projection_coverage.json');
const m4Contract = read('contracts/m4_abilities.json');
const fixture = read('scripts/woc_game/woc_m4_startle_shot_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/startle_shot_runtime_test_main.zr');

requireText(talents, 'startle_shot: {', 'source Startle Shot identity');
requireText(talents, "name: 'Startle Shot'", 'source Startle Shot name');
requireText(talents, 'learnLevel: 8', 'source Startle Shot level');
requireText(talents, 'cost: 25', 'source Startle Shot cost');
requireText(talents, 'castTime: 0', 'source Startle Shot instant cast');
requireText(talents, 'cooldown: 20', 'source Startle Shot cooldown');
requireText(talents, 'range: 35', 'source Startle Shot range');
requireText(talents, 'minRange: 8', 'source Startle Shot minimum range');
requireText(talents, "school: 'physical'", 'source Startle Shot school');
requireText(talents, "scalesWith: 'ranged'", 'source Startle Shot scaling class');
requireText(talents, 'requiresTarget: true', 'source Startle Shot target requirement');
requireText(talents, "{ type: 'incapacitate', duration: 4 }", 'source Startle Shot effect');
requireText(choices, "id: 'hun_r8_startle_shot'", 'source Startle Shot choice grant');
requireText(choices, "ability: 'startle_shot'", 'source Startle Shot grant target');
requireText(lifecycle, 'if (ability.minRange && d < ability.minRange)',
  'source minimum range admission');
requireText(lifecycle, 'const firesProjectile = ability.projectile ?? ability.school !== \'physical\';',
  'source physical non-projectile dispatch');
requireText(dispatch, "case 'incapacitate': {", 'source incapacitate dispatch');
requireText(dispatch, 'breaksOnDamage: true', 'source damage-breakable incapacitate');
requireText(dispatch, 'breakChanceScale: ability.fearDr ? FEAR_BREAK_CHANCE_SCALE : undefined',
  'source plain incap has no graded break chance');

requireText(state, 'startleShotAbilityCode', 'Startle Shot catalog identity');
requireText(state, 'startleShotPayloadAbilityIsExact', 'Startle Shot typed payload');
requireText(state, 'startleShotProfileIsValid', 'Startle Shot source profile');
requireText(state, 'startleShotTargetIndex', 'Startle Shot range admission');
requireText(state, 'startOfflineStartleShotCast', 'Startle Shot reducer');
requireText(state, 'startleShotRuntimeStateTest', 'Startle Shot runtime test');
requireText(state, 'startleShotCode = startleShotAbilityCode()',
  'Startle Shot damage-breakable incap registration');
requireText(state, 'abilityCode != startleShotCode',
  'Startle Shot damage-breakable incap filter');
requireText(state, 'abilityCode == startleShotCode',
  'Startle Shot damage-breakable incap removal');
const reducerStart = state.indexOf('startOfflineStartleShotCast(');
const reducerEnd = state.indexOf('\nwingClipAbilityCode()', reducerStart);
if (reducerStart < 0 || reducerEnd < 0) {
  throw new Error('unable to isolate Startle Shot reducer');
}
const reducer = state.slice(reducerStart, reducerEnd);
const spend = reducer.indexOf('spendOfflineAbilityResource');
const aura = reducer.indexOf('applyOfflineMotionAuraWithDetailsAndSchool');
const cooldown = reducer.indexOf('setAbilityCooldownExpiration');
if (spend < 0 || aura < spend || cooldown < aura) {
  throw new Error('Startle Shot must spend, apply incap, then arm cooldown');
}
if (reducer.includes('appendOfflineAbilityProjectile') ||
    reducer.includes('nextAuthoritativeRandomUnit') ||
    reducer.includes('offlinePhysicalCritical') || reducer.includes('effectiveOfflineArmor')) {
  throw new Error('Startle Shot must not enter projectile, RNG, crit, or armor paths');
}
requireText(state, 'if (abilityCode == startleShotAbilityCode())', 'Startle Shot slot route');
requireText(state, 'if (startleShotPayloadAbilityIsExact(payloadBytes, abilityOffset, abilityLength))',
  'Startle Shot typed route');
requireText(catalog, 'if (id == "startle_shot") { return 114; }',
  'Startle Shot catalog index');
requireText(effects, 'if (index == 114)', 'Startle Shot effect projection');
requireText(coverage, '"id": "startle_shot"', 'Startle Shot coverage record');
requireText(coverage, '"m4_index": 114', 'Startle Shot coverage index');
requireText(m4Contract,
  '"source_owner": "src/sim/content/talent_abilities_v2_a.ts#TALENT_ABILITIES_V2_A.startle_shot"',
  'Startle Shot direct source owner');
requireText(coverage,
  '"m4_source_owner": "src/sim/content/talent_abilities_v2_a.ts#TALENT_ABILITIES_V2_A.startle_shot"',
  'Startle Shot coverage source owner');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/startle_shot_runtime_test_main.zr"') {
  throw new Error('WOS240 fixture must use only zr_vm:project');
}
requireText(runtime, 'startleShotRuntimeStateTest()', 'WOS240 runtime entry');
console.log('WOS240 Startle Shot static guard passed');
