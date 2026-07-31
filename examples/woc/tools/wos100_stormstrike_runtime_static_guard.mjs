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
const start = classes.indexOf("  stormstrike: {");
const end = classes.indexOf("  // ====================== WARLOCK", start);
const stormstrike = classes.slice(start, end);
for (const needle of [
  "class: 'shaman'",
  "learnLevel: 20",
  "cost: 40",
  "castTime: 0",
  "cooldown: 12",
  "range: 0",
  "school: 'physical'",
  "requiresTarget: true",
  "type: 'weaponStrike', bonus: 26",
]) {
  if (!stormstrike.includes(needle)) throw new Error(`source Stormstrike drifted: ${needle}`);
}
requireText(
  dispatch,
  /case 'weaponStrike':[\s\S]*?ctx\.meleeSwing\(p, target, bonus[\s\S]*?weaponMult[\s\S]*?threatFlat[\s\S]*?threatMult/,
  "source weapon-strike reducer drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/ghost_wolf',[\s\S]*?'stormstrike'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Stormstrike scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "stormstrike");
if (!entry || entry.index !== 40 || entry.definition.school !== "physical" ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "weaponStrike" ||
    entry.definition.effects[0].bonus !== 26) {
  throw new Error("M4 Stormstrike projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /stormstrikeAbilityCode\([\s\S]*?stormstrikePayloadAbilityIsExact/, "Stormstrike ability identity is missing");
requireText(world, /stormstrikeTargetIndex[\s\S]*?range <= 0\.0[\s\S]*?range = 5\.0/, "Stormstrike melee target validation is missing");
requireText(world, /startOfflineStormstrikeCast[\s\S]*?stormstrikeGlobalCooldownSeconds[\s\S]*?entityResources[\s\S]*?autoAttackState\.meleeSwing[\s\S]*?commitOfflineAutoAttackRng/, "Stormstrike deterministic weapon reducer is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?stormstrikeAbilityCode\(\)[\s\S]*?startOfflineStormstrikeCast[\s\S]*?applySupportedCastCommand[\s\S]*?stormstrikePayloadAbilityIsExact/, "Stormstrike command routes are missing");
requireText(world, /pub stormstrikeCommandStateTest\(\): int[\s\S]*?stormstrike[\s\S]*?abilityCooldownExpiresAt[\s\S]*?appendTypedCastTargetCommandForTest/, "Stormstrike state regression coverage is missing");

process.stdout.write(`WOS100 Stormstrike static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
