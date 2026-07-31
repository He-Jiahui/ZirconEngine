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
const lifecycle = source("src/sim/combat/casting_lifecycle.ts");
const entity = source("src/sim/entity.ts");
const start = classes.indexOf("  primal_reflexes: {");
const end = classes.indexOf("  starfire: {", start);
if (start < 0 || end < start) throw new Error("source Primal Reflexes block is missing");
const primalReflexes = classes.slice(start, end);
for (const needle of [
  "name: 'Primal Reflexes'", "class: 'druid'", "learnLevel: 20", "cost: 0",
  "castTime: 0", "cooldown: 60", "range: 0", "school: 'nature'",
  "requiresTarget: false", "offGcd: true", "usableInForm: true",
  "type: 'selfBuff', kind: 'buff_dodge', value: 0.5, duration: 6",
]) {
  if (!primalReflexes.includes(needle)) {
    throw new Error(`source Primal Reflexes drifted: ${needle}`);
  }
}
requireText(lifecycle, /!ability\.offGcd && p\.gcdRemaining > 0/,
  "source Primal Reflexes off-GCD gate drifted");
requireText(lifecycle, /form && !isFormToggle\(ability\) && !ability\.usableInForm/,
  "source Primal Reflexes form-admission gate drifted");
requireText(entity, /a\.kind === 'buff_dodge'\) bonusDodge \+= a\.value/,
  "source buff_dodge aggregation drifted");
requireText(entity, /e\.dodgeChance = Math\.max\(0, 0\.05 \+ s\.agi \* 0\.0005 \+ bonusDodge\)/,
  "source dodge floor drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/barkskin',[\s\S]*?'primal_reflexes'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Primal Reflexes projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "primal_reflexes",
);
if (!entry || entry.index !== 67 || entry.definition.cost !== 0 ||
    entry.definition.cooldown !== 60 || !entry.definition.offGcd ||
    !entry.definition.usableInForm || entry.definition.effects?.[0]?.type !== "selfBuff" ||
    entry.definition.effects[0].kind !== "buff_dodge" ||
    entry.definition.effects[0].value !== 0.5 || entry.definition.effects[0].duration !== 6) {
  throw new Error("M4 Primal Reflexes projection drifted");
}

const ccGenerator = read("tools", "cc_contract_codegen.mjs");
const cc = JSON.parse(read("reference", "current-head", "cc_contract.json"));
if (!ccGenerator.includes("buff_dodge: 10") ||
    cc.motion_kind_codes?.buff_dodge !== 10) {
  throw new Error("buff_dodge motion-aura contract projection is missing");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /primalReflexesAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("primal_reflexes"\)/,
  "Primal Reflexes catalog identity is missing");
requireText(world, /startOfflinePrimalReflexesCast[\s\S]*?entityCastingAbility[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?m4AbilityCatalog\.flag\(abilityIndex, "offGcd"\)[\s\S]*?setAbilityCooldownExpiration[\s\S]*?applyOfflineMotionAuraWithDetails[\s\S]*?motionAuraKindCode\("buff_dodge"\)/,
  "Primal Reflexes off-GCD dodge-aura reducer is missing");
const reducerStart = world.indexOf("startOfflinePrimalReflexesCast");
const reducerEnd = world.indexOf("\n}\n", reducerStart);
if (reducerStart < 0 || reducerEnd < reducerStart ||
    world.slice(reducerStart, reducerEnd).includes("entityCastGcdRemaining")) {
  throw new Error("Primal Reflexes must not consume or require the global cooldown");
}
requireText(world, /primalReflexesDodgeBonus[\s\S]*?primalReflexesAbilityCode[\s\S]*?value != 0\.5/,
  "Primal Reflexes dodge profile is missing");
requireText(world, /effectiveOfflineDodgeChance[\s\S]*?entityDodgeChance[\s\S]*?primalReflexesDodgeBonus[\s\S]*?return dodge > 0\.0 \? dodge : 0\.0/,
  "Primal Reflexes must aggregate on top of the stored base dodge chance");
requireText(world, /resolveOfflineEastbrookMobSwingRequests[\s\S]*?effectiveOfflineDodgeChance\(state, playerIndex\)/,
  "offline mob hit table does not use the effective dodge chance");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?primalReflexesAbilityCode\(\)[\s\S]*?startOfflinePrimalReflexesCast/,
  "Primal Reflexes action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?primalReflexesPayloadAbilityIsExact[\s\S]*?startOfflinePrimalReflexesCast/,
  "Primal Reflexes typed routing is missing");
requireText(world, /pub primalReflexesCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?appendTypedCastCommandForTest[\s\S]*?abilityCooldownExpiresAt/,
  "Primal Reflexes state regression coverage is missing");
requireText(world, /if \(primalReflexesCommandStateTest\(\) != 1\) \{[\s\S]*?return -121;/,
  "world selfTest must execute Primal Reflexes");

process.stdout.write(`WOS127 Primal Reflexes static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
