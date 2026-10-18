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
const lifecycle = readPinned('src/sim/combat/casting_lifecycle.ts');
const dispatch = readPinned('src/sim/combat/effect_dispatch.ts');
const m4 = read('contracts/m4_abilities.json');
const state = read('scripts/woc_game/src/world/state.zr');
const fixture = read('scripts/woc_game/woc_m4_aimed_shot_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/aimed_shot_runtime_test_main.zr');

requireText(source, "id: 'aimed_shot'", 'source Aimed Shot identity');
requireText(source, "name: 'Long Draw'", 'source Aimed Shot name');
requireText(source, 'castTime: 3.0', 'source Aimed Shot cast time');
requireText(source, 'cooldown: 6', 'source Aimed Shot cooldown');
requireText(source, 'minRange: 8', 'source Aimed Shot minimum range');
requireText(source, 'scalesWith: \'ranged\'', 'source Aimed Shot ranged scaling');
requireText(source, "effects: [{ type: 'directDamage', min: 50, max: 62 }]",
  'source Aimed Shot direct-damage range');
requireText(lifecycle, 'const firesProjectile = ability.projectile ?? ability.school !== \'physical\';',
  'source physical projectile opt-in');
requireText(lifecycle, 'A physical shot has no resist roll; its hit/crit resolve inside runEffects.',
  'source physical projectile landing semantics');
requireText(dispatch, 'dmg += directHitBonus(abilityScalingPower(p, ability), ability, res.castTime);',
  'source resolved direct scaling');
requireText(m4, '"id": "aimed_shot"', 'M4 Aimed Shot entry');
requireText(m4, '"index": 107', 'M4 Aimed Shot index');
requireText(m4, `"source_commit": "${sourceCommit}"`, 'M4 source pin');
requireText(state, 'aimedShotAbilityCode()', 'Aimed Shot identity');
requireText(state, 'startOfflineAimedShotCast', 'Aimed Shot cast reducer');
requireText(state, 'completeOfflineAimedShotCast', 'Aimed Shot cast completion');
requireText(state, 'landOfflineAimedShotProjectile', 'Aimed Shot physical landing reducer');
requireText(state, 'mobSwing.armorReductionFromArmor(', 'Aimed Shot physical armor reducer');
requireText(state, 'aimedShotRuntimeStateTest', 'Aimed Shot runtime test');
requireText(state, 'near.entityFacing[0] = 3.14159265;', 'Aimed Shot facing rejection regression');
requireText(runtime, 'aimedShotRuntimeStateTest()', 'runtime entry');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/aimed_shot_runtime_test_main.zr"') {
  throw new Error('Aimed Shot fixture must use only zr_vm:project');
}
if (!existsSync(resolve(root, 'reference/current-head/m4_ability_projection_coverage.json'))) {
  throw new Error('M4 projection coverage is missing');
}
console.log('WOS233 Aimed Shot static guard passed');
