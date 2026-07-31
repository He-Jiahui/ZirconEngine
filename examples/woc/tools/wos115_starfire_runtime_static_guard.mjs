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
const start = classes.indexOf("  starfire: {");
const end = classes.indexOf("  travel_form: {", start);
const starfire = classes.slice(start, end);
for (const needle of [
  "class: 'druid'", "learnLevel: 14", "cost: 80", "castTime: 3.0",
  "cooldown: 0", "range: 30", "school: 'arcane'", "requiresTarget: true",
  "type: 'directDamage', min: 80, max: 112",
]) {
  if (!starfire.includes(needle)) throw new Error(`source Starfire drifted: ${needle}`);
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
if (!/healing_touch',[\s\S]*?'starfire'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Starfire projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "starfire",
);
if (!entry || entry.index !== 55 || entry.definition.class !== "druid" ||
    entry.definition.school !== "arcane" || entry.definition.learnLevel !== 14 ||
    entry.definition.cost !== 80 || entry.definition.castTime !== 3 ||
    entry.definition.cooldown !== 0 || entry.definition.range !== 30 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "directDamage" ||
    entry.definition.effects[0].min !== 80 || entry.definition.effects[0].max !== 112 ||
    (entry.definition.ranks?.length ?? 0) !== 0) {
  throw new Error("M4 Starfire projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /starfireAbilityCode\([\s\S]*?starfirePayloadAbilityIsExact[\s\S]*?starfireProjectileProfileIsValid/,
  "Starfire identity and snapshot profile are missing");
requireText(world, /startOfflineStarfireCast[\s\S]*?starfireTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?starfireGlobalCooldownSeconds/,
  "Starfire cast admission is missing");
requireText(world, /completeOfflineStarfireCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_ARCANE/,
  "Starfire completion must queue an Arcane projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?starfireAbilityCode\([\s\S]*?starfireProjectileProfileIsValid/,
  "Starfire in-flight state validation is missing");
requireText(world, /landOfflineStarfireProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "arcane"[\s\S]*?timedSpell\.resolveTimedSpellHit/,
  "Starfire landing must resolve one resist followed by direct Arcane damage");
requireText(world, /stepRetainedCasting[\s\S]*?starfireAbilityCode\(\)[\s\S]*?completeOfflineStarfireCast/,
  "Starfire completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?starfireAbilityCode\(\)[\s\S]*?landOfflineStarfireProjectile/,
  "Starfire projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?starfireAbilityCode\(\)[\s\S]*?startOfflineStarfireCast[\s\S]*?applySupportedCastCommand[\s\S]*?starfirePayloadAbilityIsExact/,
  "Starfire command routes are missing");
requireText(world, /pub starfireCommandStateTest\(\): int[\s\S]*?m4AbilityCatalog\.indexOf\("starfire"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Starfire state regression coverage is missing");
requireText(world, /if \(starfireCommandStateTest\(\) != 1\) \{[\s\S]*?return -109;/,
  "world selfTest must execute Starfire");

process.stdout.write(`WOS115 Starfire static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
