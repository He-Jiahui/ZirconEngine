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
const effects = source("src/sim/combat/effect_dispatch.ts");
const start = classes.indexOf("  swipe: {");
const end = classes.indexOf("  regrowth: {", start);
if (start < 0 || end < start) throw new Error("source Swipe block is missing");
const swipe = classes.slice(start, end);
for (const needle of [
  "name: 'Sweeping Claws'", "class: 'druid'", "learnLevel: 16", "cost: 20",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: false", "requiresForm: 'bear'", "mult: 1.75",
  "type: 'aoeDamage', min: 12, max: 15, radius: 5",
]) {
  if (!swipe.includes(needle)) throw new Error(`source Swipe drifted: ${needle}`);
}
requireText(effects, /case 'aoeDamage':[\s\S]*?ctx\.hostilesInRadius[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?if \(!isSpell\) dmg \*= 1 - armorReduction[\s\S]*?ctx\.dealDamage/,
  "source Swipe AoE/armor dispatch drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/ferocious_bite',[\s\S]*?'swipe'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Swipe projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "swipe",
);
if (!entry || entry.index !== 64 || entry.definition.cost !== 20 ||
    entry.definition.requiresTarget || entry.definition.requiresForm !== "bear" ||
    entry.definition.threat?.mult !== 1.75 ||
    entry.definition.effects?.[0]?.type !== "aoeDamage" ||
    entry.definition.effects[0].min !== 12 || entry.definition.effects[0].max !== 15 ||
    entry.definition.effects[0].radius !== 5) {
  throw new Error("M4 Swipe projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /swipeAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("swipe"\)/,
  "Swipe catalog identity is missing");
requireText(world, /startOfflineSwipeCast[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?aoeDamage[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?armorReductionFromArmor[\s\S]*?threatMult/,
  "Swipe Bear physical AoE reducer is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?swipeAbilityCode\(\)[\s\S]*?startOfflineSwipeCast/,
  "Swipe action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?swipePayloadAbilityIsExact[\s\S]*?startOfflineSwipeCast/,
  "Swipe typed routing is missing");
requireText(world, /pub swipeCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?appendTypedCastCommandForTest/,
  "Swipe state regression coverage is missing");
requireText(world, /if \(swipeCommandStateTest\(\) != 1\) \{[\s\S]*?return -118;/,
  "world selfTest must execute Swipe");

process.stdout.write(`WOS124 Swipe static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
