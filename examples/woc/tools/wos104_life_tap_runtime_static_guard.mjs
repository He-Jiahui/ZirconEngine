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
const start = classes.indexOf("  life_tap: {");
const end = classes.indexOf("  curse_of_agony: {", start);
const lifeTap = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 6", "cost: 0", "castTime: 0", "cooldown: 0",
  "range: 0", "school: 'shadow'", "requiresTarget: false",
  "type: 'lifeTap', hp: 30, mana: 30", "rank: 2", "level: 14",
  "type: 'lifeTap', hp: 55, mana: 55", "rank: 3", "level: 20",
  "type: 'lifeTap', hp: 85, mana: 85",
]) {
  if (!lifeTap.includes(needle)) throw new Error(`source Life Tap drifted: ${needle}`);
}
requireText(
  dispatch,
  /case 'lifeTap':[\s\S]*?if \(p\.hp <= eff\.hp\)[\s\S]*?p\.hp -= eff\.hp;[\s\S]*?Math\.min\(p\.maxResource, p\.resource \+ tapMana\)/,
  "source Life Tap health gate or capped mana restoration drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/immolate',[\s\S]*?'corruption',[\s\S]*?'life_tap'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Life Tap projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "life_tap");
if (!entry || entry.index !== 44 || entry.definition.class !== "warlock" ||
    entry.definition.cost !== 0 || entry.definition.castTime !== 0 ||
    entry.definition.cooldown !== 0 || entry.definition.range !== 0 ||
    entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "lifeTap" ||
    entry.definition.effects[0].hp !== 30 || entry.definition.effects[0].mana !== 30) {
  throw new Error("M4 Life Tap projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /lifeTapAbilityCode\([\s\S]*?lifeTapPayloadAbilityIsExact/, "Life Tap identity is missing");
requireText(world, /lifeTapRankLevel[\s\S]*?return 6;[\s\S]*?return 14;[\s\S]*?return 20;/, "Life Tap rank mapping is missing");
requireText(world, /startOfflineLifeTapCast[\s\S]*?lifeTapGlobalCooldownSeconds[\s\S]*?healthCost[\s\S]*?entityHp\[casterIndex\] <= healthCost[\s\S]*?entityHp\[casterIndex\] = [\s\S]*?restoredResource[\s\S]*?entityResources\[casterIndex\] =/, "Life Tap health-to-mana resolution is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?lifeTapAbilityCode\(\)[\s\S]*?startOfflineLifeTapCast[\s\S]*?applySupportedCastCommand[\s\S]*?lifeTapPayloadAbilityIsExact/, "Life Tap command routes are missing");
requireText(world, /pub lifeTapCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?entityHp[\s\S]*?entityResources/, "Life Tap state regression coverage is missing");

process.stdout.write(`WOS104 Life Tap static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
