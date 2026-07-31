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
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const start = classes.indexOf("  curse_of_agony: {");
const end = classes.indexOf("  drain_life: {", start);
const curse = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 8", "cost: 25", "castTime: 0", "cooldown: 0",
  "range: 30", "school: 'shadow'", "requiresTarget: true",
  "type: 'dot', total: 36, duration: 24, interval: 3", "rank: 2", "level: 14",
  "cost: 40", "total: 72, duration: 24, interval: 3", "rank: 3", "level: 20",
  "cost: 60", "total: 78, duration: 24, interval: 3",
]) {
  if (!curse.includes(needle)) throw new Error(`source Curse of Agony drifted: ${needle}`);
}
requireText(casting, /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?isSpellResisted\(/, "source Curse of Agony projectile ordering drifted");
requireText(dispatch, /case 'dot':[\s\S]*?const dotSp = !hybrid[\s\S]*?dotTickBonus\(abilityScalingPower\(p, ability\)/, "source pure DoT snapshot scaling drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/corruption',[\s\S]*?'life_tap',[\s\S]*?'curse_of_agony'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Curse of Agony projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "curse_of_agony");
if (!entry || entry.index !== 45 || entry.definition.class !== "warlock" ||
    entry.definition.cost !== 25 || entry.definition.castTime !== 0 ||
    entry.definition.range !== 30 || entry.definition.school !== "shadow" ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "dot" ||
    entry.definition.effects[0].total !== 36 || entry.definition.effects[0].duration !== 24 ||
    entry.definition.effects[0].interval !== 3) {
  throw new Error("M4 Curse of Agony projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /curseOfAgonyAbilityCode\([\s\S]*?curseOfAgonyPayloadAbilityIsExact/, "Curse of Agony identity is missing");
requireText(world, /pureDotAbilityIndex[\s\S]*?curseOfAgonyAbilityCode\([\s\S]*?pureDotRankLevel[\s\S]*?curseOfAgonyAbilityCode\([\s\S]*?return 8;[\s\S]*?return 14;[\s\S]*?return 20;/, "Curse of Agony pure-DoT mapping is missing");
requireText(world, /startOfflineCurseOfAgonyCast[\s\S]*?startOfflinePureDotCast/, "Curse of Agony must reuse instant pure-DoT admission");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?curseOfAgonyAbilityCode\(\)[\s\S]*?landOfflineCurseOfAgonyProjectile/, "Curse of Agony projectile landing is missing");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?pureDotAbilityIndex\(abilityCode\)/, "pure DoT periodic threat must use the shared ability index");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?curseOfAgonyAbilityCode\(\)[\s\S]*?startOfflineCurseOfAgonyCast[\s\S]*?applySupportedCastCommand[\s\S]*?curseOfAgonyPayloadAbilityIsExact/, "Curse of Agony command routes are missing");
requireText(world, /pub curseOfAgonyCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepOfflineEastbrookDots/, "Curse of Agony state regression coverage is missing");

process.stdout.write(`WOS105 Curse of Agony static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
