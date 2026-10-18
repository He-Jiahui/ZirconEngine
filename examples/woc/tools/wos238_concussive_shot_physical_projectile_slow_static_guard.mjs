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
const fixture = read('scripts/woc_game/woc_m4_concussive_shot_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/concussive_shot_runtime_test_main.zr');

requireText(classes, 'concussive_shot: {', 'source Concussive Shot identity');
requireText(classes, "name: 'Rattling Shot'", 'source Concussive Shot name');
requireText(classes, 'learnLevel: 8', 'source Concussive Shot level');
requireText(classes, 'cost: 20', 'source Concussive Shot cost');
requireText(classes, 'cooldown: 12', 'source Concussive Shot cooldown');
requireText(classes, 'range: 35', 'source Concussive Shot range');
requireText(classes, 'minRange: 8', 'source Concussive Shot minimum range');
requireText(classes, "school: 'physical'", 'source physical school');
requireText(classes, 'projectile: true', 'source projectile admission');
requireText(classes, "scalesWith: 'ranged'", 'source ranged AP scaling');
requireText(classes, "{ type: 'directDamage', min: 4, max: 6 }", 'source direct damage');
requireText(classes, "{ type: 'slow', mult: 0.5, duration: 4 }", 'source slow');
requireText(lifecycle, 'if (ability.minRange && d < ability.minRange)',
  'source minimum range admission');
requireText(lifecycle, "const firesProjectile = ability.projectile ?? ability.school !== 'physical';",
  'source physical projectile dispatch');
requireText(dispatch, 'directHitBonus(abilityScalingPower(p, ability), ability, res.castTime)',
  'source direct damage scaling');
requireText(dispatch, 'if (!isSpell) dmg *= 1 - armorReduction(ctx.effectiveArmor(target), p.level);',
  'source physical armor branch');
requireText(dispatch, "case 'slow': {", 'source slow dispatch');
requireText(dispatch, 'if (!target || target.dead) break;', 'source surviving target slow gate');

requireText(state, 'startOfflineConcussiveShotCast', 'Concussive Shot reducer');
requireText(state, 'concussiveShotTargetIndex', 'Concussive Shot range admission');
requireText(state, 'concussiveShotPayloadAbilityIsExact', 'Concussive Shot typed cast');
requireText(state, 'concussiveShotProjectileProfileIsValid', 'Concussive Shot projectile snapshot');
requireText(state, 'landOfflineConcussiveShotProjectile', 'Concussive Shot projectile landing');
requireText(state, 'nextAuthoritativeRandomUnit(state)', 'land-time direct damage random draw');
requireText(state, 'offlinePhysicalCritical(state, casterIndex)', 'land-time physical critical draw');
requireText(state, 'state.entityRangedPower[casterIndex]', 'ranged AP resolution');
requireText(state, 'mobSwing.armorReductionFromArmor', 'physical armor reduction');
requireText(state, 'applyOfflineMotionAuraWithDetailsAndSchool', 'source-ordered slow aura');
requireText(state, 'concussiveShotRuntimeStateTest', 'Concussive Shot runtime test');
const concussiveLandingStart = state.indexOf('landOfflineConcussiveShotProjectile(');
const concussiveLandingEnd = state.indexOf('\nlandOfflineRangedProjectile(', concussiveLandingStart);
if (concussiveLandingStart < 0 || concussiveLandingEnd < 0) {
  throw new Error('unable to isolate Concussive Shot landing reducer');
}
const concussiveLanding = state.slice(concussiveLandingStart, concussiveLandingEnd);
const rangeRoll = concussiveLanding.indexOf('nextAuthoritativeRandomUnit(state)');
const criticalRoll = concussiveLanding.indexOf('offlinePhysicalCritical(state, casterIndex)');
if (rangeRoll < 0 || criticalRoll < 0 || rangeRoll >= criticalRoll) {
  throw new Error('Concussive Shot must draw direct-damage range before physical critical');
}
requireText(state, 'critical.entityCritChance[0] = 1.0;',
  'Concussive Shot forced physical-critical runtime case');
requireText(state, 'baselineDamage != zeroRapDamage + expectedRangedApBonus',
  'Concussive Shot exact ranged-AP coefficient runtime assertion');
requireText(state, 'criticalDamage != baselineDamage * 2',
  'Concussive Shot exact physical-critical multiplier runtime assertion');
requireText(state, 'lethal.entityMotionAuraAbilityCodes.length != 0',
  'Concussive Shot lethal-hit slow suppression assertion');
requireText(state, 'stateThreatValue(lethal, 1, playerId), 0.0',
  'Concussive Shot lethal-hit threat suppression assertion');
requireText(catalog, 'if (id == "concussive_shot") { return 111; }',
  'Concussive Shot catalog id');
requireText(effects, 'if (index == 111)', 'Concussive Shot effect projection');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/concussive_shot_runtime_test_main.zr"') {
  throw new Error('WOS238 fixture must use only zr_vm:project');
}
requireText(runtime, 'concussiveShotRuntimeStateTest()', 'WOS238 runtime entry');
console.log('WOS238 Concussive Shot static guard passed');
