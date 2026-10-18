import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const abilities = JSON.parse(readProjectFile("contracts", "m4_abilities.json"));
const decision = readProjectFile("scripts", "woc_game", "src", "combat", "aether_surge_state.zr");
const main = readProjectFile("scripts", "woc_game", "src", "world", "aether_darts_runtime_test_main.zr");
const project = readProjectFile("scripts", "woc_game", "woc_m4_aether_darts_runtime_tests.zrp");

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

const darts = abilities.entries.find((entry) => entry.id === "arcane_missiles");
assert.ok(darts, "Aether Darts must remain in the M4 ability contract");
assert.equal(darts.definition.class, "mage");
assert.deepEqual(darts.definition.specs, ["arcane"]);
assert.equal(darts.definition.cost, 50);
assert.equal(darts.definition.castTime, 0);
assert.deepEqual(darts.definition.channel, { duration: 3, ticks: 3 });
assert.equal(darts.definition.range, 30);
assert.equal(darts.definition.school, "arcane");
assert.deepEqual(darts.definition.effects, [{ type: "directDamage", min: 8, max: 8 }]);

for (const name of [
  "arcaneMissilesAbilityCode",
  "arcaneMissilesPayloadAbilityIsExact",
  "startOfflineAetherDartsCast",
  "launchOfflineAetherDartsChannelTick",
  "landOfflineAetherDartsProjectile",
  "aetherDartsRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}
for (const field of [
  "entityAetherDartsConsumePending",
  "entityAetherDartsBonusPerBolt",
  "entityAetherDartsTicks",
]) {
  assert.match(world, new RegExp(field), `${field} must be retained`);
}

const start = functionBody("startOfflineAetherDartsCast");
assert.match(start, /catalogAdmission\(state, casterIndex, abilityCode, "", false\)/);
assert.match(start, /aetherSurge\.channelStart/);
assert.match(start, /armChannel/);
assert.match(start, /spendOfflineAbilityResource/);

const landing = functionBody("landOfflineAetherDartsProjectile");
assert.match(landing, /aetherSurge\.resolveDartsBolt/);
assert.match(landing, /applyOfflineArcaneEffectiveDamage/);
assert.match(landing, /applyOfflineTemporalEchoDamageConversion/);
assert.ok(
  landing.indexOf("aetherSurge.resolveDartsBolt") <
    landing.indexOf("nextAuthoritativeRandomUnit"),
  "the first landed missile must consume charges before its crit/range draws"
);

assert.match(decision, /pub channelStart\(/);
assert.match(decision, /pub resolveDartsBolt\(/);
const runtime = functionBody("aetherDartsRuntimeStateTest");
assert.match(runtime, /appendTypedCastTargetCommandForTest/);
assert.match(runtime, /appendCastSlotCommand/);
assert.match(runtime, /entityAetherDartsTicks\[0\] != 5/);
assert.match(runtime, /entityAetherDartsBonusPerBolt\[0\] != 7/);
assert.match(main, /state\.aetherDartsRuntimeStateTest\(\)/);
assert.match(project, /backend\s*=\s*"zr_vm:project"/);
assert.match(project, /entry\s*=\s*"src\/world\/aether_darts_runtime_test_main\.zr"/);
console.log("WOS198 Aether Darts runtime static guards passed");
