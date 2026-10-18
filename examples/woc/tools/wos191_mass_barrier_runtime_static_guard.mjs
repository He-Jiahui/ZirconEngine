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
const targeting = readWorkspaceFile(
  "dev", "world-of-claudecraft", "src", "sim", "combat", "group_targeting.ts"
);
const plan = readWorkspaceFile("docs", "plans", "woc", "191-mass-barrier-runtime.md");
const runtimeFixture = readProjectFile(
  "scripts", "woc_game", "woc_m4_mass_barrier_runtime_tests.zrp"
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

assert.match(classes, /mass_barrier:[\s\S]*?learnLevel: 17,[\s\S]*?cost: 150,[\s\S]*?cooldown: 90,[\s\S]*?school: 'frost',[\s\S]*?aoeAllyAbsorb', amount: 130, duration: 60, radius: 30, maxTargets: 5/);
assert.match(targeting, /const memberIds = party \? party\.members : \[caster\.id\]/);
assert.match(targeting, /!e \|\| !meta \|\| e\.dead/);
assert.match(targeting, /dx \* dx \+ dz \* dz > r2/);
assert.match(targeting, /out\.sort\(\(a, b\) => a\.id - b\.id\)/);
assert.match(dispatch, /if \(a\.id === p\.id\) return -1;/);
assert.match(dispatch, /return da - db \|\| a\.id - b\.id;/);
assert.match(dispatch, /personalBarrierIdForSpec\(spec\)/);
assert.match(plan, /Arcane personal barrier to at least 12 seconds, or the Fire\/Frost\s+personal barrier to at least 30 seconds/);

for (const name of [
  "massBarrierAbilityCode",
  "massBarrierProfileIsValid",
  "massBarrierAbsorbProfileMatches",
  "massBarrierRecipientIndexes",
  "startOfflineMassBarrierCast",
  "massBarrierRuntimeStateTest",
]) {
  assert.match(world, new RegExp(`(?:pub )?${name}\\(`), `${name} is required`);
}

assert.match(functionBody("offlineAbilitySchoolCode"), /knownAbilityCatalog\.abilitySchool\(<int>abilityCode\)/);
assert.match(functionBody("massBarrierProfileIsValid"), /massBarrierAbilityCode\(\)/);
assert.match(runtimeFixture, /backend = "zr_vm:project"/);
assert.match(runtimeFixture, /entry = "src\/world\/mass_barrier_runtime_test_main\.zr"/);
assert.match(functionBody("offlineAbsorbProfileMatches"), /massBarrierAbsorbProfileMatches\(/);
assert.match(functionBody("applySupportedCastSlotCommand"), /massBarrierAbilityCode\(\)[\s\S]*?startOfflineMassBarrierCast\(/);
assert.match(functionBody("applySupportedCastCommand"), /massBarrierPayloadAbilityIsExact[\s\S]*?startOfflineMassBarrierCast\(/);

const reducer = functionBody("startOfflineMassBarrierCast");
assert.match(reducer, /<float>state\.entityResources\[casterIndex\] < 150/);
assert.match(reducer, /consumeOfflineOverloadSpell\(state, casterIndex, 150\)/);
assert.match(reducer, /offlineOverloadBillableCost\(150, overloadMultiplier\)/);
assert.match(reducer, /state\.timeMicros \+ <uint>90000000/);
assert.match(reducer, /massBarrierRecipientIndexes\(state, casterIndex\)/);
assert.match(reducer, /applyOfflineFixedAbsorb\(/);
assert.match(reducer, /massBarrierPersonalBarrier/);

const recipients = functionBody("massBarrierRecipientIndexes");
assert.match(recipients, /partyIdForIndex\(/);
assert.match(recipients, /state\.entityKinds\[candidateIndex\] != <uint>1/);
assert.match(recipients, /state\.entityDead\[candidateIndex\]/);
assert.match(recipients, /dx \* dx \+ dz \* dz > 900\.0/);
assert.match(recipients, /candidateCount > 5/);

const runtimeTest = functionBody("massBarrierRuntimeStateTest");
assert.match(runtimeTest, /appendTypedCastCommandForTest\(/);
assert.match(runtimeTest, /appendCastSlotCommand\(/);
assert.match(runtimeTest, /orderedRecipients[\s\S]*?distanceRecipients/);
assert.match(runtimeTest, /applyOfflineAbsorbDamage\(state, 1, 50\)[\s\S]*?decodeState/);
assert.match(runtimeTest, /abilityCooldownExpiresAt\(fire, 0, blazingBarrier\).*30000000/);
assert.match(runtimeTest, /abilityCooldownExpiresAt\(frost, 0, iceBarrier\).*30000000/);

console.log("WOS191 Mass Barrier runtime static guards passed");
