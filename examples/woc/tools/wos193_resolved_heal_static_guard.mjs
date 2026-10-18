import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = resolve(projectRoot, "..", "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");
const readWorkspaceFile = (...segments) => readFileSync(resolve(workspaceRoot, ...segments), "utf8");

const heal = readProjectFile("scripts", "woc_game", "src", "combat", "heal_state.zr");
const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const codegen = readProjectFile("tools", "m4_ability_codegen.mjs");
const catalog = readProjectFile("scripts", "woc_game", "src", "generated", "m4_ability_catalog.zr");
const effects = readProjectFile("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
const runtimeMain = readProjectFile(
  "scripts", "woc_game", "src", "world", "chronomancy_direct_heal_runtime_test_main.zr"
);
const runtimeProject = readProjectFile(
  "scripts", "woc_game", "woc_m4_chronomancy_direct_heal_runtime_tests.zrp"
);
const classes = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "content", "classes.ts"
);

function functionBody(name) {
  const start = heal.indexOf(`\npub ${name}(`);
  assert.notEqual(start, -1, `${name} must be public`);
  const open = heal.indexOf("{", start);
  let depth = 0;
  for (let index = open; index < heal.length; index += 1) {
    if (heal[index] === "{") depth += 1;
    if (heal[index] === "}") depth -= 1;
    if (depth === 0) return heal.slice(start, index + 1);
  }
  assert.fail(`${name} body is unterminated`);
}

function worldFunctionBody(name) {
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

const resolved = functionBody("applyResolvedHeal");
assert.match(resolved, /consumeHealAbsorb/);
assert.match(resolved, /applyHealingThreat/);
assert.doesNotMatch(resolved, /takeRandomUnit/);
assert.doesNotMatch(resolved, /weaponProcCalls/);

assert.match(classes, /temporal_mend:[\s\S]*?cost: 45,[\s\S]*?castTime: 2,[\s\S]*?min: 62, max: 74/);
assert.match(classes, /temporal_echo:[\s\S]*?cost: 40,[\s\S]*?castTime: 0,[\s\S]*?min: 24, max: 30/);
assert.match(classes, /type: 'temporalEcho', duration: 15/);
assert.match(codegen, /'temporal_mend'/);
assert.match(codegen, /'temporal_echo'/);
assert.match(codegen, /EXPECTED_ABILITY_COUNT = 117/);
assert.match(catalog, /pub count\(\): int \{\s+return 117;/);
assert.match(catalog, /if \(id == "temporal_mend"\) \{ return 83; \}/);
assert.match(catalog, /if \(id == "temporal_echo"\) \{ return 84; \}/);
assert.match(effects, /return "temporalEcho"/);

for (const name of [
  "applyOfflineResolvedDirectHeal",
  "applyOfflinePowerEchoHeal",
  "temporalMendAbilityCode",
  "temporalEchoAbilityCode",
  "startOfflineTemporalMendCast",
  "completeOfflineTemporalMendCast",
  "startOfflineTemporalEchoCast",
  "applyOfflineTemporalEchoMark",
  "ageOfflineTemporalEcho",
]) {
  worldFunctionBody(name);
}

const primaryHeal = worldFunctionBody("applyOfflineDirectHeal");
assert.match(primaryHeal, /\): int \{/);
assert.match(primaryHeal, /healState\.applyHeal\(/);
assert.match(primaryHeal, /return healed;/);
const resolvedHeal = worldFunctionBody("applyOfflineResolvedDirectHeal");
assert.match(resolvedHeal, /return applyOfflineWorldHeal\([\s\S]*?false/);
assert.doesNotMatch(resolvedHeal, /nextAuthoritativeRandomUnit/);
const worldHeal = worldFunctionBody("applyOfflineWorldHeal");
assert.match(worldHeal, /if \(resolveCritical\)[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?healState\.applyHeal[\s\S]*?else[\s\S]*?healState\.applyResolvedHeal/);
const echoHeal = worldFunctionBody("applyOfflinePowerEchoHeal");
assert.match(echoHeal, /consumeOfflinePowerEcho\(state, casterIndex\)/);
assert.match(echoHeal, /applyOfflineResolvedDirectHeal\(/);
assert.match(world, /offlineTemporalEchoTargetIds/);
assert.match(world, /offlineTemporalEchoSourceIds/);
assert.match(world, /offlineTemporalEchoRemaining/);
assert.match(world, /if \(schemaVersion >= <uint>97\)/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
const decodedState = worldFunctionBody("decodeState");
assert.match(decodedState, /schemaVersion != <uint>97/);
assert.match(decodedState, /schemaVersion != <uint>107/);
assert.match(decodedState, /if \(schemaVersion >= <uint>97\)/);
assert.match(world, /temporalMendPayloadAbilityIsExact/);
assert.match(world, /temporalEchoPayloadAbilityIsExact/);
assert.match(world, /completedAbility == temporalMendAbilityCode\(\)/);
assert.match(world, /temporalEchoPayloadAbilityIsExact\(payloadBytes, abilityOffset, abilityLength\)/);
assert.match(world, /pub chronomancyDirectHealRuntimeStateTest\(\): int/);
const chronomancyRuntime = worldFunctionBody("chronomancyDirectHealRuntimeStateTest");
assert.match(chronomancyRuntime, /appendTypedCastCommandForTest\(/);
assert.match(chronomancyRuntime, /appendCastSlotCommand\(/);
assert.match(chronomancyRuntime, /rankAtLevel\(mendIndex, 5\) != 1/);
assert.match(chronomancyRuntime, /rankAtLevel\(echoIndex, 18\) != 3/);
assert.match(runtimeMain, /state\.chronomancyDirectHealRuntimeStateTest\(\)/);
assert.match(runtimeProject, /backend = "zr_vm:project"/);
assert.match(runtimeProject, /entry = "src\/world\/chronomancy_direct_heal_runtime_test_main\.zr"/);

console.log("WOS193 resolved-heal static guards passed");
