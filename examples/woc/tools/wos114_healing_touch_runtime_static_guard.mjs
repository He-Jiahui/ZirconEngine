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
const start = classes.indexOf("  healing_touch: {");
const end = classes.indexOf("  mark_of_the_wild: {", start);
const healingTouch = classes.slice(start, end);
for (const needle of [
  "class: 'druid'", "learnLevel: 1", "cost: 25", "castTime: 2.5",
  "cooldown: 0", "range: 30", "school: 'nature'", "requiresTarget: true",
  "targetType: 'friendly'", "type: 'heal', min: 37, max: 51", "rank: 2",
  "level: 8", "cost: 45", "castTime: 3.0", "min: 68, max: 86", "rank: 3",
  "level: 14", "cost: 75", "min: 115, max: 140", "rank: 4", "level: 20",
  "cost: 110", "min: 175, max: 208",
]) {
  if (!healingTouch.includes(needle)) throw new Error(`source Healing Touch drifted: ${needle}`);
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

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/wrath',[\s\S]*?'healing_touch'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Healing Touch projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "healing_touch",
);
if (!entry || entry.index !== 54 || entry.definition.class !== "druid" ||
    entry.definition.school !== "nature" || entry.definition.cost !== 25 ||
    entry.definition.castTime !== 2.5 || entry.definition.cooldown !== 0 ||
    entry.definition.range !== 30 || !entry.definition.requiresTarget ||
    entry.definition.targetType !== "friendly" || entry.definition.effects?.[0]?.type !== "heal" ||
    entry.definition.effects[0].min !== 37 || entry.definition.effects[0].max !== 51 ||
    entry.definition.ranks?.[1]?.effects?.[0]?.min !== 115 ||
    entry.definition.ranks?.[2]?.effects?.[0]?.max !== 208) {
  throw new Error("M4 Healing Touch projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /healingTouchAbilityCode\([\s\S]*?healingTouchPayloadAbilityIsExact[\s\S]*?healingTouchTargetIndex/,
  "Healing Touch identity and target profile are missing");
requireText(world, /startOfflineHealingTouchCast[\s\S]*?healingTouchResolvedTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?healingTouchGlobalCooldownSeconds/,
  "Healing Touch cast admission is missing");
requireText(world, /completeOfflineHealingTouchCast[\s\S]*?entityResources\[casterIndex\][\s\S]*?applyOfflineDirectHeal/,
  "Healing Touch completion must charge then invoke the direct-heal resolver");
requireText(world, /applyOfflineDirectHeal[\s\S]*?numericEffects\.dispatchNumericAbility[\s\S]*?healState\.applyHeal[\s\S]*?setThreat/,
  "Healing Touch must reuse authoritative range, crit and healing-threat resolution");
requireText(world, /stepRetainedCasting[\s\S]*?healingTouchAbilityCode\(\)[\s\S]*?completeOfflineHealingTouchCast/,
  "Healing Touch completion must be registered in the retained cast step");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?healingTouchAbilityCode\(\)[\s\S]*?startOfflineHealingTouchCast[\s\S]*?applySupportedCastCommand[\s\S]*?healingTouchPayloadAbilityIsExact/,
  "Healing Touch command routes are missing");
requireText(world, /pub healingTouchCommandStateTest\(\): int[\s\S]*?m4AbilityCatalog\.indexOf\("healing_touch"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Healing Touch state regression coverage is missing");
requireText(world, /if \(healingTouchCommandStateTest\(\) != 1\) \{[\s\S]*?return -108;/,
  "world selfTest must execute Healing Touch");

process.stdout.write(`WOS114 Healing Touch static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
