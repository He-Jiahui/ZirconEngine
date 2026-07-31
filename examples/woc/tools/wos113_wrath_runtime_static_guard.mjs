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
const start = classes.indexOf("  wrath: {");
const end = classes.indexOf("  healing_touch: {", start);
const wrath = classes.slice(start, end);
for (const needle of [
  "class: 'druid'", "learnLevel: 1", "cost: 20", "castTime: 1.5",
  "cooldown: 0", "range: 30", "school: 'nature'", "requiresTarget: true",
  "type: 'directDamage', min: 13, max: 16", "rank: 2", "level: 8",
  "cost: 32", "castTime: 2.0", "min: 24, max: 29", "rank: 3", "level: 14",
  "cost: 48", "min: 38, max: 45", "rank: 4", "level: 20", "cost: 70",
  "min: 60, max: 71",
]) {
  if (!wrath.includes(needle)) throw new Error(`source Wrath drifted: ${needle}`);
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
if (!/swiftmend',[\s\S]*?'wrath'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Wrath projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "wrath",
);
if (!entry || entry.index !== 53 || entry.definition.class !== "druid" ||
    entry.definition.school !== "nature" || entry.definition.cost !== 20 ||
    entry.definition.castTime !== 1.5 || entry.definition.cooldown !== 0 ||
    entry.definition.range !== 30 || !entry.definition.requiresTarget ||
    entry.definition.effects?.[0]?.type !== "directDamage" ||
    entry.definition.effects[0].min !== 13 || entry.definition.effects[0].max !== 16 ||
    entry.definition.ranks?.[3 - 2]?.effects?.[0]?.min !== 38 ||
    entry.definition.ranks?.[4 - 2]?.effects?.[0]?.max !== 71) {
  throw new Error("M4 Wrath projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /wrathAbilityCode\([\s\S]*?wrathPayloadAbilityIsExact[\s\S]*?wrathProjectileProfileIsValid/,
  "Wrath identity and snapshot profile are missing");
requireText(world, /startOfflineWrathCast[\s\S]*?wrathTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?wrathGlobalCooldownSeconds/,
  "Wrath cast admission is missing");
requireText(world, /completeOfflineWrathCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE/,
  "Wrath completion must queue a Nature projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?wrathAbilityCode\([\s\S]*?wrathProjectileProfileIsValid/,
  "Wrath in-flight state validation is missing");
requireText(world, /landOfflineWrathProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "nature"[\s\S]*?timedSpell\.resolveTimedSpellHit/,
  "Wrath landing must resolve one resist followed by direct Nature damage");
requireText(world, /stepRetainedCasting[\s\S]*?wrathAbilityCode\(\)[\s\S]*?completeOfflineWrathCast/,
  "Wrath completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?wrathAbilityCode\(\)[\s\S]*?landOfflineWrathProjectile/,
  "Wrath projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?wrathAbilityCode\(\)[\s\S]*?startOfflineWrathCast[\s\S]*?applySupportedCastCommand[\s\S]*?wrathPayloadAbilityIsExact/,
  "Wrath command routes are missing");
requireText(world, /pub wrathCommandStateTest\(\): int[\s\S]*?m4AbilityCatalog\.indexOf\("wrath"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Wrath state regression coverage is missing");
requireText(world, /if \(wrathCommandStateTest\(\) != 1\) \{[\s\S]*?return -107;/,
  "world selfTest must execute Wrath");

process.stdout.write(`WOS113 Wrath static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
