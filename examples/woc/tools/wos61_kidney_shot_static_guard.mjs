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

for (const needle of [
  "kidney_shot: {",
  "id: 'kidney_shot'",
  "learnLevel: 8",
  "cost: 25",
  "cooldown: 20",
  "school: 'physical'",
  "requiresTarget: true",
  "spendsCombo: true",
  "effects: [{ type: 'finisherStun', base: 1, perCombo: 1 }]",
]) {
  if (!classes.includes(needle)) {
    throw new Error(`source Kidney Shot definition drifted: ${needle}`);
  }
}
for (const needle of [
  "if (ability.spendsCombo && p.comboPoints <= 0)",
]) {
  if (!casting.includes(needle)) {
    throw new Error(`source Kidney Shot admission drifted: ${needle}`);
  }
}
for (const needle of [
  "case 'finisherStun': {",
  "stunDrCategory(ability.id)",
  "eff.base + eff.perCombo * spentCombo",
  "id: `${ability.id}_stun`",
  "kind: 'stun'",
  "if (ability.spendsCombo && spentCombo > 0)",
  "p.comboPoints = 0;",
]) {
  if (!dispatch.includes(needle)) {
    throw new Error(`source Kidney Shot reducer drifted: ${needle}`);
  }
}

const m4Contract = JSON.parse(read("contracts", "m4_abilities.json"));
const m4Effects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
const kidneyShotEntry = m4Contract.entries.find((entry) => entry.id === "kidney_shot");
if (!kidneyShotEntry || kidneyShotEntry.index !== 13 ||
    !kidneyShotEntry.scenarios.includes("c4b_effect_dispatch") ||
    kidneyShotEntry.definition.cost !== 25 || kidneyShotEntry.definition.cooldown !== 20 ||
    kidneyShotEntry.definition.spendsCombo !== true ||
    kidneyShotEntry.definition.effects[0].type !== "finisherStun" ||
    kidneyShotEntry.definition.effects[0].base !== 1 ||
    kidneyShotEntry.definition.effects[0].perCombo !== 1) {
  throw new Error("M4 Kidney Shot source projection drifted");
}
requireText(
  m4Effects,
  /if \(index == 13\) \{[\s\S]*?return "finisherStun";[\s\S]*?if \(field == "base"\) \{ return 1\.0; \}[\s\S]*?if \(field == "perCombo"\) \{ return 1\.0; \}/,
  "generated M4 Kidney Shot effect projection is missing",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /applyOfflineMotionAura[\s\S]*?applyOfflineKidneyShotStun[\s\S]*?kidneyShotAbilityCode[\s\S]*?motionAuraKindCode\("stun"\)/,
  "WOS61 Kidney Shot stun-aura projection is missing",
);
requireText(
  world,
  /kidneyShotAbilityCode[\s\S]*?kidneyShotTargetIndex[\s\S]*?m4AbilityCatalog\.metric\([\s\S]*?"range"[\s\S]*?startOfflineKidneyShotCast/,
  "WOS61 Kidney Shot catalog and target admission is missing",
);
requireText(
  world,
  /startOfflineKidneyShotCast[\s\S]*?abilityCooldownExpiresAt[\s\S]*?entityComboPoints[\s\S]*?m4AbilityEffects\.metric[\s\S]*?"base"[\s\S]*?"perCombo"[\s\S]*?enterIdleMobAggro[\s\S]*?applyOfflineKidneyShotStun[\s\S]*?clearOfflineComboPoints[\s\S]*?setAbilityCooldownExpiration/,
  "WOS61 Kidney Shot combo, stun, combat and cooldown reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?kidneyShotAbilityCode[\s\S]*?startOfflineKidneyShotCast[\s\S]*?applySupportedCastCommand[\s\S]*?kidneyShotPayloadAbilityIsExact[\s\S]*?startOfflineKidneyShotCast/,
  "WOS61 Kidney Shot slot and typed command routes are missing",
);
requireText(
  world,
  /pub kidneyShotCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?entityForcedTargetTimers[\s\S]*?stepOfflineEastbrookMobMeleePursuit[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?noCombo/,
  "WOS61 Kidney Shot command, persistence, forced-target and combo coverage is missing",
);

process.stdout.write(`WOS61 Kidney Shot static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
