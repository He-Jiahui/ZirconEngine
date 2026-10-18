import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const echo = readProjectFile("scripts", "woc_game", "src", "combat", "temporal_echo_state.zr");
const numeric = readProjectFile("scripts", "woc_game", "src", "combat", "effect_numeric_dispatch_state.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "temporal_cascade_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_temporal_cascade_runtime_tests.zrp");

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

const ability = abilities.entries.find((entry) => entry.id === "temporal_cascade");
assert.ok(ability);
assert.equal(ability.definition.class, "mage");
assert.deepEqual(ability.definition.specs, ["arcane"]);
assert.equal(ability.definition.learnLevel, 12);
assert.equal(ability.definition.cost, 90);
assert.equal(ability.definition.castTime, 2);
assert.equal(ability.definition.cooldown, 17);
assert.equal(ability.definition.range, 30);
assert.equal(ability.definition.partyOnlyTarget, true);
assert.deepEqual(ability.definition.effects, [{
  type: "massTemporalEcho", duration: 10, radius: 15, maxTargets: 5,
  heal: { min: 14, max: 18 },
}]);

for (const name of [
  "temporalCascadeAbilityCode", "temporalCascadePayloadAbilityIsExact",
  "temporalCascadeProfileIsValid", "temporalCascadeTargetIndex",
  "temporalCascadeTargetIndices", "startOfflineTemporalCascadeCast",
  "completeOfflineTemporalCascadeCast", "applyOfflineTemporalCascadeGroupEcho",
  "temporalCascadeRuntimeStateTest",
]) assert.match(world, new RegExp(`(?:pub )?${name}\\(`));

for (const field of [
  "offlineTemporalEchoGroups", "offlineTemporalEchoConvertRates",
]) assert.match(world, new RegExp(field));
assert.match(body("applyOfflineTemporalEchoDamageConversion"), /offlineTemporalEchoGroups/);
assert.match(body("applyOfflineTemporalEchoMark"), /stripOfflineIndividualTemporalEchoesBySource/);
assert.match(body("offlineTemporalEchoStateIsValid"), /var prior = 0/);
assert.match(body("offlineTemporalEchoStateIsValid"), /offlineTemporalEchoTargetIds\[prior\].*targetId/s);
assert.match(body("startOfflineTemporalCascadeCast"), /cast\.armTimed/);
assert.match(body("completeOfflineTemporalCascadeCast"), /applyOfflineDirectHeal/);
assert.doesNotMatch(body("completeOfflineTemporalCascadeCast"), /applyOfflinePowerEchoHeal/);
assert.match(body("completeOfflineTemporalCascadeCast"), /setAbilityCooldownExpiration/);
assert.match(body("completeOfflineAetherSurgeCast"), /scaledRawDamage, hp/);
assert.match(body("landOfflineFixedSpellProjectile"), /applyOfflineArcaneEffectiveDamage/);
const fixedImpact = body("landOfflineFixedSpellProjectile");
assert.match(
  fixedImpact,
  /applyOfflineTemporalEchoDamageConversion\(\s*state, casterIndex, dealt,\s*OFFLINE_PROJECTILE_SCHOOL_ARCANE, false\s*\)/,
);
assert.equal(
  fixedImpact.match(/applyOfflineTemporalEchoDamageConversion\(/g)?.length,
  1,
  "fixed Arcane impact converts its actual dealt damage exactly once",
);
assert.match(body("temporalCascadeRuntimeStateTest"), /applyOfflinePowerWordShield/);
assert.match(body("temporalCascadeRuntimeStateTest"), /entityHp\[1\] != 11/);
assert.match(body("applySupportedCastCommand"), /temporalCascadePayloadAbilityIsExact/);
assert.match(body("applySupportedCastSlotCommand"), /temporalCascadeAbilityCode\(\)/);
assert.match(numeric, /effectType == "massTemporalEcho"/);
assert.match(echo, /placeGroupEcho|groupSingleRate/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /schemaVersion >= <uint>104/);
assert.match(main, /state\.temporalCascadeRuntimeStateTest\(\)/);
assert.match(project, /"entry": "world\/temporal_cascade_runtime_test_main"/);
console.log("WOS202 Temporal Cascade runtime static guards passed");
