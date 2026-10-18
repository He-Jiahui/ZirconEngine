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

const rows = readPinned('src/sim/content/choice_rows_classic.ts');
const lifecycle = readPinned('src/sim/combat/casting_lifecycle.ts');
const dispatch = readPinned('src/sim/combat/effect_dispatch.ts');
const state = read('scripts/woc_game/src/world/state.zr');
const charges = read('scripts/woc_game/src/combat/ability_charge_state.zr');
const effects = read('scripts/woc_game/src/combat/talent_added_effect_state.zr');
const fixture = read('scripts/woc_game/woc_m4_arcane_shot_choice_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/arcane_shot_choice_runtime_test_main.zr');

requireText(rows, "id: 'hun_r5_quick_shots'", 'source Quick Shots identity');
requireText(rows, "ability: 'arcane_shot', bonusCharges: 1", 'source Quick Shots charge');
requireText(rows, "id: 'hun_r14_serpents_venom'", 'source Serpents Venom identity');
requireText(rows, "directPct: 0.5", 'source Serpents Venom direct percentage');
requireText(rows, 'duration: 3', 'source Serpents Venom duration');
requireText(rows, 'interval: 1', 'source Serpents Venom interval');
requireText(rows, "school: 'nature'", 'source Serpents Venom school');
requireText(lifecycle, 'Parallel per-charge recharge: every spend starts ITS OWN timer.',
  'source parallel recharge');
requireText(dispatch, 'eff.directPct !== undefined && lastDirectDamage <= 0',
  'source resisted direct-pct gate');
requireText(dispatch, 'Math.round(lastDirectDamage * eff.directPct)',
  'source direct-pct total');
requireText(state, 'offlineAbilityChargeStateIsValid', 'serialized charge state');
requireText(state, 'stepOfflineAbilityCharges', 'parallel recharge tick');
requireText(state, 'armOfflineAbilityChargeCooldown', 'charge cooldown mirror');
requireText(state, 'applyOfflineArcaneShotAddedEffects', 'dynamic projectile add effects');
requireText(state, 'applyOfflineArcaneShotVenomDot', 'Serpents Venom reducer');
requireText(state, 'abilityCode == arcaneShotAbilityCode() ?\n                            m4AbilityCatalog.indexOf("arcane_shot") :\n                            pureDotAbilityIndex(abilityCode);',
  'Serpents Venom periodic threat route');
requireText(state, 'arcaneShotChoiceRuntimeStateTest', 'WOS236 runtime test');
requireText(charges, 'tickAbilityCharges', 'charge contract');
requireText(effects, 'resolveForAbility', 'dynamic effect projection');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/arcane_shot_choice_runtime_test_main.zr"') {
  throw new Error('WOS236 fixture must use only zr_vm:project');
}
requireText(runtime, 'arcaneShotChoiceRuntimeStateTest()', 'WOS236 runtime entry');
console.log('WOS236 Arcane Shot choice static guard passed');
