import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const decision = readProjectFile("scripts", "woc_game", "src", "combat", "aether_surge_state.zr");
const main = readProjectFile(
  "scripts", "woc_game", "src", "world", "aether_surge_charge_runtime_test_main.zr"
);
const project = readProjectFile(
  "scripts", "woc_game", "woc_m4_aether_surge_charge_runtime_tests.zrp"
);

function functionBody(name) {
  const privateStart = world.indexOf(`\n${name}(`);
  const publicStart = world.indexOf(`\npub ${name}(`);
  const start = privateStart === -1 ? publicStart :
    (publicStart === -1 ? privateStart : Math.min(privateStart, publicStart));
  assert.notEqual(start, -1, `${name} must remain reachable`);
  const open = world.indexOf("{", start);
  let depth = 0;
  for (let index = open; index < world.length; index += 1) {
    if (world[index] === "{") depth += 1;
    if (world[index] === "}") depth -= 1;
    if (depth === 0) return world.slice(start, index + 1);
  }
  assert.fail(`${name} body is unterminated`);
}

const surge = abilities.entries.find((entry) => entry.id === "arcane_surge");
assert.ok(surge, "Aether Surge must be retained in the M4 ability contract");
assert.equal(surge.definition.class, "mage");
assert.deepEqual(surge.definition.specs, ["arcane"]);
assert.equal(surge.definition.cost, 16);
assert.equal(surge.definition.castTime, 2);
assert.equal(surge.definition.range, 30);
assert.equal(surge.definition.projectile, false);
assert.deepEqual(surge.definition.effects, [{ type: "directDamage", min: 10, max: 13 }]);

for (const name of [
  "arcaneSurgeAbilityCode",
  "arcaneSurgePayloadAbilityIsExact",
  "startOfflineAetherSurgeCast",
  "completeOfflineAetherSurgeCast",
  "ageOfflineAetherSurge",
  "aetherSurgeChargeRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}
assert.match(world, /entityAetherSurgeCharges/);
assert.match(world, /entityAetherSurgeChargeRemaining/);
assert.match(world, /entityAetherSurgeFreeRemaining/);
assert.match(world, /aetherSurge\.profile/);
assert.match(world, /aetherSurgeContract\.freeProcChance\(\)/);

const start = functionBody("startOfflineAetherSurgeCast");
assert.match(start, /catalogAdmission\(state, casterIndex, abilityCode, "", false\)/);
assert.match(start, /aetherSurge\.profile/);
assert.match(start, /armTimed/);

const complete = functionBody("completeOfflineAetherSurgeCast");
assert.match(complete, /spendOfflineAbilityResource/);
assert.match(complete, /resolveOfflineFixedSpellImpactDamage/);
assert.match(complete, /entityAetherSurgeCharges/);
assert.match(complete, /nextAuthoritativeRandomUnit/);
assert.ok(
  complete.indexOf("resolveOfflineFixedSpellImpactDamage") <
    complete.lastIndexOf("nextAuthoritativeRandomUnit"),
  "Aether Rush must roll after the resolved Surge impact"
);

assert.match(decision, /pub profile\(/);
assert.match(main, /state\.aetherSurgeChargeRuntimeStateTest\(\)/);
assert.match(project, /backend = "zr_vm:project"/);
assert.match(project, /entry = "src\/world\/aether_surge_charge_runtime_test_main\.zr"/);
console.log("WOS197 Aether Surge charge runtime static guards passed");
