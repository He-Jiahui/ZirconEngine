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
const start = classes.indexOf("  shadow_bolt: {");
const end = classes.indexOf("  demon_skin: {", start);
const shadowBolt = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 1", "cost: 25", "castTime: 1.7",
  "cooldown: 0", "range: 30", "school: 'shadow'", "requiresTarget: true",
  "type: 'directDamage', min: 13, max: 18", "rank: 2", "level: 8",
  "cost: 38", "castTime: 2.2", "min: 24, max: 31", "rank: 3", "level: 14",
  "cost: 55", "castTime: 2.7", "min: 42, max: 53", "rank: 4", "level: 20",
  "cost: 80", "castTime: 3.0", "min: 68, max: 84",
]) {
  if (!shadowBolt.includes(needle)) throw new Error(`source Shadow Bolt drifted: ${needle}`);
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
if (!/stormstrike',[\s\S]*?'shadow_bolt'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Shadow Bolt projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "shadow_bolt");
if (!entry || entry.index !== 41 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "shadow" || entry.definition.cost !== 25 ||
    entry.definition.castTime !== 1.7 || entry.definition.cooldown !== 0 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "directDamage" ||
    entry.definition.effects[0].min !== 13 || entry.definition.effects[0].max !== 18) {
  throw new Error("M4 Shadow Bolt projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /shadowBoltAbilityCode\([\s\S]*?shadowBoltPayloadAbilityIsExact[\s\S]*?shadowBoltProjectileProfileIsValid/, "Shadow Bolt identity and snapshot profile are missing");
requireText(world, /startOfflineShadowBoltCast[\s\S]*?shadowBoltTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?shadowBoltGlobalCooldownSeconds/, "Shadow Bolt cast admission is missing");
requireText(world, /completeOfflineShadowBoltCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW/, "Shadow Bolt completion must queue a Shadow projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?shadowBoltAbilityCode\([\s\S]*?shadowBoltProjectileProfileIsValid/, "Shadow Bolt in-flight state validation is missing");
requireText(world, /landOfflineShadowBoltProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "shadow"[\s\S]*?timedSpell\.resolveTimedSpellHit/, "Shadow Bolt landing must resolve one resist followed by direct Shadow damage");
requireText(world, /stepRetainedCasting[\s\S]*?shadowBoltAbilityCode\(\)[\s\S]*?completeOfflineShadowBoltCast/, "Shadow Bolt completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?shadowBoltAbilityCode\(\)[\s\S]*?landOfflineShadowBoltProjectile/, "Shadow Bolt projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?shadowBoltAbilityCode\(\)[\s\S]*?startOfflineShadowBoltCast[\s\S]*?applySupportedCastCommand[\s\S]*?shadowBoltPayloadAbilityIsExact/, "Shadow Bolt command routes are missing");
requireText(world, /pub shadowBoltCommandStateTest\(\): int[\s\S]*?shadow_bolt[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/, "Shadow Bolt state regression coverage is missing");

process.stdout.write(`WOS101 Shadow Bolt static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
