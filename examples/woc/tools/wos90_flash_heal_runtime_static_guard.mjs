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
const abilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const flashHealSource = abilityBlock("flash_heal", "lightning_bolt");
for (const needle of [
  "class: 'priest'", "learnLevel: 20", "cost: 75", "castTime: 1.5", "cooldown: 0",
  "range: 30", "school: 'holy'", "targetType: 'friendly'",
  "type: 'heal', min: 120, max: 142",
]) {
  if (!flashHealSource.includes(needle)) throw new Error(`source Flash Heal drifted: ${needle}`);
}
requireText(
  casting,
  /ability\.requiresTarget && ability\.targetType === 'friendly'[\s\S]*?resolveFriendlyTarget[\s\S]*?Math\.max\(ability\.range, 5\) \+ 2[\s\S]*?helpful spells never miss[\s\S]*?spendAbilityCost[\s\S]*?ctx\.runEffects/,
  "source friendly cast ordering drifted",
);
requireText(
  dispatch,
  /case 'heal':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\) \+ directHealBonus\(p\.spellPower, res\.castTime\)[\s\S]*?ctx\.applyHeal\(/,
  "source direct-heal range, scaling or crit dispatch drifted",
);

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'heal',[\s\S]*?'flash_heal'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Flash Heal projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const flashHeal = m4.entries.find((entry) => entry.id === "flash_heal");
if (!flashHeal || flashHeal.index !== 30 || flashHeal.scenarios.length !== 0 ||
    flashHeal.definition.school !== "holy" || flashHeal.definition.targetType !== "friendly" ||
    flashHeal.definition.castTime !== 1.5 || flashHeal.definition.effects?.[0]?.type !== "heal" ||
    flashHeal.definition.effects?.[0]?.min !== 120 || flashHeal.definition.effects?.[0]?.max !== 142) {
  throw new Error("M4 Flash Heal source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /flashHealAbilityCode\([\s\S]*?startOfflineFlashHealCast[\s\S]*?completeOfflineFlashHealCast/, "Flash Heal cast reducer is missing");
requireText(world, /startOfflineFlashHealCast[\s\S]*?flashHealResolvedTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?entityCastTargetIds/, "Flash Heal must lock a source-valid friendly target and arm its cast");
requireText(world, /completeOfflineFlashHealCast[\s\S]*?entityResources\[casterIndex\][\s\S]*?applyOfflineDirectHeal/, "Flash Heal completion must charge then invoke the direct-heal resolver");
requireText(world, /applyOfflineDirectHeal[\s\S]*?numericEffects\.dispatchNumericAbility[\s\S]*?healState\.applyHeal[\s\S]*?setThreat/, "Flash Heal resolver must use the shared deterministic direct-heal path");
requireText(world, /stepRetainedCasting[\s\S]*?completedAbility == flashHealAbilityCode\(\)[\s\S]*?completeOfflineFlashHealCast/, "Flash Heal completion must be registered in the retained cast step");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?flashHealAbilityCode\(\)[\s\S]*?startOfflineFlashHealCast[\s\S]*?applySupportedCastCommand[\s\S]*?flashHealPayloadAbilityIsExact/, "Flash Heal slot and typed routes are missing");
requireText(world, /pub flashHealCommandStateTest\(\): int[\s\S]*?flash_heal[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepRetainedCasting[\s\S]*?rngDraws/, "Flash Heal state regression coverage is missing");

process.stdout.write(`WOS90 Flash Heal static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
