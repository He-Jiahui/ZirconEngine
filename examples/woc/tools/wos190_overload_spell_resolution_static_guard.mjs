import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = resolve(projectRoot, "..", "..");
const readProjectFile = (...segments) => readFileSync(resolve(projectRoot, ...segments), "utf8");
const readWorkspaceFile = (...segments) => readFileSync(resolve(workspaceRoot, ...segments), "utf8");

const world = readProjectFile("scripts", "woc_game", "src", "world", "state.zr");
const source = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "combat", "casting_lifecycle.ts"
);

function functionBody(name) {
  const start = world.indexOf(`${name}(`);
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

assert.match(source, /const OVERLOAD_COST_MULT = 1\.5;/);
assert.match(source, /Math\.round\(res\.cost \* OVERLOAD_COST_MULT\)/);
assert.match(source, /\['min', 'max', 'amount', 'bonus', 'total', 'value'\]/);
assert.match(source, /if \(res\.def\.school === 'physical' \|\| res\.cost <= 0\) return res;/);

assert.match(world, /OFFLINE_OVERLOAD_COST_MULTIPLIER/);
assert.match(world, /consumeOfflineOverloadSpell\(/);
assert.match(world, /offlineOverloadScaledPositiveMetric\(/);
assert.match(world, /offlineOverloadBillableCost\(/);
assert.match(
  world,
  /state\.entityResources\[casterIndex\]\s*=\s*precision\.canSpend\([\s\S]*?\)\s*\?\s*[\s\S]*?:\s*0\.0;/,
);
assert.match(world, /offlineProjectileDotTotals: container\.Array<float>/);
assert.match(world, /schemaVersion >= <uint>93/);
assert.match(world, /appendOfflineAbilityProjectileWithDotTotal\(/);
assert.match(world, /offlineProjectileBaseDotTotal\(/);

const requiredResolvers = [
  "completeOfflineFireballCast",
  "startOfflineArcaneExplosionCast",
  "startOfflinePyroblastCast",
  "completeOfflinePyroblastCast",
  "startOfflineCombustionCast",
  "startOfflineFlamestrikeCast",
  "completeOfflineFlamestrikeCast",
  "completeOfflineFrostboltCast",
  "startOfflineConeOfColdCast",
  "startOfflineIceBarrierCast",
  "startOfflineBlazingBarrierCast",
  "completeOfflineGlacialFrontCast",
  "completeOfflineDragonsBreathCast",
];
for (const resolver of requiredResolvers) {
  const body = functionBody(resolver);
  assert.match(body, /consumeOfflineOverloadSpell\(/, `${resolver} must consume Overload`);
}

assert.match(functionBody("resolveOfflineFlamestrikeBurst"), /offlineOverloadScaledPositiveMetric\(/);
assert.match(functionBody("landOfflineFireballProjectile"), /offlineProjectileDotTotals\[projectileIndex\]/);
assert.match(functionBody("landOfflinePyroblastProjectile"), /offlineProjectileDotTotals\[projectileIndex\]/);
assert.doesNotMatch(functionBody("landOfflineFireballProjectile"), /dotTotal <= 0\.0/);
assert.doesNotMatch(functionBody("landOfflinePyroblastProjectile"), /dotTotal <= 0\.0/);
assert.match(functionBody("fireballDotProfileIsValid"), /overloadDamage/);
assert.match(functionBody("pyroblastDotProfileIsValid"), /overloadDamage/);
assert.match(world, /empoweredCone\.resolveHitWithStageDamage\(/);
assert.match(
  readProjectFile("scripts", "woc_game", "src", "combat", "empowered_cone_state.zr"),
  /pub resolveHitWithStageDamage\(/
);
assert.match(world, /pub overloadSpellResolutionStateTest\(\): int/);
assert.match(functionBody("overloadSpellResolutionStateTest"), /restoredProjectileState\.offlineProjectileDotTotals[\s\S]*?landOfflineFireballProjectile[\s\S]*?landOfflinePyroblastProjectile[\s\S]*?offlineDotStateIsValid/);

console.log("WOS190 Overload spell-resolution static guards passed");
