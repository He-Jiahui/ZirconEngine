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
const start = classes.indexOf("  ferocious_bite: {");
const end = classes.indexOf("  barkskin: {", start);
if (start < 0 || end < start) throw new Error("source Ferocious Bite block is missing");
const ferociousBite = classes.slice(start, end);
for (const needle of [
  "name: 'Gorebite'", "class: 'druid'", "learnLevel: 14", "cost: 35",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: true", "spendsCombo: true", "requiresForm: 'cat'",
  "type: 'finisherDamage', base: 10, perCombo: 14, variance: 6",
]) {
  if (!ferociousBite.includes(needle)) {
    throw new Error(`source Ferocious Bite drifted: ${needle}`);
  }
}
requireText(effects, /const spentCombo = ability\.spendsCombo \? p\.comboPoints : 0;/,
  "source Ferocious Bite combo snapshot drifted");
requireText(effects, /if \(!preservesStealth\(ability\)\) ctx\.breakStealth\(p\)/,
  "source Ferocious Bite reveal ordering drifted");
requireText(effects, /case 'finisherDamage':[\s\S]*?eff\.base[\s\S]*?eff\.perCombo \* spentCombo[\s\S]*?ctx\.rng\.range\(0, eff\.variance\)[\s\S]*?ctx\.effectiveAttackPower\(p\) \/ 14[\s\S]*?armorReduction[\s\S]*?ctx\.dealDamage/,
  "source Ferocious Bite finisher dispatch drifted");
requireText(effects, /if \(ability\.spendsCombo && spentCombo > 0\)[\s\S]*?p\.comboPoints = 0;/,
  "source Ferocious Bite combo consumption drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/claw',[\s\S]*?'ferocious_bite'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Ferocious Bite projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "ferocious_bite",
);
if (!entry || entry.index !== 63 || entry.definition.cost !== 35 ||
    entry.definition.requiresForm !== "cat" || !entry.definition.spendsCombo ||
    entry.definition.effects?.[0]?.type !== "finisherDamage" ||
    entry.definition.effects[0].base !== 10 || entry.definition.effects[0].perCombo !== 14 ||
    entry.definition.effects[0].variance !== 6) {
  throw new Error("M4 Ferocious Bite projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /ferociousBiteAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("ferocious_bite"\)/,
  "Ferocious Bite catalog identity is missing");
requireText(world, /startOfflineFerociousBiteCast[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?clearOfflineProwl[\s\S]*?finisherDamage[\s\S]*?entityComboPoints\[casterIndex\] = 0/,
  "Ferocious Bite Cat finisher reducer is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?ferociousBiteAbilityCode\(\)[\s\S]*?startOfflineFerociousBiteCast/,
  "Ferocious Bite action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?ferociousBitePayloadAbilityIsExact[\s\S]*?startOfflineFerociousBiteCast/,
  "Ferocious Bite typed routing is missing");
requireText(world, /pub ferociousBiteCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_cat"\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Ferocious Bite state regression coverage is missing");
requireText(world, /if \(ferociousBiteCommandStateTest\(\) != 1\) \{[\s\S]*?return -117;/,
  "world selfTest must execute Ferocious Bite");

process.stdout.write(`WOS123 Ferocious Bite static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
