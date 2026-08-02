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
const casting = source("src/sim/combat/casting_lifecycle.ts");
const effects = source("src/sim/combat/effect_dispatch.ts");
const types = source("src/sim/types.ts");
const sourceAbilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const serpentSource = sourceAbilityBlock("serpent_sting", "arcane_shot");
for (const needle of [
  "class: 'hunter'", "cost: 15", "castTime: 0", "range: 35", "minRange: 8",
  "school: 'nature'", "scalesWith: 'ranged'", "total: 20, duration: 15, interval: 3",
  "rank: 2", "level: 10", "total: 35, duration: 15, interval: 3", "rank: 3",
  "level: 16", "total: 55, duration: 15, interval: 3",
]) {
  if (!serpentSource.includes(needle)) throw new Error(`source Serpent Sting drifted: ${needle}`);
}
const shadowPainSource = sourceAbilityBlock("shadow_word_pain", "power_word_shield");
for (const needle of [
  "class: 'priest'", "cost: 25", "castTime: 0", "range: 30", "school: 'shadow'",
  "total: 30, duration: 18, interval: 3", "rank: 2", "level: 10",
  "total: 54, duration: 18, interval: 3", "rank: 3", "level: 16",
  "total: 84, duration: 18, interval: 3",
]) {
  if (!shadowPainSource.includes(needle)) {
    throw new Error(`source Shadow Word: Pain drifted: ${needle}`);
  }
}
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?scheduleProjectile\(ctx, p, target,[\s\S]*?isSpellResisted\(ctx\.rng, src\.level, tgt\.level, src\.hitBonus\)[\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\);/,
  "source instant spell projectile lifecycle drifted",
);
requireText(
  effects,
  /case 'dot':[\s\S]*?const hybrid = res\.effects\.some[\s\S]*?dotTickBonus\(abilityScalingPower\(p, ability\), ability, eff\.duration, eff\.interval\)[\s\S]*?tickTimer: eff\.interval/,
  "source pure-DoT snapshot dispatch drifted",
);
requireText(types, /export const MIN_GCD = 0\.75;/, "source global cooldown floor drifted");

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'serpent_sting',[\s\S]*?'shadow_word_pain'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79")) {
  throw new Error("M4 instant DoT projection scope is missing");
}
if (!zrGenerator.includes("'minRange'") || !zrGenerator.includes("'scalesWith'") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 instant DoT Zr projection fields are missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const serpent = m4.entries.find((entry) => entry.id === "serpent_sting");
const shadowPain = m4.entries.find((entry) => entry.id === "shadow_word_pain");
if (!serpent || serpent.index !== 23 || serpent.scenarios.length !== 0 ||
    serpent.definition.minRange !== 8 || serpent.definition.scalesWith !== "ranged" ||
    serpent.definition.effects?.[0]?.total !== 20 ||
    shadowPain?.index !== 24 || shadowPain.scenarios.length !== 0 ||
    shadowPain.definition.effects?.[0]?.total !== 30) {
  throw new Error("M4 instant DoT source projection drifted");
}

const numeric = read("scripts", "woc_game", "src", "combat", "effect_numeric_dispatch_state.zr");
requireText(numeric, /class PureDotProfile[\s\S]*?pub resolvePureDotProfile\([\s\S]*?pub pureDotProfileMatches\(/, "pure-DoT profile module is missing");
requireText(numeric, /abilities\.text\(abilityIndex, "scalesWith"\) == "ranged"/, "pure-DoT ranged scaling is missing");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /OFFLINE_SOURCE_MIN_GLOBAL_COOLDOWN_SECONDS: float = 0\.75;/, "pure-DoT global cooldown floor is missing");
requireText(world, /offlineDotRanks[\s\S]*?offlineDotSnapshotPowers/, "WOS63 DoT snapshot columns are missing");
requireText(world, /writer\.u16\(<uint>78, 1, 1\)[\s\S]*?offlineDotSnapshotPowers/, "WOS63 DoT snapshot tail is missing");
requireText(world, /schemaVersion != <uint>64[\s\S]*?schemaVersion >= <uint>63/, "WOS63 DoT decoder migration is missing");
requireText(world, /serpentStingAbilityCode\([\s\S]*?shadowWordPainAbilityCode\([\s\S]*?startOfflineSerpentStingCast[\s\S]*?startOfflineShadowWordPainCast/, "instant DoT cast reducers are missing");
requireText(world, /appendOfflineAbilityProjectile\([\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW/, "instant DoT projectile admission is missing");
requireText(world, /landOfflinePureDotProjectile[\s\S]*?spellResist\.resolve[\s\S]*?resolvePureDotProfile[\s\S]*?landOfflineSerpentStingProjectile[\s\S]*?landOfflineShadowWordPainProjectile/, "instant DoT landing reducer is missing");
requireText(world, /offlineDotStateIsValid[\s\S]*?pureDotProfileMatches/, "WOS63 DoT snapshot validation is missing");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?serpentStingAbilityCode\(\)[\s\S]*?shadowWordPainAbilityCode\(\)/, "instant DoT projectile dispatch is missing");
requireText(world, /pub instantDotCommandStateTest\(\): int[\s\S]*?serpent_sting[\s\S]*?shadow_word_pain/, "instant DoT state regression coverage is missing");

const main = read("scripts", "woc_game", "src", "main.zr");
if (!/\\"world_state\\":\\"WOS78\\"/.test(main)) {
  throw new Error("package state identity must retain WOS64");
}
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
if (!protocol.includes('WORLD_STATE_FORMAT: &str = "WOS78"') ||
    !protocol.includes("WORLD_STATE_SCHEMA_VERSION: u16 = 78")) {
  throw new Error("native state identity must retain WOS74");
}

process.stdout.write(`WOS84 instant DoT static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
