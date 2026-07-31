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
const healingWaveSource = abilityBlock("healing_wave", "earth_shock");
for (const needle of [
  "class: 'shaman'", "learnLevel: 1", "cost: 25", "castTime: 1.5", "cooldown: 0",
  "range: 30", "school: 'nature'", "targetType: 'friendly'",
  "type: 'heal', min: 36, max: 44",
  "rank: 2", "level: 6", "cost: 40", "castTime: 2.0", "min: 56, max: 68",
  "rank: 3", "level: 12", "cost: 65", "castTime: 2.5", "min: 92, max: 110",
  "rank: 4", "level: 18", "cost: 90", "min: 138, max: 164",
]) {
  if (!healingWaveSource.includes(needle)) {
    throw new Error(`source Healing Wave drifted: ${needle}`);
  }
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
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'lightning_bolt',[\s\S]*?'healing_wave'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Healing Wave projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const healingWave = m4.entries.find((entry) => entry.id === "healing_wave");
if (!healingWave || healingWave.index !== 33 || healingWave.scenarios.length !== 0 ||
    healingWave.definition.school !== "nature" || healingWave.definition.targetType !== "friendly" ||
    healingWave.definition.castTime !== 1.5 || healingWave.definition.effects?.[0]?.type !== "heal" ||
    healingWave.definition.effects?.[0]?.min !== 36 || healingWave.definition.effects?.[0]?.max !== 44 ||
    healingWave.definition.ranks?.length !== 3 ||
    healingWave.definition.ranks?.[2]?.effects?.[0]?.max !== 164) {
  throw new Error("M4 Healing Wave source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /healingWaveAbilityCode\([\s\S]*?startOfflineHealingWaveCast[\s\S]*?completeOfflineHealingWaveCast/, "Healing Wave cast reducer is missing");
requireText(world, /startOfflineHealingWaveCast[\s\S]*?healingWaveResolvedTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?entityCastTargetIds/, "Healing Wave must lock a source-valid friendly target and arm its cast");
requireText(world, /completeOfflineHealingWaveCast[\s\S]*?entityResources\[casterIndex\][\s\S]*?applyOfflineDirectHeal/, "Healing Wave completion must charge then invoke the direct-heal resolver");
requireText(world, /applyOfflineDirectHeal[\s\S]*?numericEffects\.dispatchNumericAbility[\s\S]*?healState\.applyHeal[\s\S]*?setThreat/, "Healing Wave resolver must use the shared deterministic direct-heal path");
requireText(world, /stepRetainedCasting[\s\S]*?completedAbility == healingWaveAbilityCode\(\)[\s\S]*?completeOfflineHealingWaveCast/, "Healing Wave completion must be registered in the retained cast step");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?healingWaveAbilityCode\(\)[\s\S]*?startOfflineHealingWaveCast[\s\S]*?applySupportedCastCommand[\s\S]*?healingWavePayloadAbilityIsExact/, "Healing Wave slot and typed routes are missing");
requireText(world, /pub healingWaveCommandStateTest\(\): int[\s\S]*?healing_wave[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepRetainedCasting[\s\S]*?rngDraws/, "Healing Wave state regression coverage is missing");

process.stdout.write(`WOS93 Healing Wave static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
