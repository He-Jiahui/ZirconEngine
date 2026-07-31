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
const lifecycle = source("src/sim/combat/casting_lifecycle.ts");
const start = classes.indexOf("  barkskin: {");
const end = classes.indexOf("  primal_reflexes: {", start);
if (start < 0 || end < start) throw new Error("source Barkskin block is missing");
const barkskin = classes.slice(start, end);
for (const needle of [
  "name: 'Oakhide'", "class: 'druid'", "learnLevel: 16", "cost: 30",
  "castTime: 0", "cooldown: 60", "range: 0", "school: 'nature'",
  "requiresTarget: false", "offGcd: true", "usableInForm: true",
  "type: 'selfBuff', kind: 'buff_armor', value: 150, duration: 15",
]) {
  if (!barkskin.includes(needle)) throw new Error(`source Barkskin drifted: ${needle}`);
}
requireText(lifecycle, /!ability\.offGcd && p\.gcdRemaining > 0/,
  "source Barkskin off-GCD gate drifted");
requireText(lifecycle, /form && !isFormToggle\(ability\) && !ability\.usableInForm/,
  "source Barkskin form-admission gate drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/regrowth',[\s\S]*?'barkskin'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Barkskin projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "barkskin",
);
if (!entry || entry.index !== 66 || entry.definition.cost !== 30 ||
    entry.definition.cooldown !== 60 || !entry.definition.offGcd ||
    !entry.definition.usableInForm || entry.definition.effects?.[0]?.type !== "selfBuff" ||
    entry.definition.effects[0].kind !== "buff_armor" ||
    entry.definition.effects[0].value !== 150 || entry.definition.effects[0].duration !== 15) {
  throw new Error("M4 Barkskin projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /barkskinAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("barkskin"\)/,
  "Barkskin catalog identity is missing");
requireText(world, /startOfflineBarkskinCast[\s\S]*?entityCastingAbility[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?m4AbilityCatalog\.flag\(abilityIndex, "offGcd"\)[\s\S]*?setAbilityCooldownExpiration[\s\S]*?applyOfflineMotionAuraWithDetails[\s\S]*?motionAuraKindCode\("buff_armor"\)/,
  "Barkskin off-GCD armor-aura reducer is missing");
const barkskinReducerStart = world.indexOf("startOfflineBarkskinCast");
const barkskinReducerEnd = world.indexOf("\n}\n", barkskinReducerStart);
if (barkskinReducerStart < 0 || barkskinReducerEnd < barkskinReducerStart ||
    world.slice(barkskinReducerStart, barkskinReducerEnd).includes("entityCastGcdRemaining")) {
  throw new Error("Barkskin must not consume or require the global cooldown");
}
requireText(world, /barkskinArmorBonus[\s\S]*?barkskinAbilityCode[\s\S]*?value != 150\.0/,
  "Barkskin effective-armor profile is missing");
requireText(world, /effectiveOfflineArmor[\s\S]*?demonSkinArmorBonus[\s\S]*?barkskinArmorBonus/,
  "Barkskin armor bonus is not connected to physical mitigation");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?barkskinAbilityCode\(\)[\s\S]*?startOfflineBarkskinCast/,
  "Barkskin action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?barkskinPayloadAbilityIsExact[\s\S]*?startOfflineBarkskinCast/,
  "Barkskin typed routing is missing");
requireText(world, /pub barkskinCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?appendTypedCastCommandForTest[\s\S]*?abilityCooldownExpiresAt/,
  "Barkskin state regression coverage is missing");
requireText(world, /if \(barkskinCommandStateTest\(\) != 1\) \{[\s\S]*?return -120;/,
  "world selfTest must execute Barkskin");

process.stdout.write(`WOS126 Barkskin static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
