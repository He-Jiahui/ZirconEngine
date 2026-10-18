import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (path) => readFileSync(resolve(root, path), 'utf8');
const sourceRoot = resolve(root, '..', '..', 'dev', 'world-of-claudecraft');
const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const readPinned = (path) => {
  const result = spawnSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], {
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    throw new Error(`unable to read source-pinned ${path}: ${result.stderr.trim()}`);
  }
  return result.stdout;
};
const requireText = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`missing ${label}: ${needle}`);
};

const source = readPinned('src/sim/content/classes.ts');
const sourceSim = readPinned('src/sim/sim.ts');
const m4 = read('contracts/m4_abilities.json');
const state = read('scripts/woc_game/src/world/state.zr');
const fixture = read('scripts/woc_game/woc_m4_rapid_fire_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/rapid_fire_runtime_test_main.zr');

requireText(source, "id: 'rapid_fire'", 'source Rapid Fire identity');
requireText(source, "name: 'Fevered Draw'", 'source Rapid Fire name');
requireText(source, 'cooldown: 300', 'source Rapid Fire cooldown');
requireText(source, 'offGcd: true', 'source Rapid Fire off-GCD flag');
requireText(source, "kind: 'buff_haste', value: 1.4, duration: 15", 'source Rapid Fire haste profile');
requireText(sourceSim, "if (a.kind === 'buff_haste') m /= a.value;", 'source swing haste fold');
requireText(m4, '"id": "rapid_fire"', 'M4 Rapid Fire entry');
requireText(m4, '"index": 108', 'M4 Rapid Fire index');
requireText(m4, '"cooldown": 300', 'M4 Rapid Fire cooldown');
requireText(m4, `"source_commit": "${sourceCommit}"`, 'M4 source pin');
requireText(state, 'rapidFireAbilityCode()', 'Rapid Fire identity');
requireText(state, 'rapidFireProfileIsValid', 'Rapid Fire profile');
requireText(state, 'startOfflineRapidFireCast', 'Rapid Fire reducer');
requireText(state, 'offlineSwingHasteAuraProfileIsValid', 'identity-aware haste profile');
requireText(state, 'offlineSwingIntervalMultiplier', 'composed swing cadence fold');
requireText(state, 'actor.swingIntervalMultiplier = bladeFlurrySwingIntervalMultiplier',
  'player auto cadence integration');
requireText(state, 'rapidFireRuntimeStateTest', 'Rapid Fire runtime test');
requireText(state, 'state.entityCastGcdRemaining[0] = 1.25;',
  'Rapid Fire off-GCD regression');
requireText(state, '1.0 / (1.4 * 1.2)', 'Rapid Fire and Blade Flurry composition regression');
requireText(state, 'autoAttackState.fixedTick(meleeActor, target, meleeEvents)',
  'Rapid Fire melee cadence regression');
requireText(state, 'autoAttackState.fixedTick(rangedActor, target, rangedEvents)',
  'Rapid Fire ranged cadence regression');
requireText(state, 'decodeState(encodeState(state))', 'Rapid Fire serialization regression');
requireText(state, 'startOfflineRapidFireCast(lowLevel, 0)',
  'Rapid Fire learn-level regression');
requireText(state, 'wrongClass.entityTemplates[0] = <uint>(rogue + 1);',
  'Rapid Fire class-admission regression');
requireText(runtime, 'rapidFireRuntimeStateTest()', 'runtime entry');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/rapid_fire_runtime_test_main.zr"') {
  throw new Error('Rapid Fire fixture must use only zr_vm:project');
}
if (!existsSync(resolve(root, 'reference/current-head/m4_ability_projection_coverage.json'))) {
  throw new Error('M4 projection coverage is missing');
}
console.log('WOS234 Rapid Fire static guard passed');
