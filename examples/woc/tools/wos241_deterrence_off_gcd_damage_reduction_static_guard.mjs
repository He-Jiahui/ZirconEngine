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
const damage = readPinned('src/sim/combat/damage.ts');
const state = read('scripts/woc_game/src/world/state.zr');
const catalog = read('scripts/woc_game/src/generated/m4_ability_catalog.zr');
const effects = read('scripts/woc_game/src/generated/m4_ability_effects.zr');
const ccContract = read('scripts/woc_game/src/generated/cc_contract.zr');
const coverage = read('reference/current-head/m4_ability_projection_coverage.json');
const m4Contract = read('contracts/m4_abilities.json');
const fixture = read('scripts/woc_game/woc_m4_deterrence_runtime_tests.zrp');
const runtime = read('scripts/woc_game/src/world/deterrence_runtime_test_main.zr');

requireText(talents, 'deterrence: {', 'source Deterrence identity');
requireText(talents, "name: 'Bristleguard'", 'source Deterrence name');
requireText(talents, 'learnLevel: 10', 'source Deterrence level');
requireText(talents, 'cost: 0', 'source Deterrence cost');
requireText(talents, 'castTime: 0', 'source Deterrence instant cast');
requireText(talents, 'cooldown: 300', 'source Deterrence cooldown');
requireText(talents, 'range: 0', 'source Deterrence self range');
requireText(talents, 'offGcd: true', 'source Deterrence off-GCD contract');
requireText(talents, "{ type: 'selfBuff', kind: 'buff_dodge', value: 0.25, duration: 10 }",
  'source Deterrence dodge effect');
requireText(talents, "{ type: 'selfBuff', kind: 'buff_dr', value: 0.3, duration: 10 }",
  'source Deterrence damage-reduction effect');
requireText(choices, "id: 'hun_r17_deterrence'", 'source Deterrence choice grant');
requireText(choices, "ability: 'deterrence'", 'source Deterrence grant target');
requireText(damage, "aura.kind === 'buff_dr' || aura.kind === 'die_by_sword'",
  'source all-school damage reduction kind');
requireText(damage, 'Math.round(amount * Math.max(0, 1 - reduction))',
  'source damage reduction rounding');

requireText(state, 'deterrenceAbilityCode', 'Deterrence catalog identity');
requireText(state, 'deterrencePayloadAbilityIsExact', 'Deterrence typed payload');
requireText(state, 'deterrenceProfileIsValid', 'Deterrence source profile');
requireText(state, 'retainedDamageReductionAuraProfileIsValid',
  'Deterrence retained damage-reduction profile');
requireText(state, 'applyOfflineRetainedDamageReduction',
  'Deterrence source-style incoming damage reduction');
requireText(state, 'startOfflineDeterrenceCast', 'Deterrence reducer');
requireText(state, 'deterrenceRuntimeStateTest', 'Deterrence runtime test');
const reducerStart = state.indexOf('startOfflineDeterrenceCast(');
const reducerEnd = state.indexOf('\nstartOfflineFrostTrapCast(', reducerStart);
if (reducerStart < 0 || reducerEnd < 0) {
  throw new Error('unable to isolate Deterrence reducer');
}
const reducer = state.slice(reducerStart, reducerEnd);
if (reducer.includes('entityCastGcdRemaining') || reducer.includes('spendOfflineAbilityResource') ||
    reducer.includes('appendOfflineAbilityProjectile') || reducer.includes('nextAuthoritativeRandomUnit')) {
  throw new Error('Deterrence must remain zero-cost, off-GCD, non-projectile, and non-random');
}
const cooldown = reducer.indexOf('setAbilityCooldownExpiration');
const dodgeAura = reducer.indexOf('ccContract.motionAuraKindCode("buff_dodge")');
const reductionAura = reducer.indexOf('ccContract.motionAuraKindCode("buff_dr")');
if (cooldown < 0 || dodgeAura < cooldown || reductionAura < dodgeAura) {
  throw new Error('Deterrence must arm cooldown, then apply dodge and damage-reduction auras');
}
const swingStart = state.indexOf('resolveOfflineEastbrookMobSwingRequests(');
const swingEnd = state.indexOf('\nstepOfflineEastbrookMobMeleePursuit(', swingStart);
if (swingStart < 0 || swingEnd < 0) throw new Error('unable to isolate Eastbrook mob swing bridge');
const swing = state.slice(swingStart, swingEnd);
const reduction = swing.indexOf('applyOfflineRetainedDamageReduction');
const absorb = swing.indexOf('applyOfflineAbsorbDamage');
if (reduction < 0 || absorb < reduction) {
  throw new Error('Deterrence damage reduction must apply before retained absorbs');
}
requireText(state, 'if (abilityCode == deterrenceAbilityCode())', 'Deterrence slot route');
requireText(state,
  'if (deterrencePayloadAbilityIsExact(payloadBytes, abilityOffset, abilityLength))',
  'Deterrence typed route');
requireText(catalog, 'if (id == "deterrence") { return 115; }', 'Deterrence catalog index');
requireText(effects, 'if (index == 115)', 'Deterrence effect projection');
requireText(effects, 'if (field == "kind") { return "buff_dr"; }',
  'Deterrence damage-reduction effect kind');
requireText(ccContract, 'if (kind == "buff_dr") { return <uint>20; }',
  'Deterrence motion aura kind');
requireText(coverage, '"id": "deterrence"', 'Deterrence coverage record');
requireText(coverage, '"m4_index": 115', 'Deterrence coverage index');
requireText(m4Contract,
  '"source_owner": "src/sim/content/talent_abilities_v2_a.ts#TALENT_ABILITIES_V2_A.deterrence"',
  'Deterrence direct source owner');
requireText(coverage,
  '"m4_source_owner": "src/sim/content/talent_abilities_v2_a.ts#TALENT_ABILITIES_V2_A.deterrence"',
  'Deterrence coverage source owner');
if (fixture.trim() !== 'backend = "zr_vm:project"\nentry = "src/world/deterrence_runtime_test_main.zr"') {
  throw new Error('WOS241 fixture must use only zr_vm:project');
}
requireText(runtime, 'deterrenceRuntimeStateTest()', 'WOS241 runtime entry');
console.log('WOS241 Deterrence static guard passed');
