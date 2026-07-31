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
const damage = source("src/sim/combat/damage.ts");

for (const needle of [
  "id: 'eviscerate'",
  "learnLevel: 1",
  "cost: 35",
  "castTime: 0",
  "school: 'physical'",
  "requiresTarget: true",
  "spendsCombo: true",
  "effects: [{ type: 'finisherDamage', base: 4, perCombo: 7, variance: 4 }]",
]) {
  if (!classes.includes(needle)) {
    throw new Error(`source Eviscerate definition drifted: ${needle}`);
  }
}
for (const needle of [
  "const maxRange = ability.range > 0 ? ability.range : MELEE_RANGE;",
  "if (facingDiff > MELEE_ARC)",
  "if (!ability.offGcd) p.gcdRemaining = Math.max(p.gcdRemaining, gcd);",
]) {
  if (!casting.includes(needle)) {
    throw new Error(`source Eviscerate cast gate drifted: ${needle}`);
  }
}
for (const needle of [
  "const spentCombo = ability.spendsCombo ? p.comboPoints : 0;",
  "case 'finisherDamage':",
  "ctx.rng.range(0, eff.variance)",
  "ctx.effectiveAttackPower(p) / 14",
  "armorReduction(ctx.effectiveArmor(target), p.level)",
  "ctx.dealDamage(",
  "if (ability.spendsCombo && spentCombo > 0)",
  "p.comboPoints = 0;",
]) {
  if (!dispatch.includes(needle)) {
    throw new Error(`source Eviscerate effect reducer drifted: ${needle}`);
  }
}
for (const needle of [
  "if (source && source.id !== target.id) ctx.enterCombat(source, target);",
  "const threat =",
  "addThreat(target, source.id, threat);",
]) {
  if (!damage.includes(needle)) {
    throw new Error(`source direct-damage combat/threat reducer drifted: ${needle}`);
  }
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /eviscerateTargetIndex[\s\S]*?m4AbilityCatalog\.metric\([\s\S]*?"range"[\s\S]*?absoluteAngleDifference[\s\S]*?startOfflineEviscerateCast[\s\S]*?catalogAdmission[\s\S]*?entityComboPoints/,
  "WOS58 Eviscerate admission, combo, range, and facing route are missing",
);
requireText(
  world,
  /startOfflineEviscerateCast[\s\S]*?entityCastGcdRemaining[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?mobSwing\.armorReductionFromArmor[\s\S]*?entityHp[\s\S]*?enterIdleMobAggro[\s\S]*?state\.setThreat[\s\S]*?settleOfflineEastbrookLethal[\s\S]*?entityComboPoints/,
  "WOS58 Eviscerate cost, RNG, damage, aggro/threat, combo, and death reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?eviscerateAbilityCode[\s\S]*?startOfflineEviscerateCast[\s\S]*?applySupportedCastCommand[\s\S]*?evisceratePayloadAbilityIsExact[\s\S]*?startOfflineEviscerateCast/,
  "WOS58 Eviscerate slot and typed command routes are missing",
);
requireText(
  world,
  /pub eviscerateCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?entityComboPoints[\s\S]*?decodeState\(encodeState\(state\)\)/,
  "WOS58 Eviscerate command, combo-spend, and persistence coverage is missing",
);

process.stdout.write(`WOS58 Eviscerate static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
