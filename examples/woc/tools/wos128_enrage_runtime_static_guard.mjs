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
const start = classes.indexOf("  enrage: {");
const end = classes.indexOf("  bash: {", start);
if (start < 0 || end < start) throw new Error("source Enrage block is missing");
const enrage = classes.slice(start, end);
for (const needle of [
  "name: 'Stoke'", "class: 'druid'", "learnLevel: 16", "cost: 0",
  "castTime: 0", "cooldown: 60", "range: 0", "school: 'physical'",
  "requiresTarget: false", "offGcd: true", "requiresForm: 'bear'",
  "type: 'gainResource', amount: 20",
]) {
  if (!enrage.includes(needle)) throw new Error(`source Enrage drifted: ${needle}`);
}
requireText(lifecycle, /!ability\.offGcd && p\.gcdRemaining > 0/,
  "source Enrage off-GCD gate drifted");
requireText(lifecycle, /p\.resource = Math\.min\(p\.maxResource, p\.resource \+ eff\.amount\)/,
  "source Enrage resource-cap semantics drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/primal_reflexes',[\s\S]*?'enrage'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Enrage projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "enrage",
);
if (!entry || entry.index !== 68 || entry.definition.cost !== 0 ||
    entry.definition.cooldown !== 60 || !entry.definition.offGcd ||
    entry.definition.requiresForm !== "bear" ||
    entry.definition.effects?.[0]?.type !== "gainResource" ||
    entry.definition.effects[0].amount !== 20) {
  throw new Error("M4 Enrage projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /enrageAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("enrage"\)/,
  "Enrage catalog identity is missing");
requireText(world, /startOfflineEnrageCast[\s\S]*?entityCastingAbility[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?m4AbilityCatalog\.flag\(abilityIndex, "offGcd"\)[\s\S]*?entityMaxResources[\s\S]*?setAbilityCooldownExpiration/,
  "Enrage Bear-only resource reducer is missing");
const reducerStart = world.indexOf("startOfflineEnrageCast");
const reducerEnd = world.indexOf("\n}\n", reducerStart);
if (reducerStart < 0 || reducerEnd < reducerStart ||
    world.slice(reducerStart, reducerEnd).includes("entityCastGcdRemaining")) {
  throw new Error("Enrage must not consume or require the global cooldown");
}
requireText(world, /applySupportedCastSlotCommand[\s\S]*?enrageAbilityCode\(\)[\s\S]*?startOfflineEnrageCast/,
  "Enrage action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?enragePayloadAbilityIsExact[\s\S]*?startOfflineEnrageCast/,
  "Enrage typed routing is missing");
requireText(world, /pub enrageCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?appendTypedCastCommandForTest[\s\S]*?abilityCooldownExpiresAt/,
  "Enrage state regression coverage is missing");
requireText(world, /if \(enrageCommandStateTest\(\) != 1\) \{[\s\S]*?return -122;/,
  "world selfTest must execute Enrage");

process.stdout.write(`WOS128 Enrage static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
