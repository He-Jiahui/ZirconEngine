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
const sim = source("src/sim/sim.ts");

for (const needle of [
  "id: 'sinister_strike'",
  "learnLevel: 1",
  "cost: 45",
  "castTime: 0",
  "school: 'physical'",
  "requiresTarget: true",
  "awardsCombo: 1",
  "effects: [{ type: 'weaponStrike', bonus: 3 }]",
]) {
  if (!classes.includes(needle)) {
    throw new Error(`source Sinister Strike definition drifted: ${needle}`);
  }
}
for (const needle of [
  "const maxRange = ability.range > 0 ? ability.range : MELEE_RANGE;",
  "if (facingDiff > MELEE_ARC)",
  "if (!ability.offGcd) p.gcdRemaining = Math.max(p.gcdRemaining, gcd);",
]) {
  if (!casting.includes(needle)) {
    throw new Error(`source Sinister Strike cast gate drifted: ${needle}`);
  }
}
for (const needle of [
  "const hit = ctx.meleeSwing(p, target, bonus, ability.name",
  "if (hit && ability.awardsCombo)",
]) {
  if (!dispatch.includes(needle)) {
    throw new Error(`source Sinister Strike effect reducer drifted: ${needle}`);
  }
}
for (const needle of [
  "const COMBO_POINT_DURATION = 30;",
  "p.comboPoints = Math.min(5, p.comboPoints + points);",
  "p.comboUntil = this.time + COMBO_POINT_DURATION;",
]) {
  if (!sim.includes(needle)) {
    throw new Error(`source combo-point reducer drifted: ${needle}`);
  }
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /sinisterStrikeTargetIndex[\s\S]*?m4AbilityCatalog\.metric\([\s\S]*?"range"[\s\S]*?absoluteAngleDifference[\s\S]*?startOfflineSinisterStrikeCast[\s\S]*?knownAbilityPartitionContains[\s\S]*?catalogAdmission/,
  "WOS57 Sinister Strike admission, range, and facing route are missing",
);
requireText(
  world,
  /startOfflineSinisterStrikeCast[\s\S]*?entityCastGcdRemaining[\s\S]*?m4AbilityCatalog\.metric[\s\S]*?autoAttackState\.meleeSwing[\s\S]*?commitOfflineAutoAttackRng[\s\S]*?enterIdleMobAggro[\s\S]*?state\.setThreat[\s\S]*?entityComboPoints[\s\S]*?entityComboUntil/,
  "WOS57 Sinister Strike cost, GCD, weapon-hit, aggro/threat, and combo reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?sinisterStrikeAbilityCode[\s\S]*?startOfflineSinisterStrikeCast[\s\S]*?applySupportedCastCommand[\s\S]*?sinisterStrikePayloadAbilityIsExact[\s\S]*?startOfflineSinisterStrikeCast/,
  "WOS57 Sinister Strike slot and typed command routes are missing",
);
requireText(
  world,
  /stepRetainedPlayerTicks[\s\S]*?expireOfflineComboPoints[\s\S]*?entityComboUntil[\s\S]*?entityComboPoints/,
  "WOS57 combo-point expiry is not advanced by the retained player tick",
);
requireText(
  world,
  /pub sinisterStrikeCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?entityComboUntil[\s\S]*?decodeState\(encodeState\(state\)\)/,
  "WOS57 Sinister Strike command, combo, and persistence coverage is missing",
);

process.stdout.write(`WOS57 Sinister Strike static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
