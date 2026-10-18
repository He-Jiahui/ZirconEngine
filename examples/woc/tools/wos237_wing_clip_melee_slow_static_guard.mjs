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
const resolution = read('scripts/woc_game/src/combat/wing_clip_resolution.zr');
const catalog = read('scripts/woc_game/src/generated/m4_ability_catalog.zr');
const effects = read('scripts/woc_game/src/generated/m4_ability_effects.zr');
const fixture = read('scripts/woc_game/woc_m4_wing_clip_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/wing_clip_runtime_test_main.zr');

requireText(classes, "wing_clip: {", 'source Wing Clip identity');
requireText(classes, "name: 'Fettering Slash'", 'source Wing Clip name');
requireText(classes, "learnLevel: 10", 'source Wing Clip level');
requireText(classes, "cost: 20", 'source Wing Clip cost');
requireText(classes, "school: 'physical'", 'source Wing Clip school');
requireText(classes, "{ type: 'directDamage', min: 3, max: 5 }", 'source direct damage');
requireText(classes, "{ type: 'slow', mult: 0.6, duration: 10 }", 'source slow');
requireText(lifecycle, 'const maxRange = ability.range > 0 ? ability.range : MELEE_RANGE;',
  'source zero-range melee admission');
requireText(dispatch, 'if (!isSpell) dmg *= 1 - armorReduction(ctx.effectiveArmor(target), p.level);',
  'source physical armor branch');
requireText(dispatch, "case 'slow': {", 'source slow dispatch');
requireText(state, 'startOfflineWingClipCast', 'Wing Clip reducer');
requireText(state, 'wingClipTargetIndex', 'Wing Clip melee admission');
requireText(state, 'wingClipPayloadAbilityIsExact', 'Wing Clip typed cast');
requireText(state, 'wingClipResolution.resolveDirectDamage', 'Wing Clip direct resolution');
requireText(state, 'OFFLINE_PROJECTILE_SCHOOL_PHYSICAL', 'Wing Clip physical aura school');
requireText(state, 'wingClipRuntimeStateTest', 'Wing Clip runtime test');
requireText(resolution, 'spellScaling.directHitBonus(attackPower, "physical", false, castTime, false)',
  'physical direct AP scaling');
requireText(resolution, 'mobSwing.armorReductionFromArmor', 'physical armor reduction');
requireText(catalog, 'if (id == "wing_clip") { return 110; }', 'Wing Clip catalog id');
requireText(effects, 'if (index == 110)', 'Wing Clip effect projection');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/wing_clip_runtime_test_main.zr"') {
  throw new Error('WOS237 fixture must use only zr_vm:project');
}
requireText(runtime, 'wingClipRuntimeStateTest()', 'WOS237 runtime entry');
console.log('WOS237 Wing Clip static guard passed');
