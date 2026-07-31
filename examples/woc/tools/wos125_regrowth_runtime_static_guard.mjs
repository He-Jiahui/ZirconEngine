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
const start = classes.indexOf("  regrowth: {");
const end = classes.indexOf("  barkskin: {", start);
if (start < 0 || end < start) throw new Error("source Regrowth block is missing");
const regrowth = classes.slice(start, end);
for (const needle of [
  "name: 'Second Bloom'", "class: 'druid'", "learnLevel: 14", "cost: 55",
  "castTime: 2.0", "cooldown: 0", "range: 30", "school: 'nature'",
  "requiresTarget: true", "targetType: 'friendly'",
  "type: 'heal', min: 52, max: 62",
  "type: 'hot', total: 49, duration: 21, interval: 3",
]) {
  if (!regrowth.includes(needle)) throw new Error(`source Regrowth drifted: ${needle}`);
}
requireText(dispatch, /case 'heal':[\s\S]*?directHealBonus\(p\.spellPower, res\.castTime\)[\s\S]*?applyHeal/,
  "source Regrowth direct-heal dispatch drifted");
requireText(dispatch, /case 'hot':[\s\S]*?const hybridHeal = res\.effects\.some\(\(e\) => e\.type === 'heal'\)[\s\S]*?const hotSp = hybridHeal \? 0 : hotTickBonus\(p\.spellPower, eff\.duration, eff\.interval\)[\s\S]*?applyAura/,
  "source Regrowth hybrid HoT snapshot rule drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/swipe',[\s\S]*?'regrowth'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Regrowth projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "regrowth",
);
if (!entry || entry.index !== 65 || entry.definition.cost !== 55 ||
    entry.definition.castTime !== 2 || entry.definition.targetType !== "friendly" ||
    entry.definition.effects?.length !== 2 ||
    entry.definition.effects[0].type !== "heal" || entry.definition.effects[0].min !== 52 ||
    entry.definition.effects[0].max !== 62 || entry.definition.effects[1].type !== "hot" ||
    entry.definition.effects[1].total !== 49 || entry.definition.effects[1].duration !== 21 ||
    entry.definition.effects[1].interval !== 3) {
  throw new Error("M4 Regrowth projection drifted");
}

const hybridProfiles = read("scripts", "woc_game", "src", "combat", "hybrid_hot_profile_state.zr");
requireText(hybridProfiles, /class HybridHotProfile[\s\S]*?effects\.count\(abilityIndex, rank\) != 2[\s\S]*?effects\.typeAt\(abilityIndex, rank, 0\) != "heal"[\s\S]*?effects\.typeAt\(abilityIndex, rank, 1\) != "hot"[\s\S]*?profile\.heal = base;[\s\S]*?resolveHybridHotProfile/,
  "Regrowth hybrid HoT profile must retain source zero-extra-scaling ticks");
requireText(hybridProfiles, /hybridHotProfileMatches[\s\S]*?contractTest[\s\S]*?regrowth[\s\S]*?heal != 7/,
  "Regrowth hybrid HoT profile contract coverage is missing");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /var hybridHotProfiles = %import\("combat\/hybrid_hot_profile_state"\);/,
  "Regrowth hybrid profile module is not wired");
requireText(world, /regrowthAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("regrowth"\)/,
  "Regrowth catalog identity is missing");
requireText(world, /startOfflineRegrowthCast[\s\S]*?cast\.armTimed[\s\S]*?regrowthCastSeconds[\s\S]*?regrowthGlobalCooldownSeconds[\s\S]*?cast\.lockTargets/,
  "Regrowth friendly timed-cast reducer is missing");
requireText(world, /completeOfflineRegrowthCast[\s\S]*?m4AbilityEffects\.count\(abilityIndex, rank\) != 2[\s\S]*?applyOfflineDirectHeal[\s\S]*?applyOfflineRegrowthHot/,
  "Regrowth direct-then-HoT completion ordering is missing");
requireText(world, /applyOfflineRegrowthHot[\s\S]*?hybridHotProfiles\.resolveHybridHotProfile[\s\S]*?offlineHotSnapshotPowers\.add\(0\)/,
  "Regrowth zero-snapshot HoT queue reducer is missing");
requireText(world, /offlineHotStateIsValid[\s\S]*?hybridHotProfiles\.hybridHotProfileMatches/,
  "Regrowth serialized HoT profile validation is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?regrowthAbilityCode\(\)[\s\S]*?startOfflineRegrowthCast/,
  "Regrowth action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?regrowthPayloadAbilityIsExact[\s\S]*?startOfflineRegrowthCast/,
  "Regrowth typed routing is missing");
requireText(world, /stepRetainedCasting[\s\S]*?regrowthAbilityCode\(\)[\s\S]*?completeOfflineRegrowthCast/,
  "Regrowth cast-completion routing is missing");
requireText(world, /pub regrowthCommandStateTest\(\): int[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?offlineHotSnapshotPowers/, 
  "Regrowth state regression coverage is missing");
requireText(world, /if \(regrowthCommandStateTest\(\) != 1\) \{[\s\S]*?return -119;/,
  "world selfTest must execute Regrowth");

process.stdout.write(`WOS125 Regrowth static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
