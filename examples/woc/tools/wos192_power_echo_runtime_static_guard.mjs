import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = resolve(projectRoot, "..", "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");
const readWorkspaceFile = (...segments) => readFileSync(resolve(workspaceRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const classes = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "content", "classes.ts"
);
const dispatch = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "combat", "effect_dispatch.ts"
);
const runtimeFixture = readProjectFile(
  "scripts", "woc_game", "woc_m4_power_echo_runtime_tests.zrp"
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

assert.match(classes, /power_echo:[\s\S]*?learnLevel: 14,[\s\S]*?cost: 0,[\s\S]*?cooldown: 30,[\s\S]*?school: 'arcane',[\s\S]*?offGcd: true,[\s\S]*?kind: 'power_echo', value: 0\.5, duration: 10/);
assert.match(dispatch, /const echoAmt = Math\.max\(1, Math\.round\(finalDamage \* echoAura\.value\)\)/);
assert.match(dispatch, /scheduleProjectile\(ctx, p, target/);
assert.match(dispatch, /const echoHeal = Math\.max\(1, Math\.round\(healed \* echoAura\.value\)\)/);
assert.match(dispatch, /ctx\.applyHeal\(p, healTarget, echoHeal, ability\.name, ability\.id, false, false\)/);
assert.match(runtimeFixture, /backend = "zr_vm:project"/);
assert.match(runtimeFixture, /entry = "src\/world\/power_echo_runtime_test_main\.zr"/);

for (const name of [
  "powerEchoAbilityCode",
  "powerEchoProfileIsValid",
  "startOfflinePowerEchoCast",
  "consumeOfflinePowerEcho",
  "appendOfflineFixedSpellProjectile",
  "resolveOfflineFixedSpellImpactDamage",
  "landOfflineFixedSpellProjectile",
  "landOfflineFrostboltProjectile",
  "landOfflineFireballProjectile",
  "landOfflinePyroblastProjectile",
  "stepOfflineEastbrookProjectiles",
  "applyOfflineCancelAuraCommand",
  "powerEchoRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}

const abilitySchool = functionBody("offlineAbilitySchoolCode");
assert.match(abilitySchool, /knownAbilityCatalog\.abilitySchool\(<int>abilityCode\)/);
assert.match(abilitySchool, /offlineLockoutSchoolCode/);
const powerEchoStart = functionBody("startOfflinePowerEchoCast");
assert.match(powerEchoStart, /state\.timeMicros \+ <uint>30000000/);
assert.match(powerEchoStart, /entityPowerEchoRemaining\[casterIndex\] = 10\.0/);
assert.match(powerEchoStart, /entityPowerEchoValues\[casterIndex\] = 0\.5/);
assert.doesNotMatch(powerEchoStart, /entityCastGcdRemaining/);
assert.doesNotMatch(powerEchoStart, /spendOfflineAbilityResource/);
const powerEchoProfile = functionBody("powerEchoProfileIsValid");
assert.match(powerEchoProfile, /talentSelectionCatalog\.optionCode\(\s*"mage", 14, "mag_r14_power_echo"\s*\)/);
assert.match(powerEchoProfile, /talentSelectionCatalog\.optionMatchesClassRow\("mage", 14, optionCode\)/);
assert.match(functionBody("consumeOfflinePowerEcho"), /state\.entityPowerEchoRemaining\[casterIndex\] = 0\.0/);
assert.match(functionBody("consumeOfflinePowerEcho"), /state\.entityPowerEchoValues\[casterIndex\] = 0\.0/);

const fixedProjectile = functionBody("appendOfflineFixedSpellProjectile");
assert.match(fixedProjectile, /fixedDamage > 0/);
assert.match(fixedProjectile, /OFFLINE_PROJECTILE_PENDING_MAX/);
assert.match(fixedProjectile, /offlineProjectileMinimums\.add\(0\.0\)/);
assert.match(fixedProjectile, /offlineProjectileMaximums\.add\(0\.0\)/);
assert.match(fixedProjectile, /critical: bool/);
assert.match(fixedProjectile, /offlineProjectileCriticals\.add\(critical\)/);

const schedule = functionBody("scheduleOfflinePowerEchoDamage");
assert.match(schedule, /consumeOfflinePowerEcho\(state, casterIndex\)[\s\S]*?targetHpAfterPrimary <= 0/);
assert.match(schedule, /appendOfflineFixedSpellProjectile/);
assert.match(schedule, /offlineAbilitySchoolCode\(abilityCode\)/);
assert.match(schedule, /critical: bool/);
const fixedLanding = functionBody("landOfflineFixedSpellProjectile");
assert.match(fixedLanding, /offlineProjectileCriticals\[projectileIndex\]/);
assert.match(fixedLanding, /resolveOfflineFixedSpellImpactDamage\([\s\S]*?critical/);
assert.doesNotMatch(fixedLanding, /applyOfflineFireMageSpellHit/);
for (const name of [
  "landOfflineFrostboltProjectile",
  "landOfflineFireballProjectile",
  "landOfflinePyroblastProjectile",
]) {
  assert.match(functionBody(name), /scheduleOfflinePowerEchoDamage/);
}

const projectileStep = functionBody("stepOfflineEastbrookProjectiles");
assert.match(
  projectileStep,
  /offlineProjectileFixedDamages\[projectileIndex\][\s\S]*?landOfflineFixedSpellProjectile[\s\S]*?frostboltAbilityCode\(\)/
);
const cancelAura = functionBody("applyOfflineCancelAuraCommand");
assert.match(cancelAura, /powerEchoPayloadAbilityIsExact[\s\S]*?entityPowerEchoRemaining\[actorIndex\] = 0\.0[\s\S]*?entityPowerEchoValues\[actorIndex\] = 0\.0/);

assert.match(world, /entityPowerEchoRemaining: container\.Array<float>/);
assert.match(world, /entityPowerEchoValues: container\.Array<float>/);
assert.match(world, /offlineProjectileFixedDamages: container\.Array<int>/);
assert.match(world, /offlineProjectileCriticals: container\.Array<bool>/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /schemaVersion != <uint>94 &&\s*schemaVersion != <uint>95/);
assert.match(world, /schemaVersion >= <uint>95[\s\S]*?projectileCritical[\s\S]*?offlineProjectileCriticals\.add\(projectileCritical == <uint>1\)/);
const powerEchoTest = functionBody("powerEchoRuntimeStateTest");
assert.match(powerEchoTest, /entityCritChance\[0\] = 1\.0/);
assert.match(powerEchoTest, /rngDraws[\s\S]*?offlineProjectileFixedDamages[\s\S]*?offlineProjectileCriticals[\s\S]*?decodeState/);
assert.match(powerEchoTest, /<bool>restored\.offlineProjectileCriticals\[0\] != copiedCritical/);
assert.match(powerEchoTest, /encodeStateVersion\(historical, <uint>94\)[\s\S]*?restoredHistorical\.offlineProjectileCriticals/);
assert.match(powerEchoTest, /appendCancelAuraCommandForTest[\s\S]*?entityPowerEchoRemaining\[0\] != 0\.0[\s\S]*?abilityCooldownExpiresAt\(cancelled, 0, powerEcho\) != <uint>30000000/);

console.log("WOS192 Power Echo runtime static guards passed");
