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
const casting = source("src/sim/combat/casting_lifecycle.ts");
const abilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const smiteSource = abilityBlock("smite", "lesser_heal");
for (const needle of [
  "class: 'priest'", "learnLevel: 1", "cost: 20", "castTime: 2.0", "cooldown: 0",
  "range: 30", "school: 'holy'", "type: 'directDamage', min: 15, max: 20",
  "rank: 2", "level: 8", "cost: 32", "min: 26, max: 33",
  "rank: 3", "level: 14", "cost: 48", "castTime: 2.5", "min: 42, max: 52",
  "rank: 4", "level: 20", "cost: 70", "min: 64, max: 78",
]) {
  if (!smiteSource.includes(needle)) throw new Error(`source Smite drifted: ${needle}`);
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

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'power_word_shield',[\s\S]*?'smite'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Smite projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const smite = m4.entries.find((entry) => entry.id === "smite");
if (!smite || smite.index !== 27 || smite.scenarios.length !== 0 ||
    smite.definition.school !== "holy" || smite.definition.castTime !== 2 ||
    smite.definition.effects?.[0]?.type !== "directDamage" ||
    smite.definition.effects?.[0]?.min !== 15 || smite.definition.effects?.[0]?.max !== 20) {
  throw new Error("M4 Smite source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /smiteAbilityCode\([\s\S]*?startOfflineSmiteCast[\s\S]*?completeOfflineSmiteCast/, "Smite cast reducer is missing");
requireText(world, /startOfflineSmiteCast[\s\S]*?cast\.armTimed[\s\S]*?completeOfflineSmiteCast[\s\S]*?appendOfflineAbilityProjectile/, "Smite must defer cost to successful cast completion and queue a projectile");
requireText(world, /landOfflineSmiteProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_HOLY/, "Smite landing must resolve one resist followed by direct-spell damage");
requireText(world, /stepRetainedCasting[\s\S]*?smiteAbilityCode\(\)[\s\S]*?completeOfflineSmiteCast/, "Smite completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?smiteAbilityCode\(\)[\s\S]*?landOfflineSmiteProjectile/, "Smite projectile landing must be dispatched");
requireText(world, /pub smiteCommandStateTest\(\): int[\s\S]*?smite[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepRetainedCasting[\s\S]*?stepOfflineEastbrookProjectiles/, "Smite state regression coverage is missing");

process.stdout.write(`WOS87 Smite static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
