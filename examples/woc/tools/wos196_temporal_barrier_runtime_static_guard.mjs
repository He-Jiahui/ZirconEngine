import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const catalog = readProjectFile("scripts", "woc_game", "src", "generated", "m4_ability_catalog.zr");
const effects = readProjectFile("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
const profiles = readProjectFile("scripts", "woc_game", "src", "combat", "absorb_profile_state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const main = readProjectFile(
  "scripts", "woc_game", "src", "world", "temporal_barrier_runtime_test_main.zr"
);
const project = readProjectFile(
  "scripts", "woc_game", "woc_m4_temporal_barrier_runtime_tests.zrp"
);

function functionBody(name) {
  const privateStart = world.indexOf(`\n${name}(`);
  const publicStart = world.indexOf(`\npub ${name}(`);
  const start = privateStart === -1 ? publicStart :
    (publicStart === -1 ? privateStart : Math.min(privateStart, publicStart));
  assert.notEqual(start, -1, `${name} must remain reachable`);
  const open = world.indexOf("{", start);
  assert.notEqual(open, -1, `${name} must have a body`);
  let depth = 0;
  for (let index = open; index < world.length; index += 1) {
    if (world[index] === "{") depth += 1;
    if (world[index] === "}") depth -= 1;
    if (depth === 0) return world.slice(start, index + 1);
  }
  assert.fail(`${name} body is unterminated`);
}

assert.match(catalog, /"temporal_barrier"/);
assert.match(effects, /"absorb"/);
assert.match(profiles, /temporal_barrier/);
const barrierEntry = abilities.entries.find((entry) => entry.id === "temporal_barrier");
assert.ok(barrierEntry, "Temporal Barrier must remain in the generated M4 ability scope");
assert.equal(barrierEntry.definition.class, "mage");
assert.deepEqual(barrierEntry.definition.specs, ["arcane"]);
assert.equal(barrierEntry.definition.cost, 50);
assert.equal(barrierEntry.definition.cooldown, 12);
assert.equal(barrierEntry.definition.range, 30);
assert.deepEqual(barrierEntry.definition.effects, [{ type: "absorb", amount: 55, duration: 10 }]);
assert.deepEqual(barrierEntry.definition.ranks, [
  { rank: 2, level: 12, cost: 75, effects: [{ type: "absorb", amount: 100, duration: 10 }] },
  { rank: 3, level: 18, cost: 105, effects: [{ type: "absorb", amount: 160, duration: 10 }] },
]);

for (const name of [
  "temporalBarrierAbilityCode",
  "temporalBarrierPayloadAbilityIsExact",
  "temporalBarrierTargetIndex",
  "temporalBarrierProfileIsValid",
  "startOfflineTemporalBarrierCast",
  "temporalBarrierRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}

const barrier = functionBody("startOfflineTemporalBarrierCast");
assert.match(barrier, /m4AbilityCatalog\.indexOf\("temporal_barrier"\)/);
assert.match(barrier, /catalogAdmission\(state, casterIndex, abilityCode, "", false\)/);
assert.match(barrier, /temporalBarrierTargetIndex/);
assert.match(barrier, /absorbProfiles\.resolveAbsorbProfile/);
assert.match(barrier, /spendOfflineAbilityResource/);
assert.match(barrier, /setAbilityCooldownExpiration/);
assert.match(barrier, /applyOfflineFixedAbsorb/);
assert.doesNotMatch(barrier, /nextAuthoritativeRandomUnit/);

const slot = functionBody("applySupportedCastSlotCommand");
assert.match(slot, /temporalBarrierAbilityCode\(\)[\s\S]*?startOfflineTemporalBarrierCast/);
const typed = functionBody("applySupportedCastCommand");
assert.match(typed, /temporalBarrierPayloadAbilityIsExact[\s\S]*?startOfflineTemporalBarrierCast/);

assert.match(world, /offlineAbsorbProfileMatches[\s\S]*?temporalBarrierAbilityCode/);
assert.match(world, /offlineWardedPersonalBarrierIsActive[\s\S]*?temporalBarrierAbilityCode/);
assert.match(world, /applyOfflineWardedShieldConsumed[\s\S]*?temporalBarrierAbilityCode/);
assert.match(main, /state\.temporalBarrierRuntimeStateTest\(\)/);
assert.match(project, /backend = "zr_vm:project"/);
assert.match(project, /entry = "src\/world\/temporal_barrier_runtime_test_main\.zr"/);

console.log("WOS196 Temporal Barrier runtime static guards passed");
