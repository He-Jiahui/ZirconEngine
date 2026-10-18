import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_COMMIT = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = path.resolve(root, "..", "..");
const sourceRoot = path.resolve(workspaceRoot, "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const source = (file) => execFileSync(
  "git", ["-C", sourceRoot, "show", `${SOURCE_COMMIT}:${file}`], { encoding: "utf8" },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const classes = source("src/sim/content/classes.ts");
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const types = source("src/sim/types.ts");
requireText(
  classes,
  /pummel:[\s\S]*?class: 'warrior'[\s\S]*?learnLevel: 8[\s\S]*?cost: 0[\s\S]*?castTime: 0[\s\S]*?cooldown: 10[\s\S]*?range: 0[\s\S]*?school: 'physical'[\s\S]*?requiresTarget: true[\s\S]*?offGcd: true[\s\S]*?type: 'interrupt', lockout: 4, rageOnInterrupt: 10/,
  "source Pummel profile drifted",
);
requireText(
  dispatch,
  /case 'interrupt':[\s\S]*?target\.castingAbility === null[\s\S]*?FISHING_CAST_ID[\s\S]*?interruptedDef\?\.school === 'physical'[\s\S]*?interruptedDef\?\.uninterruptible[\s\S]*?diminishedCrowdControlDuration\(p, target, 'lockout', eff\.lockout\)[\s\S]*?ctx\.cancelCast\(target\)[\s\S]*?eff\.rageOnInterrupt[\s\S]*?warriorAbilityRageMult[\s\S]*?if \(remaining === null\) break[\s\S]*?kind: 'lockout'/,
  "source Pummel interrupt-result ordering drifted",
);
requireText(
  types,
  /function rageGenAuraMult[\s\S]*?buff_rage_gen[\s\S]*?buff_reckless[\s\S]*?battle_stance/,
  "source Rage-generation aura multiplier drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
const knownAbilityGenerator = read("tools", "known_ability_catalog_codegen.mjs");
const knownAbilityExtractor = read("tools", "known_ability_catalog_source_extract.mjs");
requireText(generator, /'pummel'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/, "Pummel is absent from M4 projection");
requireText(zrGenerator, /document\.entries\.length === 117/, "M4 Zr projection count drifted");
requireText(zrGenerator, /'rageOnInterrupt'/, "M4 Zr projection omits Pummel Rage reward");
requireText(zrGenerator, /'usableInForm', 'uninterruptible'/, "M4 Zr projection omits uninterruptible admission");
requireText(
  knownAbilityExtractor,
  /school: definition\.school/,
  "current-known ability projection omits source school metadata",
);
requireText(
  knownAbilityGenerator,
  /invalid school[\s\S]*?pub abilitySchool\(code: int\): string/,
  "current-known ability school accessor is missing",
);

const knownAbilityCatalog = JSON.parse(read("reference", "current-head", "known_ability_catalog.json"));
const blizzard = knownAbilityCatalog.abilities.find((ability) => ability.id === "blizzard");
if (knownAbilityCatalog.schema_version !== 4 || !blizzard || blizzard.school !== "frost") {
  throw new Error("current-known Blizzard school projection drifted");
}

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const pummel = m4.entries.find((entry) => entry.id === "pummel");
if (m4.entries.length !== 117 || !pummel || pummel.index !== 95 ||
    pummel.definition.class !== "warrior" || pummel.definition.learnLevel !== 8 ||
    pummel.definition.cost !== 0 || pummel.definition.castTime !== 0 ||
    pummel.definition.cooldown !== 10 || pummel.definition.range !== 0 ||
    pummel.definition.school !== "physical" || !pummel.definition.requiresTarget ||
    !pummel.definition.offGcd || pummel.definition.effects?.[0]?.type !== "interrupt" ||
    pummel.definition.effects?.[0]?.lockout !== 4 ||
    pummel.definition.effects?.[0]?.rageOnInterrupt !== 10) {
  throw new Error("M4 Pummel source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /pummelAbilityCode[\s\S]*?pummelPayloadAbilityIsExact[\s\S]*?pummelProfileIsValid[\s\S]*?range <= 0\.0[\s\S]*?range = 5\.0[\s\S]*?pummelTargetCastIsInterruptible[\s\S]*?"uninterruptible"[\s\S]*?startOfflinePummelCast/,
  "Pummel identity, profile, range, or interruptible admission is missing",
);
requireText(
  world,
  /offlineAbilitySchoolCode[\s\S]*?knownAbilityCatalog\.abilitySchool[\s\S]*?pummelCastIsInterruptible[\s\S]*?pummelTargetCastIsInterruptible[\s\S]*?abilityIndex >= 0[\s\S]*?"uninterruptible"[\s\S]*?startOfflinePummelCast[\s\S]*?m4AbilityCatalog\.flag\(abilityIndex, "offGcd"\)[\s\S]*?setAbilityCooldownExpiration[\s\S]*?pummelTargetCastIsInterruptible[\s\S]*?cancelOfflineTargetCast[\s\S]*?rageOnInterrupt[\s\S]*?applyOfflineMotionAuraWithDetailsAndSchool/,
  "Pummel result-dependent reward, lockout, or cooldown ordering is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?pummelAbilityCode\(\)[\s\S]*?startOfflinePummelCast[\s\S]*?applySupportedCastCommand[\s\S]*?pummelPayloadAbilityIsExact[\s\S]*?startOfflinePummelCast/,
  "Pummel slot and typed routes are missing",
);
const testOffset = world.indexOf("pub pummelInterruptRageRuntimeStateTest(): int");
const runtimeTest = testOffset >= 0 ? world.slice(testOffset) : "";
for (const required of [
  "abilityIndex != 95",
  "entityCastGcdRemaining[0] = 0.75",
  "entityResources[0] != 100",
  "appendCastSlotCommand",
  "appendTypedCastTargetCommandForTest",
  'knownAbilityCatalog.abilityCode("blizzard")',
  "offlineAbilitySchoolCode(blizzard) != OFFLINE_PROJECTILE_SCHOOL_FROST",
  "pummelCastIsInterruptible(true, false, OFFLINE_PROJECTILE_SCHOOL_FROST, true)",
  "fishingCast.fishing = true",
  "entityZ[1] = 5.1",
  "entityFacing[0] = 3.14159",
  "entityResources[0] != 10",
  "entityCastingAbility[1] != pummel",
  "entityCastingAbility[1] = <uint>0",
  "rngDraws == <uint>0",
]) {
  if (!runtimeTest.includes(required)) {
    throw new Error(`Pummel runtime regression coverage is missing: ${required}`);
  }
}

const fixture = read("scripts", "woc_game", "woc_m4_pummel_interrupt_rage_runtime_tests.zrp");
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/pummel_interrupt_rage_runtime_test_main.zr"')) {
  throw new Error("Pummel ZrVM project fixture is missing");
}

process.stdout.write(`WOS220 Pummel static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
