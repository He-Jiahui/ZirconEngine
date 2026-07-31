import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const commit = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = path.resolve(root, "..", "..", "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const source = (file) => execFileSync("git", ["-C", sourceRoot, "show", `${commit}:${file}`], { encoding: "utf8" });
const requireText = (text, pattern, message) => { if (!pattern.test(text)) throw new Error(message); };

const classes = source("src/sim/content/classes.ts");
const start = classes.indexOf("  demon_skin: {");
const end = classes.indexOf("  immolate: {", start);
const demonSkin = classes.slice(start, end);
for (const needle of ["class: 'warlock'", "learnLevel: 1", "cost: 20", "castTime: 0", "cooldown: 0", "range: 0", "kind: 'buff_armor', value: 30, duration: 1800", "rank: 2", "level: 12", "cost: 35", "value: 55", "rank: 3", "level: 20", "cost: 50", "value: 80"]) {
  if (!demonSkin.includes(needle)) throw new Error(`source Demon Skin drifted: ${needle}`);
}
const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/shadowburn',[\s\S]*?'demon_skin'/.test(generator) || !generator.includes("EXPECTED_ABILITY_COUNT = 79") || !zrGenerator.includes("document.entries.length === 79")) throw new Error("M4 Demon Skin scope is missing");
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find((item) => item.id === "demon_skin");
if (!entry || entry.index !== 48 || entry.definition.effects?.[0]?.kind !== "buff_armor" || entry.definition.ranks?.[1]?.effects?.[0]?.value !== 80) throw new Error("M4 Demon Skin projection drifted");
const cc = read("scripts", "woc_game", "src", "generated", "cc_contract.zr");
if (!cc.includes('if (kind == "buff_armor") { return <uint>9; }')) throw new Error("buff_armor aura kind is missing");
const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /demonSkinAbilityCode\([\s\S]*?demonSkinPayloadAbilityIsExact[\s\S]*?demonSkinProfileIsValid/, "Demon Skin identity/profile is missing");
requireText(world, /startOfflineDemonSkinCast[\s\S]*?gcdRemaining[\s\S]*?entityResources[\s\S]*?applyOfflineMotionAuraWithDetails[\s\S]*?motionAuraKindCode\("buff_armor"\)/, "Demon Skin instant aura application is missing");
requireText(world, /demonSkinMotionStateIsValid[\s\S]*?1800\.0[\s\S]*?80\.0/, "Demon Skin persisted aura validation is missing");
requireText(world, /effectiveOfflineArmor[\s\S]*?demonSkinArmorBonus[\s\S]*?resolveOfflineEastbrookMobSwingRequests[\s\S]*?effectiveOfflineArmor\(state, playerIndex\)/, "Demon Skin must affect physical player defense");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?demonSkinAbilityCode\(\)[\s\S]*?startOfflineDemonSkinCast[\s\S]*?applySupportedCastCommand[\s\S]*?demonSkinPayloadAbilityIsExact/, "Demon Skin command routes are missing");
requireText(world, /pub demonSkinCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?ageMotionAurasForEntity/, "Demon Skin regression coverage is missing");
process.stdout.write(`WOS108 Demon Skin static guards passed (${commit.slice(0, 15)})\n`);
