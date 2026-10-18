import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const contract = readProjectFile("scripts", "woc_game", "src", "generated", "aether_surge_contract.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "perfect_moment_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_perfect_moment_runtime_tests.zrp");

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

const perfectMoment = abilities.entries.find((entry) => entry.id === "perfect_moment");
assert.ok(perfectMoment, "Perfect Moment must remain in the M4 ability contract");
assert.equal(perfectMoment.definition.class, "mage");
assert.deepEqual(perfectMoment.definition.specs, ["arcane"]);
assert.equal(perfectMoment.definition.learnLevel, 10);
assert.equal(perfectMoment.definition.cost, 0);
assert.equal(perfectMoment.definition.castTime, 0);
assert.equal(perfectMoment.definition.cooldown, 120);
assert.equal(perfectMoment.definition.school, "arcane");
assert.equal(perfectMoment.definition.offGcd, true);
assert.equal(perfectMoment.definition.requiresTarget, false);
assert.deepEqual(perfectMoment.definition.effects, [{ type: "perfectMoment" }]);

for (const name of [
  "perfectMomentAbilityCode",
  "perfectMomentPayloadAbilityIsExact",
  "perfectMomentProfileIsValid",
  "startOfflinePerfectMomentCast",
  "perfectMomentRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}
assert.match(world, /entityAetherSurgePerfectMomentRemaining/);
assert.match(contract, /pub perfectMomentDuration\(\): float \{\s+return 10\.0;/);

const start = functionBody("startOfflinePerfectMomentCast");
assert.match(start, /catalogAdmission\(state, casterIndex, abilityCode, "", false\)/);
assert.match(start, /entityAetherSurgeCharges\[casterIndex\] = aetherSurgeContract\.maxCharges\(\)/);
assert.match(start, /entityAetherSurgeChargeRemaining\[casterIndex\] = aetherSurgeContract\.perfectMomentDuration\(\)/);
assert.match(start, /entityAetherSurgePerfectMomentRemaining\[casterIndex\] = aetherSurgeContract\.perfectMomentDuration\(\)/);
assert.match(start, /state\.timeMicros \+ <uint>120000000/);
assert.doesNotMatch(start, /entityCastGcdRemaining/,
  "the source off-GCD cast must not require or mutate the normal GCD");
assert.doesNotMatch(start, /spendOfflineAbilityResource|nextAuthoritativeRandomUnit/,
  "Perfect Moment has neither resource spend nor RNG");

const auras = functionBody("aetherSurgeAurasForEntity");
assert.match(auras, /aetherSurgeContract\.perfectMomentId\(\)/);
const cancel = functionBody("applyOfflineCancelAuraCommand");
assert.match(cancel, /perfectMomentPayloadAbilityIsExact/);
assert.match(cancel, /entityAetherSurgePerfectMomentRemaining\[actorIndex\] = 0\.0/);
assert.doesNotMatch(cancel, /perfectMomentPayloadAbilityIsExact[\s\S]*entityAetherSurgeCharges\[actorIndex\] = 0/,
  "cancelling Perfect Moment must not remove its separately-owned Charge aura");
assert.match(world, /schemaVersion != <uint>100/);
assert.match(world, /schemaVersion != <uint>101/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /if \(schemaVersion >= <uint>100\)/);
assert.match(world, /schemaVersion != <uint>118/);

const runtime = functionBody("perfectMomentRuntimeStateTest");
assert.match(runtime, /appendTypedCastCommandForTest/);
assert.match(runtime, /appendCastSlotCommand/);
assert.match(runtime, /entityAetherSurgeCharges\[0\] != 4/);
assert.match(runtime, /entityAetherSurgePerfectMomentRemaining\[0\], 10\.0/);
assert.match(runtime, /entityAetherDartsTicks\[0\] != 5/);
assert.match(runtime, /entityAetherDartsConsumePending\[0\]/);
assert.match(main, /state\.perfectMomentRuntimeStateTest\(\)/);
assert.match(project, /backend\s*=\s*"zr_vm:project"/);
assert.match(project, /entry\s*=\s*"src\/world\/perfect_moment_runtime_test_main\.zr"/);
console.log("WOS199 Perfect Moment runtime static guards passed");
