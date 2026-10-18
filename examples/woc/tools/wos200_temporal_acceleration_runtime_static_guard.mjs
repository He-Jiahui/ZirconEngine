import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const group = readProjectFile("scripts", "woc_game", "src", "combat", "group_haste_state.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "temporal_acceleration_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_temporal_acceleration_runtime_tests.zrp");

function body(name) {
  const start = world.indexOf(`\n${name}(`) >= 0 ? world.indexOf(`\n${name}(`) :
    world.indexOf(`\npub ${name}(`);
  assert.notEqual(start, -1, `${name} is reachable`);
  const open = world.indexOf("{", start);
  let depth = 0;
  for (let i = open; i < world.length; i += 1) {
    if (world[i] === "{") depth += 1;
    if (world[i] === "}") depth -= 1;
    if (depth === 0) return world.slice(start, i + 1);
  }
  assert.fail(`${name} is unterminated`);
}

const ability = abilities.entries.find((entry) => entry.id === "temporal_acceleration");
assert.ok(ability);
assert.equal(ability.definition.class, "mage");
assert.deepEqual(ability.definition.specs, ["arcane"]);
assert.equal(ability.definition.cost, 120);
assert.equal(ability.definition.cooldown, 300);
assert.deepEqual(ability.definition.effects, [{
  type: "aoeAllyHaste", mult: 1.3, duration: 15, radius: 40,
  spell: true, exhaust: true, groupOnly: true,
}]);
for (const name of [
  "temporalAccelerationAbilityCode", "temporalAccelerationPayloadAbilityIsExact",
  "temporalAccelerationProfileIsValid", "startOfflineTemporalAccelerationCast",
  "groupHasteCandidateIndices", "ageOfflineGroupHaste",
  "temporalAccelerationRuntimeStateTest",
]) assert.match(world, new RegExp(`(?:pub )?${name}\\(`));
for (const field of [
  "entityGroupHasteRemaining", "entityGroupHasteAttackBonuses",
  "entityGroupHasteSpellBonuses", "entitySatedRemaining",
]) assert.match(world, new RegExp(field));
const start = body("startOfflineTemporalAccelerationCast");
assert.match(start, /groupHaste\.planGroupHaste/);
assert.match(start, /spendOfflineAbilityResource/);
assert.match(start, /state\.timeMicros \+ <uint>300000000/);
assert.doesNotMatch(start, /nextAuthoritativeRandomUnit/);
assert.match(body("mageSpellHasteMultiplier"), /groupHasteSpellBonus/);
assert.match(body("applyOfflineCancelAuraCommand"), /temporalAccelerationPayloadAbilityIsExact/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /schemaVersion >= <uint>101/);
assert.match(world, /schemaVersion != <uint>118/);
assert.match(group, /skip_sated_only_when_exhausting|planGroupHaste/);
assert.match(body("temporalAccelerationRuntimeStateTest"), /appendTypedCastCommandForTest/);
assert.match(body("temporalAccelerationRuntimeStateTest"), /appendCastSlotCommand/);
assert.match(main, /state\.temporalAccelerationRuntimeStateTest\(\)/);
assert.match(project, /backend\s*=\s*"zr_vm:project"/);
assert.match(project, /entry\s*=\s*"src\/world\/temporal_acceleration_runtime_test_main\.zr"/);
console.log("WOS200 Temporal Acceleration runtime static guards passed");
