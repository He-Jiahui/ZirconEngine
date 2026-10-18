import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const helper = readProjectFile("scripts", "woc_game", "src", "combat", "temporal_hourglass_state.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "temporal_hourglass_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_temporal_hourglass_runtime_tests.zrp");

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

const ability = abilities.entries.find((entry) => entry.id === "temporal_hourglass");
assert.ok(ability);
assert.equal(ability.definition.class, "mage");
assert.deepEqual(ability.definition.specs, ["arcane"]);
assert.equal(ability.definition.cost, 110);
assert.equal(ability.definition.cooldown, 50);
assert.equal(ability.definition.range, 28);
assert.equal(ability.definition.targetMode, "position");
assert.deepEqual(ability.definition.effects, [{
  type: "temporalHourglass", duration: 5, hostilePveDuration: 60,
  hostilePvpDuration: 10, groundDuration: 30, selfRadius: 1.75,
  captureRadius: 1.75, healMaxHpPct: 0.3, selfCooldownRate: 2,
  allyCooldownRate: 1.75,
}]);
for (const name of [
  "temporalHourglassAbilityCode", "temporalHourglassPayloadAbilityIsExact",
  "temporalHourglassProfileIsValid", "startOfflineTemporalHourglassCast",
  "applyOfflineTemporalHourglass", "stepOfflineTemporalHourglassGround",
  "ageOfflineTemporalHourglass", "temporalHourglassRuntimeStateTest",
]) assert.match(world, new RegExp(`(?:pub )?${name}\\(`));
for (const field of [
  "entityTemporalHourglassRemaining", "entityTemporalHourglassModes",
  "entityTemporalHourglassCooldownRates", "entityTemporalHourglassHealRemaining",
  "entityTemporalHourglassHealTicksRemaining", "offlineTemporalHourglassSourceIds",
  "offlineTemporalHourglassProtectiveDurations", "offlineTemporalHourglassCreatedTicks",
]) assert.match(world, new RegExp(field));
assert.match(body("applySupportedCastAtCommand"), /temporalHourglassPayloadAbilityIsExact/);
assert.match(body("applyOfflineCancelAuraCommand"), /temporalHourglassPayloadAbilityIsExact/);
assert.match(body("applyOfflineTemporalHourglass"), /enterOfflineSpellProjectileCombat/);
assert.match(body("stepOfflineTemporalHourglassGround"), /offlineTemporalHourglassCreatedTicks/);
assert.match(body("stepOfflineTemporalHourglassGround"), /enterOfflineSpellProjectileCombat/);
assert.match(body("ageOfflineTemporalHourglass"), /temporalHourglass\.tickTemporalHourglassHealing/);
assert.match(body("temporalHourglassRuntimeStateTest"), /appendTypedCastAtCommandForTest/);
assert.match(body("temporalHourglassRuntimeStateTest"), /decodeState\(encodeState\(state\)\)/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /schemaVersion >= <uint>104/);
assert.match(world, /schemaVersion != <uint>118/);
assert.match(helper, /tickTemporalHourglassHealing|temporalHourglassCooldownDelta/);
assert.match(main, /state\.temporalHourglassRuntimeStateTest\(\)/);
assert.match(project, /backend\s*=\s*"zr_vm:project"/);
assert.match(project, /entry\s*=\s*"src\/world\/temporal_hourglass_runtime_test_main\.zr"/);
console.log("WOS201 Temporal Hourglass runtime static guards passed");
