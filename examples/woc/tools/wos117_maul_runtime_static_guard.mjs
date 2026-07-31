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
const start = classes.indexOf("  maul: {");
const end = classes.indexOf("  growl: {", start);
const maul = classes.slice(start, end);
for (const needle of [
  "name: 'Bonecrush'", "class: 'druid'", "learnLevel: 10", "cost: 15",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: true", "onNextSwing: true", "offGcd: true",
  "requiresForm: 'bear'", "flat: 35", "type: 'weaponDamage', bonus: 18",
  "rank: 2", "level: 16", "threatFlat: 50", "type: 'weaponDamage', bonus: 27",
]) {
  if (!maul.includes(needle)) throw new Error(`source Maul drifted: ${needle}`);
}
requireText(
  casting,
  /if \(ability\.requiresForm\)[\s\S]*?if \(!form \|\| form\.kind !== need\)[\s\S]*?if \(ability\.onNextSwing\)[\s\S]*?p\.queuedOnSwing === ability\.id[\s\S]*?p\.queuedOnSwing = toggledOff \? null : ability\.id/,
  "source Maul form admission and queued-swing lifecycle drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/entangling_roots',[\s\S]*?'maul'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Maul projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "maul",
);
if (!entry || entry.index !== 57 || entry.definition.class !== "druid" ||
    entry.definition.learnLevel !== 10 || entry.definition.cost !== 15 ||
    entry.definition.castTime !== 0 || entry.definition.cooldown !== 0 ||
    entry.definition.range !== 0 || entry.definition.school !== "physical" ||
    !entry.definition.requiresTarget || !entry.definition.onNextSwing ||
    !entry.definition.offGcd || entry.definition.requiresForm !== "bear" ||
    entry.definition.threat?.flat !== 35 ||
    entry.definition.effects?.[0]?.type !== "weaponDamage" ||
    entry.definition.effects[0].bonus !== 18 || entry.definition.ranks?.length !== 1 ||
    entry.definition.ranks[0].rank !== 2 || entry.definition.ranks[0].level !== 16 ||
    entry.definition.ranks[0].cost !== 15 || entry.definition.ranks[0].threatFlat !== 50 ||
    entry.definition.ranks[0].effects?.[0]?.type !== "weaponDamage" ||
    entry.definition.ranks[0].effects[0].bonus !== 27) {
  throw new Error("M4 Maul projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /queuedOnSwingPayloadAbilityCode[\s\S]*?knownAbilityCatalog\.abilityCode\("maul"\)/,
  "Maul typed command identity is missing",
);
requireText(
  world,
  /startQueuedOnSwingCast[\s\S]*?catalogAdmission\([\s\S]*?forms\.formKindForAbilityCode\([\s\S]*?<uint>state\.entityActiveFormAbilityCodes\[casterIndex\][\s\S]*?false\)/,
  "queued swings must pass the active form into catalog admission",
);
const queuedCastStart = world.slice(
  world.indexOf("startQueuedOnSwingCast("),
  world.indexOf("    var targetIndex =", world.indexOf("startQueuedOnSwingCast(")),
);
if (queuedCastStart.includes("entityActiveFormAbilityCodes[casterIndex] != <uint>0")) {
  throw new Error("queued swings still reject every active form before source admission");
}
requireText(
  world,
  /pub maulCommandStateTest\(\): int[\s\S]*?m4AbilityCatalog\.indexOf\("maul"\)[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?entityQueuedOnSwingAbilityCodes[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Maul state regression coverage is missing",
);
requireText(
  world,
  /if \(maulCommandStateTest\(\) != 1\) \{[\s\S]*?return -111;/,
  "world selfTest must execute Maul",
);

process.stdout.write(`WOS117 Maul static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
