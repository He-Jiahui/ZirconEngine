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
const effects = source("src/sim/combat/effect_dispatch.ts");
requireText(
  classes,
  /lesser_heal:\s*\{[\s\S]*?learnLevel: 1,[\s\S]*?cost: 30,[\s\S]*?castTime: 2\.0,[\s\S]*?range: 30,[\s\S]*?school: 'holy',[\s\S]*?targetType: 'friendly',[\s\S]*?type: 'heal', min: 47, max: 58[\s\S]*?rank: 2, level: 6, cost: 45,[\s\S]*?min: 72, max: 86[\s\S]*?rank: 3, level: 12, cost: 65,[\s\S]*?min: 110, max: 132/,
  "source Lesser Heal definition drifted",
);
requireText(
  casting,
  /ability\.requiresTarget && ability\.targetType === 'friendly'[\s\S]*?resolveFriendlyTarget[\s\S]*?Math\.max\(ability\.range, 5\) \+ 2[\s\S]*?helpful spells never miss[\s\S]*?spendAbilityCost[\s\S]*?ctx\.runEffects/,
  "source friendly target resolution or completion ordering drifted",
);
requireText(
  effects,
  /case 'heal':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\) \+ directHealBonus\(p\.spellPower, res\.castTime\)[\s\S]*?ctx\.applyHeal\(/,
  "source Lesser Heal range, scaling or healing dispatch drifted",
);

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const lesserHeal = m4.entries.find((entry) => entry.id === "lesser_heal");
if (!lesserHeal || lesserHeal.index !== 6 || lesserHeal.definition.class !== "priest" ||
    lesserHeal.definition.learnLevel !== 1 || lesserHeal.definition.cost !== 30 ||
    lesserHeal.definition.castTime !== 2 || lesserHeal.definition.range !== 30 ||
    lesserHeal.definition.school !== "holy" || lesserHeal.definition.targetType !== "friendly" ||
    lesserHeal.definition.effects?.[0]?.type !== "heal" ||
    lesserHeal.definition.effects?.[0]?.min !== 47 || lesserHeal.definition.effects?.[0]?.max !== 58) {
  throw new Error("M4 Lesser Heal projection drifted");
}

const healState = read("scripts", "woc_game", "src", "combat", "heal_state.zr");
requireText(
  healState,
  /pub applyHeal\([\s\S]*?amount: float,[\s\S]*?fractionalSource[\s\S]*?52\.5[\s\S]*?\)\s*!= 79/,
  "heal kernel must round the fractional Lesser Heal amount after its crit multiplier",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /var healState = %import\("combat\/heal_state"\);/, "WorldState must use the healing state kernel");
requireText(world, /var numericEffects = %import\("combat\/effect_numeric_dispatch_state"\);/, "WorldState must use the numeric effect dispatcher");
requireText(world, /lesserHealAbilityCode\([\s\S]*?abilityCode\("lesser_heal"\)[\s\S]*?m4AbilityCatalog\.indexOf\("lesser_heal"\)/, "Lesser Heal identity is missing");
requireText(world, /lesserHealTargetIndex[\s\S]*?Math\.max\(range, 5\.0\) \+ 2\.0[\s\S]*?lesserHealCastSeconds[\s\S]*?startOfflineLesserHealCast[\s\S]*?cast\.armTimed[\s\S]*?entityCastTargetIds/, "Lesser Heal must lock a source-valid friendly target and arm its cast");
requireText(world, /completeOfflineLesserHealCast[\s\S]*?lesserHealTargetIndex[\s\S]*?entityResources\[casterIndex\][\s\S]*?numericEffects\.dispatchNumericAbility[\s\S]*?healState\.applyHeal[\s\S]*?setThreat/, "Lesser Heal completion must charge, roll range then crit, commit healing and distribute threat");
requireText(world, /stepRetainedCasting[\s\S]*?completedAbility == lesserHealAbilityCode\(\)[\s\S]*?completeOfflineLesserHealCast/, "Lesser Heal must complete before target lock cleanup");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?lesserHealAbilityCode\(\)[\s\S]*?startOfflineLesserHealCast[\s\S]*?applySupportedCastCommand[\s\S]*?lesserHealPayloadAbilityIsExact/, "Lesser Heal slot and typed routes are missing");
requireText(world, /pub lesserHealCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepRetainedCasting[\s\S]*?rngDraws/, "Lesser Heal state regression coverage is missing");
requireText(world, /if \(lesserHealCommandStateTest\(\) != 1\) \{\s*return -62;\s*\}/, "Lesser Heal self-test route is missing");

process.stdout.write(`WOS66 Lesser Heal static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
