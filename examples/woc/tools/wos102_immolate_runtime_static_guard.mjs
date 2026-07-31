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
const start = classes.indexOf("  immolate: {");
const end = classes.indexOf("  corruption: {", start);
const immolate = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 1", "cost: 25", "castTime: 2.0",
  "cooldown: 0", "range: 30", "school: 'fire'", "requiresTarget: true",
  "type: 'directDamage', min: 11, max: 11", "type: 'dot', total: 20, duration: 15, interval: 3",
  "rank: 2", "level: 10", "cost: 40", "min: 22, max: 22", "total: 35",
  "rank: 3", "level: 16", "cost: 60", "min: 38, max: 38", "total: 60",
]) {
  if (!immolate.includes(needle)) throw new Error(`source Immolate drifted: ${needle}`);
}
requireText(
  dispatch,
  /case 'dot':[\s\S]*?Fireball, Pyroblast, Immolate[\s\S]*?const hybrid = res\.effects\.some[\s\S]*?const dotSp = !hybrid[\s\S]*?: 0;[\s\S]*?tickTimer: eff\.interval/,
  "source hybrid DoT snapshot and cadence drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/shadow_bolt',[\s\S]*?'immolate'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Immolate projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "immolate");
if (!entry || entry.index !== 42 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "fire" || entry.definition.cost !== 25 ||
    entry.definition.castTime !== 2 || entry.definition.cooldown !== 0 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "directDamage" ||
    entry.definition.effects[0].min !== 11 || entry.definition.effects[0].max !== 11 ||
    entry.definition.effects?.[1]?.type !== "dot" || entry.definition.effects[1].total !== 20 ||
    entry.definition.effects[1].duration !== 15 || entry.definition.effects[1].interval !== 3) {
  throw new Error("M4 Immolate projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /immolateAbilityCode\([\s\S]*?immolatePayloadAbilityIsExact[\s\S]*?immolateProjectileProfileIsValid[\s\S]*?immolateDotProfileIsValid/, "Immolate identity and profiles are missing");
requireText(world, /startOfflineImmolateCast[\s\S]*?immolateTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?immolateGlobalCooldownSeconds/, "Immolate cast admission is missing");
requireText(world, /completeOfflineImmolateCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/, "Immolate completion must queue a Fire projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?immolateAbilityCode\([\s\S]*?immolateProjectileProfileIsValid/, "Immolate in-flight state validation is missing");
requireText(world, /offlineDotStateIsValid[\s\S]*?immolateAbilityCode\([\s\S]*?immolateDotProfileIsValid/, "Immolate DoT state validation is missing");
requireText(world, /landOfflineImmolateProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?applyOfflineImmolateDot/, "Immolate landing must apply the DoT only after a live direct hit");
requireText(world, /stepRetainedCasting[\s\S]*?immolateAbilityCode\(\)[\s\S]*?completeOfflineImmolateCast/, "Immolate completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?immolateAbilityCode\(\)[\s\S]*?landOfflineImmolateProjectile/, "Immolate projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?immolateAbilityCode\(\)[\s\S]*?startOfflineImmolateCast[\s\S]*?applySupportedCastCommand[\s\S]*?immolatePayloadAbilityIsExact/, "Immolate command routes are missing");
requireText(world, /pub immolateCommandStateTest\(\): int[\s\S]*?immolate[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepOfflineEastbrookDots/, "Immolate state regression coverage is missing");

process.stdout.write(`WOS102 Immolate static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
