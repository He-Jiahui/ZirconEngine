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
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const start = classes.indexOf("  corruption: {");
const end = classes.indexOf("  life_tap: {", start);
const corruption = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 4", "cost: 35", "castTime: 2.0",
  "cooldown: 0", "range: 30", "school: 'shadow'", "requiresTarget: true",
  "type: 'dot', total: 40, duration: 18, interval: 3", "rank: 2", "level: 12",
  "cost: 55", "total: 72, duration: 18, interval: 3", "rank: 3", "level: 18",
  "cost: 75", "total: 85, duration: 18, interval: 3",
]) {
  if (!corruption.includes(needle)) throw new Error(`source Corruption drifted: ${needle}`);
}
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source Corruption projectile and resist ordering drifted",
);
requireText(
  dispatch,
  /case 'dot':[\s\S]*?const hybrid = res\.effects\.some[\s\S]*?const dotSp = !hybrid[\s\S]*?dotTickBonus\(abilityScalingPower\(p, ability\), ability, eff\.duration, eff\.interval\)/,
  "source pure DoT snapshot scaling drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/immolate',[\s\S]*?'corruption'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Corruption projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "corruption");
if (!entry || entry.index !== 43 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "shadow" || entry.definition.cost !== 35 ||
    entry.definition.castTime !== 2 || entry.definition.cooldown !== 0 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "dot" ||
    entry.definition.effects[0].total !== 40 || entry.definition.effects[0].duration !== 18 ||
    entry.definition.effects[0].interval !== 3) {
  throw new Error("M4 Corruption projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /corruptionAbilityCode\([\s\S]*?corruptionPayloadAbilityIsExact/, "Corruption identity is missing");
requireText(world, /pureDotRankLevel\([\s\S]*?corruptionAbilityCode\([\s\S]*?rank == <uint>2\) \{ return 12; \}[\s\S]*?rank == <uint>3\) \{ return 18; \}/, "Corruption pure-DoT rank mapping is missing");
requireText(world, /pureDotAbilityIndex[\s\S]*?corruptionAbilityCode\([\s\S]*?pureDotProjectileProfileIsValid[\s\S]*?castTime == m4AbilityCatalog\.metric/, "Corruption projectile profile must retain its cast time");
requireText(world, /startOfflineCorruptionCast[\s\S]*?corruptionTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?corruptionGlobalCooldownSeconds/, "Corruption hard-cast admission is missing");
requireText(world, /completeOfflineCorruptionCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW/, "Corruption completion must queue a Shadow projectile");
requireText(world, /stepRetainedCasting[\s\S]*?corruptionAbilityCode\(\)[\s\S]*?completeOfflineCorruptionCast/, "Corruption completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?corruptionAbilityCode\(\)[\s\S]*?landOfflineCorruptionProjectile/, "Corruption projectile landing must be dispatched");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?pureDotAbilityIndex\(abilityCode\)/, "pure DoT periodic threat must use the shared ability index");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?corruptionAbilityCode\(\)[\s\S]*?startOfflineCorruptionCast[\s\S]*?applySupportedCastCommand[\s\S]*?corruptionPayloadAbilityIsExact/, "Corruption command routes are missing");
requireText(world, /pub corruptionCommandStateTest\(\): int[\s\S]*?corruption[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepOfflineEastbrookDots/, "Corruption state regression coverage is missing");

process.stdout.write(`WOS103 Corruption static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
