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
const start = classes.indexOf("  searing_pain: {");
const end = classes.indexOf("  shadowburn: {", start);
const searingPain = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 14", "cost: 35", "castTime: 1.5",
  "cooldown: 0", "range: 30", "school: 'fire'", "requiresTarget: true",
  "type: 'directDamage', min: 30, max: 38",
]) {
  if (!searingPain.includes(needle)) throw new Error(`source Searing Pain drifted: ${needle}`);
}
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source spell projectile and resist ordering drifted",
);
requireText(
  dispatch,
  /case 'directDamage':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?directHitBonus\([\s\S]*?ctx\.rng\.chance[\s\S]*?Math\.round\(dmg\)/,
  "source direct spell damage ordering drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/curse_of_agony',[\s\S]*?'searing_pain'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Searing Pain projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "searing_pain");
if (!entry || entry.index !== 46 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "fire" || entry.definition.learnLevel !== 14 ||
    entry.definition.cost !== 35 || entry.definition.castTime !== 1.5 ||
    entry.definition.cooldown !== 0 || !entry.definition.requiresTarget ||
    entry.definition.effects?.[0]?.type !== "directDamage" ||
    entry.definition.effects[0].min !== 30 || entry.definition.effects[0].max !== 38 ||
    (entry.definition.ranks?.length ?? 0) !== 0) {
  throw new Error("M4 Searing Pain projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /searingPainAbilityCode\([\s\S]*?searingPainPayloadAbilityIsExact[\s\S]*?searingPainProjectileProfileIsValid/, "Searing Pain identity and snapshot profile are missing");
requireText(world, /startOfflineSearingPainCast[\s\S]*?searingPainTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?searingPainGlobalCooldownSeconds/, "Searing Pain cast admission is missing");
requireText(world, /completeOfflineSearingPainCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/, "Searing Pain completion must queue a Fire projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?searingPainAbilityCode\([\s\S]*?searingPainProjectileProfileIsValid/, "Searing Pain in-flight state validation is missing");
requireText(world, /landOfflineSearingPainProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "fire"[\s\S]*?timedSpell\.resolveTimedSpellHit/, "Searing Pain landing must resolve one resist followed by direct Fire damage");
requireText(world, /stepRetainedCasting[\s\S]*?searingPainAbilityCode\(\)[\s\S]*?completeOfflineSearingPainCast/, "Searing Pain completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?searingPainAbilityCode\(\)[\s\S]*?landOfflineSearingPainProjectile/, "Searing Pain projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?searingPainAbilityCode\(\)[\s\S]*?startOfflineSearingPainCast[\s\S]*?applySupportedCastCommand[\s\S]*?searingPainPayloadAbilityIsExact/, "Searing Pain command routes are missing");
requireText(world, /pub searingPainCommandStateTest\(\): int[\s\S]*?searing_pain[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/, "Searing Pain state regression coverage is missing");

process.stdout.write(`WOS106 Searing Pain static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
