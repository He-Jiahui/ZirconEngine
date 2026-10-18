import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = resolve(projectRoot, "..", "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");
const readWorkspaceFile = (...segments) => readFileSync(resolve(workspaceRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const fireMage = readProjectFile(
  "scripts", "woc_game", "src", "combat", "fire_mage_state.zr"
);
const damage = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "combat", "damage.ts"
);
const sourceFireMage = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "combat", "fire_mage.ts"
);
const runtimeFixture = readProjectFile(
  "scripts", "woc_game", "woc_m4_critical_spell_impact_runtime_tests.zrp"
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

assert.match(damage, /if \(crit && amount > 0 && source && source\.id !== target\.id\)[\s\S]*?ctx\.critVulnBonus\(target\)/);
assert.match(damage, /igniteOnCrit\(ctx, source, target, amount, crit, school, ability\)/);
assert.match(sourceFireMage, /applyIgnite\(ctx, source, target, Math\.round\(amount \* mods\.global\.ignitionPct\)\)/);
assert.match(fireMage, /pub igniteOnCritPlan\(/);
assert.match(fireMage, /pub ignitePlan\(/);

for (const name of [
  "offlineCritVulnBonus",
  "applyOfflineIgniteOnCriticalSpellImpact",
  "resolveOfflineFixedSpellImpactDamage",
  "landOfflineFixedSpellProjectile",
  "criticalSpellImpactRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}

const fixedImpact = functionBody("resolveOfflineFixedSpellImpactDamage");
assert.match(fixedImpact, /offlineCritVulnBonus\(state, targetIndex\)/);
assert.match(fixedImpact, /applyOfflineIgniteOnCriticalSpellImpact/);
assert.doesNotMatch(fixedImpact, /applyOfflineFireMageSpellHit/);
assert.match(world, /offlineCritVulnTargetIds/);
assert.match(world, /offlineCritVulnStateIsValid/);
assert.match(world, /applyOfflineCritVuln\(/);
assert.match(world, /ageOfflineCritVuln\(state\)/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /if \(schemaVersion >= <uint>96\)/);
assert.match(runtimeFixture, /backend = "zr_vm:project"/);
assert.match(runtimeFixture, /entry = "src\/world\/critical_spell_impact_runtime_test_main\.zr"/);

const directSpellLandings = [
  "landOfflineEarthShockProjectile",
  "landOfflineFlameShockProjectile",
  "landOfflineFrostShockProjectile",
  "landOfflineFrostboltProjectile",
  "landOfflineSmiteProjectile",
  "landOfflineShadowBoltProjectile",
  "landOfflineWrathProjectile",
  "landOfflineStarfireProjectile",
  "landOfflineSearingPainProjectile",
  "landOfflineShadowburnProjectile",
  "landOfflineConflagrateProjectile",
  "landOfflineMindBlastProjectile",
  "landOfflineLightningBoltProjectile",
  "landOfflineImmolateProjectile",
  "landOfflineFireballProjectile",
  "landOfflinePyroblastProjectile",
  "landOfflineMoonfireProjectile",
];

for (const name of directSpellLandings) {
  const handler = functionBody(name);
  assert.match(handler, /resolveOfflineFixedSpellImpactDamage\(/);
}
assert.match(world, /var nonMageCritical = resolveOfflineFixedSpellImpactDamage\([\s\S]*?lightningBolt[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE/);

for (const name of [
  "landOfflineFrostboltProjectile",
  "landOfflineFireballProjectile",
  "landOfflinePyroblastProjectile",
]) {
  const handler = functionBody(name);
  assert.match(handler, /scheduleOfflinePowerEchoDamage\([\s\S]*?rawDamage, hp/);
}

const ignite = functionBody("applyOfflineIgniteDot");
assert.match(ignite, /offlineDotSourceIds\[index\] == sourceId/);
assert.match(ignite, /offlineDotTickTimers\.add\(interval\)/);
assert.doesNotMatch(ignite, /offlineDotTickTimers\[index\] =/);
assert.match(world, /if \(abilityCode == ignitionAbilityCode\(\)\)/);
assert.match(world, /resolvedTotal == <int>state\.offlineDotDamages\[index\] \* ticks/);
assert.match(world, /offlineDotAbilityCodes\[index\] == ignitionAbilityCode\(\)/);

console.log("WOS194 critical spell impact runtime static guards passed");
