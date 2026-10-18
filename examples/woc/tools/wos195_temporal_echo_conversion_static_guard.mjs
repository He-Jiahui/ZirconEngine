import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const contract = readProjectFile("scripts", "woc_game", "src", "generated", "temporal_echo_contract.zr");
const main = readProjectFile(
  "scripts", "woc_game", "src", "world", "temporal_echo_conversion_runtime_test_main.zr"
);
const project = readProjectFile(
  "scripts", "woc_game", "woc_m4_temporal_echo_conversion_runtime_tests.zrp"
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

assert.match(contract, /pub singleRate\(\): float \{ return 0\.35; \}/);
assert.match(contract, /pub areaRate\(\): float \{ return 0\.15; \}/);
assert.match(contract, /pub groupSingleRate\(\): float \{ return 0\.13; \}/);
assert.match(contract, /pub groupAreaRate\(\): float \{ return 0\.06; \}/);

const effectiveDamage = functionBody("applyOfflineArcaneEffectiveDamage");
assert.match(effectiveDamage, /applyOfflineAbsorbDamage/);
assert.match(effectiveDamage, /var dealt = remainingDamage < hpBefore/);
assert.match(effectiveDamage, /state\.entityHp\[targetIndex\] = hpBefore - dealt/);

const conversion = functionBody("applyOfflineTemporalEchoDamageConversion");
assert.match(conversion, /schoolCode != OFFLINE_PROJECTILE_SCHOOL_ARCANE/);
assert.match(conversion, /entityKinds\[casterIndex\] != <uint>1/);
assert.match(conversion, /offlineTemporalEchoGroups\[index\]/);
assert.match(conversion, /contract\.groupAreaRate\(\) : contract\.areaRate\(\)/);
assert.match(conversion, /offlineTemporalEchoConvertRates\[index\]/);
assert.match(conversion, /applyOfflineResolvedDirectHeal/);
assert.doesNotMatch(conversion, /nextAuthoritativeRandomUnit/);
assert.equal(
  conversion.split("applyOfflineTemporalEchoDamageConversion(").length, 2,
  "Temporal Echo conversion must not recurse"
);

const area = functionBody("startOfflineArcaneExplosionCast");
assert.match(area, /applyOfflineArcaneEffectiveDamage/);
assert.match(area, /OFFLINE_PROJECTILE_SCHOOL_ARCANE, true/);
const ranged = functionBody("landOfflineRangedProjectile");
assert.match(ranged, /offlineProjectileWands/);
assert.match(ranged, /OFFLINE_PROJECTILE_SCHOOL_ARCANE, false/);
const allocation = functionBody("replaceOfflineTalentAllocation");
assert.match(allocation, /stripOfflineTemporalEchoesBySource/);
assert.match(world, /pub temporalEchoConversionRuntimeStateTest\(\): int/);
assert.match(main, /state\.temporalEchoConversionRuntimeStateTest\(\)/);
assert.match(project, /backend = "zr_vm:project"/);
assert.match(project, /entry = "src\/world\/temporal_echo_conversion_runtime_test_main\.zr"/);

console.log("WOS195 Temporal Echo conversion static guards passed");
