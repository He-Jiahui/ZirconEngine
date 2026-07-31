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
const start = classes.indexOf("  bash: {");
const end = classes.indexOf("  faerie_fire: {", start);
if (start < 0 || end < start) throw new Error("source Bash block is missing");
const bash = classes.slice(start, end);
for (const needle of [
  "name: 'Concuss'", "class: 'druid'", "learnLevel: 8", "cost: 10",
  "castTime: 0", "cooldown: 60", "range: 8", "school: 'physical'",
  "requiresTarget: true", "requiresForm: 'bear'",
  "type: 'stun', duration: 2",
]) {
  if (!bash.includes(needle)) throw new Error(`source Bash drifted: ${needle}`);
}
requireText(dispatch, /case 'stun':[\s\S]*?kind: 'stun',[\s\S]*?ctx\.enterCombat\(p, target\)/,
  "source Bash stun application drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/enrage',[\s\S]*?'bash'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Bash projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "bash",
);
if (!entry || entry.index !== 69 || entry.definition.cost !== 10 ||
    entry.definition.cooldown !== 60 || entry.definition.range !== 8 ||
    entry.definition.requiresForm !== "bear" || entry.definition.effects?.[0]?.type !== "stun" ||
    entry.definition.effects[0].duration !== 2) {
  throw new Error("M4 Bash projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /bashAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("bash"\)/,
  "Bash catalog identity is missing");
requireText(world, /bashTargetIndex[\s\S]*?m4AbilityCatalog\.metric[\s\S]*?range[\s\S]*?targetIndex : -1/,
  "Bash source range gate is missing");
requireText(world, /startOfflineBashCast[\s\S]*?entityCastingAbility[\s\S]*?entityCastGcdRemaining[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?applyOfflineBashStun[\s\S]*?setAbilityCooldownExpiration/,
  "Bash Bear-only stun reducer is missing");
requireText(world, /applyOfflineBashStun[\s\S]*?bashAbilityCode\(\)[\s\S]*?motionAuraKindCode\("stun"\)/,
  "Bash stun aura projection is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?bashAbilityCode\(\)[\s\S]*?startOfflineBashCast/,
  "Bash action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?bashPayloadAbilityIsExact[\s\S]*?startOfflineBashCast/,
  "Bash typed routing is missing");
requireText(world, /pub bashCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?abilityCooldownExpiresAt/,
  "Bash state regression coverage is missing");
requireText(world, /if \(bashCommandStateTest\(\) != 1\) \{[\s\S]*?return -123;/,
  "world selfTest must execute Bash");

process.stdout.write(`WOS129 Bash static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
