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
const start = classes.indexOf("  growl: {");
const end = classes.indexOf("  demoralizing_roar: {", start);
const growl = classes.slice(start, end);
for (const needle of [
  "name: 'Menace'", "class: 'druid'", "learnLevel: 10", "cost: 0",
  "castTime: 0", "cooldown: 10", "range: 8", "school: 'physical'",
  "requiresTarget: true", "offGcd: true", "requiresForm: 'bear'",
  "type: 'taunt'",
]) {
  if (!growl.includes(needle)) throw new Error(`source Growl drifted: ${needle}`);
}

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/maul',[\s\S]*?'growl'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Growl projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "growl",
);
if (!entry || entry.index !== 58 || entry.definition.class !== "druid" ||
    entry.definition.learnLevel !== 10 || entry.definition.cost !== 0 ||
    entry.definition.castTime !== 0 || entry.definition.cooldown !== 10 ||
    entry.definition.range !== 8 || entry.definition.school !== "physical" ||
    !entry.definition.requiresTarget || !entry.definition.offGcd ||
    entry.definition.requiresForm !== "bear" ||
    entry.definition.effects?.[0]?.type !== "taunt") {
  throw new Error("M4 Growl projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /growlAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("growl"\)/,
  "Growl catalog identity is missing");
requireText(world, /startOfflineGrowlCast[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?completeOfflineTauntCast/,
  "Growl must use Bear-form admission and the shared Taunt settlement");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?growlAbilityCode\(\)[\s\S]*?startOfflineGrowlCast/,
  "Growl action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?growlPayloadAbilityIsExact[\s\S]*?startOfflineGrowlCast/,
  "Growl typed routing is missing");
requireText(world, /pub growlCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Growl state regression coverage is missing");
requireText(world, /if \(growlCommandStateTest\(\) != 1\) \{[\s\S]*?return -112;/,
  "world selfTest must execute Growl");

process.stdout.write(`WOS118 Growl static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
