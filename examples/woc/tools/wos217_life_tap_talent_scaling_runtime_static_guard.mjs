import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const source = read('../../dev/world-of-claudecraft/src/sim/content/classes.ts');
const dispatch = read('../../dev/world-of-claudecraft/src/sim/combat/effect_dispatch.ts');
const contract = JSON.parse(read('contracts/m4_abilities.json'));
const scaling = read('scripts/woc_game/src/combat/life_tap_talent_scaling_state.zr');
const state = read('scripts/woc_game/src/world/state.zr');
const testMain = read('scripts/woc_game/src/world/life_tap_talent_scaling_runtime_test_main.zr');
const manifest = read('scripts/woc_game/woc_m4_life_tap_talent_runtime_tests.zrp');

assert.match(source, /case 'lifeTap':\s*return \{ \.\.\.eff, mana: Math\.round\(eff\.mana \* dmgMult \+ flat\) \};/);
assert.match(source, /life_tap:[\s\S]*?school: 'shadow',[\s\S]*?effects: \[\{ type: 'lifeTap', hp: 30, mana: 30 \}\][\s\S]*?rank: 3, level: 20, cost: 0, effects: \[\{ type: 'lifeTap', hp: 85, mana: 85 \}\]/);
assert.match(dispatch, /case 'lifeTap':[\s\S]*?p\.hp <= eff\.hp[\s\S]*?p\.hp -= eff\.hp[\s\S]*?const tapMana = Math\.round\(eff\.mana \* \(1 \+ \(mods\.abilities\[ability\.id\]\?\.buffPct \?\? 0\)\)\)[\s\S]*?Math\.min\(p\.maxResource, p\.resource \+ tapMana\)/);

const lifeTap = contract.entries.find((entry) => entry.id === 'life_tap');
assert.ok(lifeTap, 'Life Tap must be generated into the M4 contract');
assert.equal(lifeTap.definition.school, 'shadow');
assert.deepEqual(lifeTap.definition.effects[0], { type: 'lifeTap', hp: 30, mana: 30 });

assert.match(scaling, /pub resolveLifeTapMana\([\s\S]*?school == "physical" \? "meleeDmgPct" : "spellDmgPct"[\s\S]*?"dmgPct"[\s\S]*?"flatDmg"[\s\S]*?"buffPct"/);
assert.match(scaling, /resolveLifeTapMana\(85, mixed, "shadow", "life_tap"\) == 136/);
assert.match(state, /var lifeTapTalentScaling = %import\("combat\/life_tap_talent_scaling_state"\);/);
assert.match(state, /startOfflineLifeTapCast[\s\S]*?var healthCost[\s\S]*?lifeTapTalentScaling\.resolveLifeTapMana\([\s\S]*?offlineTalentModifiersForEntity\(state, casterIndex\)[\s\S]*?entityHp\[casterIndex\] = [\s\S]*?offlineCappedResourceGain\(state, casterIndex/);
assert.match(state, /pub lifeTapTalentScalingRuntimeStateTest\(\): int/);
assert.match(state, /lifeTapTalentScalingRuntimeStateTest[\s\S]*?startOfflineLifeTapCast\(state, 0\)[\s\S]*?entityHp\[0\] != 115[\s\S]*?entityResources\[0\] != 85[\s\S]*?entityHp\[0\] == 85/);
assert.match(testMain, /state\.lifeTapTalentScalingRuntimeStateTest\(\)/);
assert.match(manifest, /backend = "zr_vm:project"/);

console.log('WOS217 Life Tap talent scaling static guard passed');
