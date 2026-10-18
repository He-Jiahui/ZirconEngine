import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const contract = JSON.parse(readProjectFile("reference", "current-head", "rewind_contract.json"));
const generated = readProjectFile("scripts", "woc_game", "src", "generated", "rewind_contract.zr");
const effects = readProjectFile("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
const history = readProjectFile("scripts", "woc_game", "src", "combat", "damage_history_state.zr");
const rewind = readProjectFile("scripts", "woc_game", "src", "combat", "rewind_state.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "rewind_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_rewind_runtime_tests.zrp");

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

assert.equal(contract.ability.id, "temporal_rewind");
assert.equal(contract.ability.class, "mage");
assert.equal(contract.ability.learn_level, 14);
assert.equal(contract.ability.resource_cost, 150);
assert.equal(contract.ability.cast_time_seconds, 0);
assert.equal(contract.ability.cooldown_seconds, 120);
assert.equal(contract.ability.range, 0);
assert.equal(contract.ability.requires_target, false);
assert.equal(contract.ability.radius, 40);
assert.equal(contract.healing.fraction, 0.3);
assert.equal(contract.healing.max_hp_fraction, 0.35);
assert.match(generated, /pub resourceCost\(required: bool\): int \{ return required \? 150 : 0; \}/);
assert.match(generated, /pub radius\(required: bool\): float \{ return required \? 40\.0 : 0\.0; \}/);
assert.match(history, /recordDamageTaken/);
assert.match(rewind, /rewindHealAmount/);
assert.equal(abilities.entries.length, 117);
const rewindAbility = abilities.entries.find((entry) => entry.id === "temporal_rewind");
assert.ok(rewindAbility);
assert.deepEqual(rewindAbility.definition, {
  id: "temporal_rewind", name: "Rewind", class: "mage", learnLevel: 14,
  specs: ["arcane"], cost: 150, castTime: 0, cooldown: 120, range: 0,
  school: "arcane", requiresTarget: false,
  effects: [{ type: "rewind", fraction: 0.3, maxHpFraction: 0.35, windowSec: 5, radius: 40 }],
  description: "Sends an arcane wave through your group or raid, rewinding time to restore 30% of the damage each ally within 40 yards took over the last 5 seconds (up to 35% of their maximum health). Cannot be a critical effect. (Chronomancy)",
});
assert.match(effects, /field == "fraction"/);
assert.match(effects, /field == "maxHpFraction"/);
assert.match(effects, /field == "windowSec"/);

for (const name of [
  "temporalRewindAbilityCode", "temporalRewindProfileIsValid",
  "temporalRewindTargetIndices", "recordOfflineRewindDamage",
  "offlineRewindDamageTaken", "offlineRewindDamageStateIsValid",
  "startOfflineTemporalRewindCast", "applyOfflineTemporalRewind",
  "temporalRewindRuntimeStateTest",
]) assert.match(world, new RegExp(`(?:pub )?${name}\\(`));

for (const field of [
  "offlineRewindDamageTargetIds", "offlineRewindDamageTicks", "offlineRewindDamageAmounts",
]) assert.match(world, new RegExp(field));
assert.match(body("recordOfflineRewindDamage"), /applyOfflineRewindDamagePrune/);
assert.match(body("applyOfflineTemporalRewind"), /applyOfflineResolvedDirectHeal/);
assert.doesNotMatch(body("applyOfflineTemporalRewind"), /nextAuthoritativeRandomUnit/);
assert.match(body("startOfflineTemporalRewindCast"), /spendOfflineAbilityResource/);
assert.match(body("startOfflineTemporalRewindCast"), /setAbilityCooldownExpiration/);
assert.match(body("applySupportedCastCommand"), /temporalRewindAbilityCode\(\)/);
assert.match(body("applySupportedCastSlotCommand"), /temporalRewindAbilityCode\(\)/);
const runtimeTest = body("temporalRewindRuntimeStateTest");
assert.match(runtimeTest, /applyOfflineArcaneEffectiveDamage/);
assert.match(runtimeTest, /applyOfflinePowerWordShield/);
assert.doesNotMatch(runtimeTest, /recordOfflineRewindDamage\(/);
assert.match(runtimeTest, /fullyAbsorbed != 0/);
assert.match(runtimeTest, /appendTypedCastCommandForTest/);
assert.match(runtimeTest, /appendCastSlotCommand/);
assert.match(runtimeTest, /applyOfflineMotionAuraWithDetailsAndSchool/);
assert.match(runtimeTest, /entityHp\[3\] != 1000/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /schemaVersion >= <uint>105/);
assert.match(main, /state\.temporalRewindRuntimeStateTest\(\)/);
assert.match(project, /"backend": "zr_vm:project"/);
console.log("WOS203 Rewind runtime static guards passed");
