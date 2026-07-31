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
const start = classes.indexOf("  claw: {");
const end = classes.indexOf("  ferocious_bite: {", start);
if (start < 0 || end < start) throw new Error("source Claw block is missing");
const claw = classes.slice(start, end);
for (const needle of [
  "name: 'Claw'", "class: 'druid'", "learnLevel: 5", "cost: 45",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: true", "awardsCombo: 1", "requiresForm: 'cat'",
  "type: 'weaponStrike', bonus: 12", "rank: 2", "level: 18",
  "cost: 45", "type: 'weaponStrike', bonus: 20",
]) {
  if (!claw.includes(needle)) throw new Error(`source Claw drifted: ${needle}`);
}
requireText(effects, /case 'weaponStrike':[\s\S]*?if \(hit && ability\.awardsCombo\)/,
  "source Claw strike/combo dispatch drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/rake',[\s\S]*?'claw'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Claw projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "claw",
);
if (!entry || entry.index !== 62 || entry.definition.cost !== 45 ||
    entry.definition.requiresForm !== "cat" || entry.definition.awardsCombo !== 1 ||
    entry.definition.effects?.[0]?.type !== "weaponStrike" ||
    entry.definition.effects[0].bonus !== 12 || entry.definition.ranks?.[0]?.effects?.[0]?.bonus !== 20) {
  throw new Error("M4 Claw projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /clawAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("claw"\)/,
  "Claw catalog identity is missing");
requireText(world, /startOfflineClawCast[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?weaponStrike[\s\S]*?awardsCombo/,
  "Claw Cat strike reducer is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?clawAbilityCode\(\)[\s\S]*?startOfflineClawCast/,
  "Claw action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?clawPayloadAbilityIsExact[\s\S]*?startOfflineClawCast/,
  "Claw typed routing is missing");
requireText(world, /pub clawCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_cat"\)[\s\S]*?appendTypedCastCommandForTest/,
  "Claw state regression coverage is missing");
requireText(world, /if \(clawCommandStateTest\(\) != 1\) \{[\s\S]*?return -116;/,
  "world selfTest must execute Claw");

process.stdout.write(`WOS122 Claw static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
