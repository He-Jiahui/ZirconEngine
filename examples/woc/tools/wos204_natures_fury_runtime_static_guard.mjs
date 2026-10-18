import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const fury = JSON.parse(readProjectFile("reference", "current-head", "natures_fury_contract.json"));
const generated = readProjectFile("scripts", "woc_game", "src", "generated", "natures_fury_contract.zr");
const modifiers = readProjectFile("scripts", "woc_game", "src", "generated", "talent_modifier_catalog.zr");
const selection = readProjectFile("scripts", "woc_game", "src", "generated", "current_talent_selection_catalog.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "natures_fury_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_natures_fury_runtime_tests.zrp");

function body(name) {
  const start = world.indexOf(`\n${name}(`) >= 0 ? world.indexOf(`\n${name}(`) :
    world.indexOf(`\npub ${name}(`);
  assert.notEqual(start, -1, `${name} is reachable`);
  const open = world.indexOf("{", start);
  let depth = 0;
  for (let index = open; index < world.length; index += 1) {
    if (world[index] === "{") depth += 1;
    if (world[index] === "}") depth -= 1;
    if (depth === 0) return world.slice(start, index + 1);
  }
  assert.fail(`${name} is unterminated`);
}

assert.equal(fury.radius, 30);
assert.equal(fury.ticks, 20);
assert.equal(fury.refresh, 3);
assert.match(generated, /pub radius\(\):float\{return 30\.0;\}/);
assert.match(generated, /pub pulseTicks\(\):int\{return 20;\}/);
assert.match(generated, /pub refreshSeconds\(\):float\{return 3\.0;\}/);
assert.match(selection, /dru_r20_improved_hurricane/);
assert.match(modifiers, /moonwingPartyCritPct/);
for (const name of [
  "offlineNaturesFuryStateIsValid", "offlineNaturesFurySpellCritBonus",
  "offlineSpellCritChance", "ageOfflineNaturesFury", "stepOfflineNaturesFury",
  "naturesFuryRuntimeStateTest",
]) assert.match(world, new RegExp(`(?:pub )?${name}\\(`));
for (const field of [
  "offlineNaturesFuryTargetIds", "offlineNaturesFurySourceIds",
  "offlineNaturesFuryValues", "offlineNaturesFuryRemaining",
]) assert.match(world, new RegExp(field));
assert.match(body("stepOfflineNaturesFury"), /form_moonkin/);
assert.match(body("stepOfflineNaturesFury"), /moonwingPartyCritPct/);
assert.match(body("stepOfflineNaturesFury"), /state\.tick %/);
assert.match(body("stepOfflineNaturesFury"), /entityPartyIds/);
assert.doesNotMatch(body("stepOfflineNaturesFury"), /nextAuthoritativeRandomUnit/);
assert.match(body("offlineSpellCritChance"), /offlineNaturesFurySpellCritBonus/);
assert.match(body("ageOfflineNaturesFury"), /OFFLINE_PERIODIC_TICK_EPS/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(body("decodeState"), /schemaVersion >= <uint>106[\s\S]*?offlineNaturesFuryTargetIds/);
assert.match(
  body("evaluateFixedTick"),
  /ageOfflineNaturesFury\(state\);[\s\S]*?stepRetainedPlayerTicks\(state\);[\s\S]*?stepOfflineNaturesFury\(state\);/,
);
assert.match(world, /initialBytes\.length != 501/);
assert.doesNotMatch(
  world,
  /spell\.critChance = <float>state\.entityCritChance\[casterIndex\]|source\.spellCritChance = <float>state\.entityCritChance\[\w+Index\]/,
);
assert.match(body("naturesFuryRuntimeStateTest"), /replaceOfflineTalentAllocation/);
assert.match(body("naturesFuryRuntimeStateTest"), /entityActiveFormAbilityCodes/);
assert.match(body("naturesFuryRuntimeStateTest"), /decodeState\(encodeState\(state\)\)/);
assert.match(body("naturesFuryRuntimeStateTest"), /encodeStateVersion\(legacy, <uint>105\)/);
assert.match(body("naturesFuryRuntimeStateTest"), /offlineRewindDamageAmounts\[0\] != 7/);
assert.match(body("naturesFuryRuntimeStateTest"), /applyOfflineNaturesFury\(restored, 1, 1, 0\.02, 1\.0\)/);
assert.match(body("naturesFuryRuntimeStateTest"), /applyOfflineNaturesFury\(restored, 1, 1, 0\.02, 2\.0\)/);
assert.match(body("naturesFuryRuntimeStateTest"), /secondSourceAgeTicks < 40/);
assert.match(main, /state\.naturesFuryRuntimeStateTest\(\)/);
assert.match(project, /"backend": "zr_vm:project"/);
console.log("WOS204 Nature's Fury runtime static guards passed");
