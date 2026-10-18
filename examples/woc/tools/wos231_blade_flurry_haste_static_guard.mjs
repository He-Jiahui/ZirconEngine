import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (path) => readFileSync(resolve(root, path), 'utf8');
const requireText = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`missing ${label}: ${needle}`);
};

const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const sourceSim = read('../../dev/world-of-claudecraft/src/sim/sim.ts');
const m4 = read('contracts/m4_abilities.json');
const state = read('scripts/woc_game/src/world/state.zr');
const cc = read('reference/current-head/cc_contract.json');
const fixture = read('scripts/woc_game/woc_m4_blade_flurry_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/blade_flurry_runtime_test_main.zr');

requireText(source, "id: 'blade_flurry'", 'source Blade Flurry identity');
requireText(source, "kind: 'buff_haste', value: 1.2, duration: 12", 'source haste profile');
requireText(sourceSim, "if (a.kind === 'buff_haste') m /= a.value;", 'source haste fold');
requireText(m4, '"id": "blade_flurry"', 'M4 Blade Flurry entry');
requireText(m4, '"index": 105', 'M4 Blade Flurry index');
requireText(m4, '"cost": 25', 'M4 Blade Flurry cost');
requireText(m4, '"duration": 12', 'M4 Blade Flurry duration');
requireText(m4, '"value": 1.2', 'M4 Blade Flurry haste value');
requireText(cc, '"buff_haste": 18', 'CC buff_haste code');
requireText(state, 'bladeFlurryAbilityCode()', 'Blade Flurry identity');
requireText(state, 'bladeFlurryProfileIsValid', 'Blade Flurry profile');
requireText(state, 'startOfflineBladeFlurryCast', 'Blade Flurry reducer');
requireText(state, 'bladeFlurrySwingIntervalMultiplier', 'Blade Flurry cadence fold');
requireText(state, 'actor.swingIntervalMultiplier = bladeFlurrySwingIntervalMultiplier',
  'player auto cadence integration');
requireText(state, 'bladeFlurryRuntimeStateTest', 'Blade Flurry runtime test');
requireText(state, 'autoAttackState.fixedTick(meleeActor, target, meleeEvents)',
  'melee cadence regression');
requireText(state, 'autoAttackState.fixedTick(dualActor, target, dualEvents)',
  'offhand cadence regression');
requireText(state, 'autoAttackState.fixedTick(rangedActor, target, rangedEvents)',
  'ranged cadence regression');
requireText(state, 'appendCancelAuraCommandForTest(cancel, playerId, <uint>1, <uint>3, bytes)',
  'Blade Flurry cancellation regression');
requireText(runtime, 'bladeFlurryRuntimeStateTest()', 'runtime entry');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/blade_flurry_runtime_test_main.zr"') {
  throw new Error('Blade Flurry fixture must use only zr_vm:project');
}
if (!existsSync(resolve(root, 'reference/current-head/m4_ability_projection_coverage.json'))) {
  throw new Error('M4 projection coverage is missing');
}
console.log('WOS231 Blade Flurry static guard passed');
