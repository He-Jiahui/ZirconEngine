import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const talentSource = read('../../dev/world-of-claudecraft/src/sim/content/talents_classic.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/gain_resource_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const testMain = read('scripts/woc_game/src/world/gain_resource_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_gain_resource_talent_runtime_tests.zrp');

assert.match(source, /case 'gainResource':\s*return \{ \.\.\.eff, amount: Math\.round\(eff\.amount \* dmgMult \+ flat\) \};/);
assert.match(source, /enrage:[\s\S]*?school: 'physical',[\s\S]*?effects: \[\{ type: 'gainResource', amount: 20 \}\]/);
assert.match(talentSource, /'feral',[\s\S]*?global: \{ meleeDmgPct: 0\.15, dotDmgPct: 0\.15/);

const idsFor = (type) => contract.entries
  .filter((entry) => entry.definition.effects.some((effect) => effect.type === type))
  .map((entry) => entry.id)
  .sort();
assert.deepEqual(idsFor('gainResource'), ['adrenaline_rush', 'berserker_rage', 'enrage']);

assert.match(scaling, /pub resolveGainResourceAmount\([\s\S]*?school == "physical" \? "meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"[\s\S]*?"flatDmg"/);
assert.match(scaling, /resolveGainResourceAmount\(20, mixed, "physical", "enrage"\) == 27[\s\S]*?resolveGainResourceAmount\(20, mixed, "fire", "enrage"\) == 28/);

assert.match(state, /var gainResourceTalentScaling = %import\("combat\/gain_resource_talent_scaling_state"\);/);
assert.match(state, /startOfflineEnrageCast[\s\S]*?gainResourceTalentScaling\.resolveGainResourceAmount\([\s\S]*?offlineTalentModifiersForEntity\(state, casterIndex\)[\s\S]*?m4AbilityCatalog\.text\(abilityIndex, "school"\)[\s\S]*?offlineCappedResourceGain\(state, casterIndex/);
assert.match(state, /pub gainResourceTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /gainResourceTalentScalingRuntimeStateTest[\s\S]*?replaceOfflineTalentAllocation\([\s\S]*?startOfflineEnrageCast\(state, 0\)[\s\S]*?entityResources\[0\] != 23[\s\S]*?replaceOfflineTalentAllocation\([\s\S]*?entityResources\[0\] == 20/);
assert.match(testMain, /state\.gainResourceTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS215 gainResource talent scaling static guard passed');
