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
const talents = source("src/sim/content/talents_classic.ts");
requireText(
  classes,
  /chain_heal:[\s\S]*?type: 'chainHeal', min: 120, max: 145, jumps: 2, falloff: 0\.5, radius: 12/,
  "source Chain Heal effect contract drifted",
);
requireText(
  classes,
  /case 'chainHeal':[\s\S]*?min: Math\.round\(eff\.min \* healMult \+ flat\)[\s\S]*?max: Math\.round\(eff\.max \* healMult \+ flat\)/,
  "source Chain Heal talent scaling drifted",
);
requireText(
  classes,
  /if \(am\.costPct\) entry\.cost = Math\.max\(0, Math\.round\(entry\.cost \* \(1 \+ am\.costPct\)\)\)/,
  "source per-ability cost scaling drifted",
);
requireText(
  talents,
  /ability: 'chain_heal', costPct: -0\.2/,
  "source Restoration Chain Heal cost modifier drifted",
);
requireText(
  dispatch,
  /case 'chainHeal':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?chain\.includes\(e\)[\s\S]*?frac < bestFrac[\s\S]*?d2 < bestD2[\s\S]*?e\.id < best\.id[\s\S]*?Math\.max\(1, Math\.round\(baseAmount \* eff\.falloff \*\* i\)\)[\s\S]*?ctx\.applyHeal\(/,
  "source Chain Heal selection, rounding or heal dispatch drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
requireText(
  generator,
  /'chain_heal'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  "Chain Heal is absent from the retained M4 projection",
);
requireText(
  zrGenerator,
  /document\.entries\.length === 117/,
  "M4 Zr projection count did not retain Chain Heal",
);
requireText(zrGenerator, /'jumps'/, "M4 Zr projection cannot represent Chain Heal jumps");
requireText(zrGenerator, /'falloff'/, "M4 Zr projection cannot represent Chain Heal falloff");

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const chainHeal = m4.entries.find((entry) => entry.id === "chain_heal");
if (!chainHeal || chainHeal.index !== 93 || chainHeal.definition.class !== "shaman" ||
    chainHeal.definition.learnLevel !== 10 || chainHeal.definition.cost !== 60 ||
    chainHeal.definition.castTime !== 2.5 || chainHeal.definition.range !== 30 ||
    chainHeal.definition.school !== "nature" || chainHeal.definition.targetType !== "friendly" ||
    chainHeal.definition.effects?.[0]?.type !== "chainHeal" ||
    chainHeal.definition.effects?.[0]?.min !== 120 ||
    chainHeal.definition.effects?.[0]?.max !== 145 ||
    chainHeal.definition.effects?.[0]?.jumps !== 2 ||
    chainHeal.definition.effects?.[0]?.falloff !== 0.5 ||
    chainHeal.definition.effects?.[0]?.radius !== 12) {
  throw new Error("M4 Chain Heal source projection drifted");
}

const selection = read("scripts", "woc_game", "src", "combat", "chain_heal_state.zr");
requireText(
  selection,
  /pub betterCandidate[\s\S]*?candidateFraction[\s\S]*?bestFraction[\s\S]*?candidateDistanceSquared[\s\S]*?candidateId < bestId/,
  "WOC Chain Heal deterministic selection helper is missing",
);
requireText(
  selection,
  /pub hopAmount[\s\S]*?amount = amount \* falloff[\s\S]*?rounded < 1 \? 1 : rounded[\s\S]*?hopAmount\(145\.0, 0\.5, 2\) == 36/,
  "WOC Chain Heal falloff helper is missing",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /chainHealAbilityCode\([\s\S]*?chainHealProfileIsValid[\s\S]*?chainHealResolvedCost[\s\S]*?startOfflineChainHealCast[\s\S]*?completeOfflineChainHealCast/,
  "Chain Heal world cast reducer is missing",
);
requireText(
  world,
  /chainHealResolvedCost[\s\S]*?talentModifiers\.abilityField\(modifiers, "chain_heal", "costPct"\)[\s\S]*?startOfflineChainHealCast[\s\S]*?var cost = chainHealResolvedCost[\s\S]*?completeOfflineChainHealCast[\s\S]*?var cost = chainHealResolvedCost/,
  "Chain Heal start and completion must share the source-resolved talent cost",
);
requireText(
  world,
  /chainHealTargetIndices[\s\S]*?lesserHealFriendlyTargetIndex[\s\S]*?chainHealState\.betterCandidate[\s\S]*?targets\.add\(bestIndex\)/,
  "Chain Heal must select eligible hops with the source ladder",
);
requireText(
  world,
  /completeOfflineChainHealCast[\s\S]*?directHealTalentScaling\.resolveDirectHealRange[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?chainHealTargetIndices[\s\S]*?applyOfflineWorldHeal[\s\S]*?chainHealState\.hopAmount[\s\S]*?true/,
  "Chain Heal must share one range draw then resolve every hop through normal heals",
);
requireText(
  world,
  /applyOfflineWorldHeal[\s\S]*?resolveCritical[\s\S]*?events\.randomUnits\.add\(nextAuthoritativeRandomUnit\(state\)\)[\s\S]*?healState\.applyHeal[\s\S]*?healState\.applyResolvedHeal/,
  "World heal adapter must preserve critical and resolved paths",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?chainHealAbilityCode\(\)[\s\S]*?startOfflineChainHealCast[\s\S]*?applySupportedCastCommand[\s\S]*?chainHealPayloadAbilityIsExact[\s\S]*?startOfflineChainHealCast/,
  "Chain Heal slot and typed routes are missing",
);
requireText(
  world,
  /stepRetainedCasting[\s\S]*?completedAbility == chainHealAbilityCode\(\)[\s\S]*?completeOfflineChainHealCast[\s\S]*?pub chainHealAuthoritativeRuntimeStateTest\(\): int[\s\S]*?rngDraws != <uint>4/,
  "Chain Heal completion or regression coverage is missing",
);
const chainHealTestOffset = world.indexOf("pub chainHealAuthoritativeRuntimeStateTest(): int");
const chainHealTest = chainHealTestOffset >= 0 ? world.slice(chainHealTestOffset) : "";
if (!chainHealTest.includes("state.entityResources[0] = 48;") ||
    !chainHealTest.includes("state.entityTalentSpecCodes[0] = <uint>18;") ||
    !chainHealTest.includes("state.applyCommands(slot, 1);") ||
    !chainHealTest.includes("entityResources[0] != 48") ||
    !chainHealTest.includes("stepRetainedCasting(restored, 0, false);") ||
    !chainHealTest.includes("entityResources[0] != 0")) {
  throw new Error("Chain Heal Restoration-cost regression coverage is missing");
}

const fixture = read("scripts", "woc_game", "woc_m4_chain_heal_authoritative_runtime_tests.zrp");
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/chain_heal_authoritative_runtime_test_main.zr"')) {
  throw new Error("Chain Heal ZrVM project fixture is missing");
}

process.stdout.write(`WOS218 Chain Heal projection static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
