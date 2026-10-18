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
  if (result.status !== 0) throw new Error(`unable to read source-pinned ${path}`);
  return result.stdout;
};
const requireText = (text, needle, label) => {
  if (!text.includes(needle)) throw new Error(`missing ${label}: ${needle}`);
};

const source = readPinned('src/sim/content/classes.ts');
const lifecycle = readPinned('src/sim/combat/casting_lifecycle.ts');
const dispatch = readPinned('src/sim/combat/effect_dispatch.ts');
const m4 = read('contracts/m4_abilities.json');
const state = read('scripts/woc_game/src/world/state.zr');
const fixture = read('scripts/woc_game/woc_m4_arcane_shot_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/arcane_shot_runtime_test_main.zr');

requireText(source, "id: 'arcane_shot'", 'source Arcane Shot identity');
requireText(source, "name: 'Fell Shot'", 'source Arcane Shot name');
requireText(source, 'minRange: 8', 'source Arcane Shot minimum range');
requireText(source, "school: 'arcane'", 'source Arcane Shot school');
requireText(source, "scalesWith: 'ranged'", 'source Arcane Shot ranged scaling');
requireText(lifecycle, "const firesProjectile = ability.projectile ?? ability.school !== 'physical';",
  'source nonphysical projectile convention');
requireText(dispatch, 'dmg += directHitBonus(abilityScalingPower(p, ability), ability, res.castTime);',
  'source direct scaling');
requireText(m4, '"id": "arcane_shot"', 'M4 Arcane Shot entry');
requireText(m4, '"index": 109', 'M4 Arcane Shot index');
requireText(m4, `"source_commit": "${sourceCommit}"`, 'M4 source pin');
requireText(state, 'arcaneShotAbilityCode()', 'Arcane Shot identity');
requireText(state, 'startOfflineArcaneShotCast', 'Arcane Shot cast reducer');
requireText(state, 'landOfflineArcaneShotProjectile', 'Arcane Shot landing reducer');
requireText(state, 'arcaneShotRuntimeStateTest', 'Arcane Shot runtime test');
requireText(runtime, 'arcaneShotRuntimeStateTest()', 'runtime entry');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/arcane_shot_runtime_test_main.zr"') {
  throw new Error('Arcane Shot fixture must use only zr_vm:project');
}
if (!existsSync(resolve(root, 'reference/current-head/m4_ability_projection_coverage.json'))) {
  throw new Error('M4 projection coverage is missing');
}
console.log('WOS235 Arcane Shot static guard passed');
