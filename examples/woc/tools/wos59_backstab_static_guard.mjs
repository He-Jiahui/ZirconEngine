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
  "id: 'backstab'",
  "learnLevel: 4",
  "cost: 60",
  "castTime: 0",
  "school: 'physical'",
  "requiresTarget: true",
  "awardsCombo: 1",
  "effects: [{ type: 'weaponStrike', bonus: 11, requiresBehind: true, weaponMult: 1.5 }]",
]) {
  if (!classes.includes(needle)) {
    throw new Error(`source Backstab definition drifted: ${needle}`);
  }
}
for (const needle of [
  "const maxRange = ability.range > 0 ? ability.range : MELEE_RANGE;",
  "if (facingDiff > MELEE_ARC)",
  "if (!p.weapon.dagger)",
  "const behindDiff = Math.abs(normAngle(angleTo(target.pos, p.pos) - target.facing));",
  "behindDiff < Math.PI / 2",
  "dist2d(target.pos, p.pos) < FACING_HOLD_DIST",
  "if (!ability.offGcd) p.gcdRemaining = Math.max(p.gcdRemaining, gcd);",
]) {
  if (!casting.includes(needle)) {
    throw new Error(`source Backstab cast gate drifted: ${needle}`);
  }
}
for (const needle of [
  "const hit = ctx.meleeSwing(p, target, bonus, ability.name",
  "weaponMult",
  "if (hit && ability.awardsCombo)",
]) {
  if (!dispatch.includes(needle)) {
    throw new Error(`source Backstab weapon-strike reducer drifted: ${needle}`);
  }
}

const m5Generator = read("tools", "m5_content_zr_codegen.mjs");
const m5Equipment = read("scripts", "woc_game", "src", "progression", "m5_equipment_state.zr");
if (!m5Generator.includes("weaponDagger: definition.weapon?.dagger === true")) {
  throw new Error("M5 weapon dagger projection is missing");
}
if (!m5Equipment.includes("pub mainhandIsDagger")) {
  throw new Error("M5 mainhand dagger query is missing");
}

const m4SourceGenerator = read("tools", "m4_ability_codegen.mjs");
const m4ZrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
const m4Contract = JSON.parse(read("contracts", "m4_abilities.json"));
const m4Effects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
const backstabEntry = m4Contract.entries.find((entry) => entry.id === "backstab");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'backstab'/.test(m4SourceGenerator)) {
  throw new Error("M4 WOC-only Backstab projection scope is missing");
}
if (!m4ZrGenerator.includes("'weaponMult'") ||
    !m4ZrGenerator.includes("'requiresBehind'")) {
  throw new Error("M4 Backstab effect metric/flag projection is missing");
}
if (!backstabEntry || backstabEntry.index !== 21 || backstabEntry.scenarios.length !== 0 ||
    backstabEntry.definition.effects[0].weaponMult !== 1.5 ||
    backstabEntry.definition.effects[0].requiresBehind !== true ||
    backstabEntry.definition.ranks.length !== 2) {
  throw new Error("M4 Backstab retained source projection drifted");
}
requireText(
  m4Effects,
  /if \(index == 21\) \{[\s\S]*?if \(field == "weaponMult"\) \{ return 1\.5; \}[\s\S]*?pub flag[\s\S]*?if \(index == 21\) \{[\s\S]*?if \(field == "requiresBehind"\) \{ return true; \}/,
  "generated M4 Backstab multiplier or behind flag is missing",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /offlineMainhandIsDagger[\s\S]*?m5Equipment\.mainhandIsDagger[\s\S]*?backstabTargetIndex[\s\S]*?m4AbilityCatalog\.metric\([\s\S]*?"range"[\s\S]*?entityFacing\[targetIndex\][\s\S]*?m4AbilityEffects\.flag[\s\S]*?"requiresBehind"[\s\S]*?startOfflineBackstabCast/,
  "WOS59 Backstab dagger, range, caster-facing, and behind-target admission is missing",
);
requireText(
  world,
  /startOfflineBackstabCast[\s\S]*?entityCastGcdRemaining[\s\S]*?offlineMainhandIsDagger[\s\S]*?autoAttackState\.meleeSwing[\s\S]*?weaponMultiplier[\s\S]*?commitOfflineAutoAttackRng[\s\S]*?enterIdleMobAggro[\s\S]*?state\.setThreat[\s\S]*?entityComboPoints[\s\S]*?entityComboUntil/,
  "WOS59 Backstab cost, dagger, shared weapon-hit, aggro/threat, and combo reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?backstabAbilityCode[\s\S]*?startOfflineBackstabCast[\s\S]*?applySupportedCastCommand[\s\S]*?backstabPayloadAbilityIsExact[\s\S]*?startOfflineBackstabCast/,
  "WOS59 Backstab slot and typed command routes are missing",
);
requireText(
  world,
  /pub backstabCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?entityM5MainhandItemCodes[\s\S]*?decodeState\(encodeState\(state\)\)/,
  "WOS59 Backstab dagger, behind, command, and persistence coverage is missing",
);

process.stdout.write(`WOS59 Backstab static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
