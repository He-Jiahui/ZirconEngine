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

const types = source("src/sim/types.ts");
const dispatch = source("src/sim/combat/effect_dispatch.ts");
requireText(
  types,
  /export const STANCE_RAGE_GEN = 0\.1[\s\S]*?export const RECKLESSNESS_RAGE_GEN = 0\.5[\s\S]*?export function rageGenAuraMult[\s\S]*?buff_rage_gen[\s\S]*?buff_reckless[\s\S]*?battle_stance/,
  "source Rage-generation constants or aura ordering drifted",
);
requireText(
  dispatch,
  /function warriorAbilityRageMult[\s\S]*?1 \+ ctx\.playerMods\(meta\)\.global\.abilityRagePct[\s\S]*?rageGenAuraMult\(player\)/,
  "source Warrior ability-Rage multiplier drifted",
);
requireText(
  types,
  /rageGenAuraMult[\s\S]*?let mult = 1[\s\S]*?buff_rage_gen[\s\S]*?mult \+= aura\.value[\s\S]*?buff_reckless[\s\S]*?battle_stance[\s\S]*?return mult/,
  "source Rage aura aggregation semantics drifted",
);

const helper = read("scripts", "woc_game", "src", "combat", "rage_generation_state.zr");
requireText(
  helper,
  /recklessnessRageGeneration\(\): float \{ return 0\.5; \}[\s\S]*?battleStanceRageGeneration\(\): float \{ return 0\.1; \}[\s\S]*?var auraMultiplier = 1\.0 \+ rageGenerationAuraValue[\s\S]*?return \(1\.0 \+ abilityRagePct\) \* auraMultiplier[\s\S]*?pummelReward[\s\S]*?11\.5[\s\S]*?20\.7/,
  "WOC exact Rage multiplier helper drifted",
);
const ccGenerator = read("tools", "cc_contract_codegen.mjs");
requireText(
  ccGenerator,
  /buff_rage_gen[\s\S]*?buff_reckless[\s\S]*?battle_stance[\s\S]*?buff_rage_gen: 14[\s\S]*?buff_reckless: 15[\s\S]*?battle_stance: 16/,
  "WOC Rage aura vocabulary code generation drifted",
);
const cc = JSON.parse(read("reference", "current-head", "cc_contract.json"));
if (cc.schema_version !== 2 || cc.motion_kind_codes.buff_rage_gen !== 14 ||
    cc.motion_kind_codes.buff_reckless !== 15 || cc.motion_kind_codes.battle_stance !== 16) {
  throw new Error("WOC generated Rage aura vocabulary drifted");
}
const resource = read("scripts", "woc_game", "src", "combat", "resource_precision_state.zr");
requireText(
  resource,
  /class ExactResourceState[\s\S]*?isIntegral[\s\S]*?fromLegacy[\s\S]*?canEncodeHistorical[\s\S]*?canSpend[\s\S]*?spend[\s\S]*?cappedAdd[\s\S]*?fractional\.current != 11\.5/,
  "WOC exact resource migration foundation drifted",
);
const fixture = read("scripts", "woc_game", "woc_rage_generation_state_tests.zrp");
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/combat/rage_generation_state_test_main.zr"')) {
  throw new Error("Rage-generation ZrVM project fixture is missing");
}
const resourceFixture = read("scripts", "woc_game", "woc_resource_precision_state_tests.zrp");
if (!resourceFixture.includes('backend = "zr_vm:project"') ||
    !resourceFixture.includes('entry = "src/combat/resource_precision_state_test_main.zr"')) {
  throw new Error("Resource-precision ZrVM project fixture is missing");
}
const worldFixture = read("scripts", "woc_game", "woc_resource_precision_world_state_tests.zrp");
if (!worldFixture.includes('backend = "zr_vm:project"') ||
    !worldFixture.includes('entry = "src/world/resource_precision_world_state_test_main.zr"')) {
  throw new Error("WOS113 resource world-state ZrVM fixture is missing");
}
const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /entitySavedManaMaximums: container\.Array<float>[\s\S]*?entitySavedMana: container\.Array<float>[\s\S]*?entityResources: container\.Array<float>[\s\S]*?entityMaxResources: container\.Array<float>[\s\S]*?offlineCappedResourceGain[\s\S]*?ExactResourceState[\s\S]*?spendOfflineAbilityResource[\s\S]*?precision\.spend/,
  "WOS113 exact resource authority or spend boundary drifted",
);
requireText(
  world,
  /offlineWarriorAbilityRageMultiplier[\s\S]*?buff_rage_gen[\s\S]*?buff_reckless[\s\S]*?battle_stance[\s\S]*?startOfflinePummelCast[\s\S]*?offlineCappedResourceGain[\s\S]*?offlineWarriorAbilityRageMultiplier/,
  "WOS113 Pummel Rage aura integration drifted",
);
requireText(
  world,
  /schemaVersion >= <uint>113[\s\S]*?writer\.fixed6[\s\S]*?entityResources[\s\S]*?entitySavedManaMaximums[\s\S]*?encodeState\(state: WorldState\)[\s\S]*?WOS113|encodeState\(state: WorldState\)[\s\S]*?<uint>113/,
  "WOS113 exact resource codec tail drifted",
);
requireText(
  world,
  /encodeStateVersion\(state: WorldState, schemaVersion: uint\): container\.Array<uint> \{[\s\S]*?!resourcePrecisionStateCanEncodeVersion\(state, schemaVersion\)[\s\S]*?throw "woc fractional resource state cannot be encoded by a historical schema"/,
  "WOS113 writer does not enforce the historical-resource boundary",
);
requireText(
  world,
  /resourcePrecisionStateCanEncodeVersion\(state: WorldState, schemaVersion: uint\): bool \{[\s\S]*?schemaVersion >= <uint>113 \|\| resourcePrecisionStateIsSchema112Compatible\(state\)[\s\S]*?fractional[\s\S]*?resourcePrecisionStateCanEncodeVersion\(fractional, <uint>112\)[\s\S]*?resourcePrecisionStateCanEncodeVersion\(fractional, <uint>113\)/,
  "WOS113 historical-writer refusal boundary drifted",
);
const pummelTest = world.slice(world.indexOf("pub pummelInterruptRageRuntimeStateTest(): int"));
for (const required of [
  "entityTalentRowOptionCodes[3] = <uint>10",
  "entityResources[0], 11.5",
  "entityResources[0], 1.5",
  'motionAuraKindCode("buff_rage_gen")',
  'motionAuraKindCode("buff_reckless")',
  'motionAuraKindCode("battle_stance")',
  "entityResources[0], 20.7",
]) {
  if (!pummelTest.includes(required)) {
    throw new Error(`WOS113 Pummel precision coverage is missing: ${required}`);
  }
}
const plan = read("..", "..", "docs", "plans", "woc", "221-rage-precision-aura-state-migration.md");
requireText(
  plan,
  /WOS113[\s\S]*?fixed6[\s\S]*?WOS112[\s\S]*?nonintegral[\s\S]*?11\.5/,
  "WOS221 migration plan lost its precision boundary",
);
const main = read("scripts", "woc_game", "src", "main.zr");
if ((main.match(/world_state\\\":\\\"WOS118/g) ?? []).length !== 2) {
  throw new Error("WOC package identity does not publish WOS118");
}
const worldContract = read("contracts", "world-state.md");
requireText(
  worldContract,
  /# WOC authoritative world state \(`WOS118`\)[\s\S]*?fixed6[\s\S]*?buff_rage_gen[\s\S]*?10 \* 1\.15[\s\S]*?11\.5/,
  "WOC public WOS118 resource contract drifted",
);

process.stdout.write(`WOS221 Rage precision static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
