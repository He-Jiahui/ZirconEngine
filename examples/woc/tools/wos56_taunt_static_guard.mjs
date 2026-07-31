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
const effects = source("src/sim/combat/effect_dispatch.ts");
const sim = source("src/sim/sim.ts");
const targeting = source("src/sim/mob/targeting.ts");

for (const needle of [
  "id: 'taunt'",
  "learnLevel: 5",
  "cooldown: 10",
  "range: 8",
  "offGcd: true",
  "effects: [{ type: 'taunt' }]",
]) {
  if (!classes.includes(needle)) {
    throw new Error(`source taunt definition drifted: ${needle}`);
  }
}
if (!effects.includes("ctx.applyTaunt(p, target);")) {
  throw new Error("source taunt effect dispatch drifted");
}
for (const needle of [
  "mob.threat.set(p.id, Math.max(mine, top, 1));",
  "mob.forcedTargetTimer = TAUNT_FORCE_SECONDS;",
  "this.aggroMob(mob, p, false);",
  "mob.aiState = 'attack';",
  "mob.fleeTimer = 0;",
]) {
  if (!sim.includes(needle)) {
    throw new Error(`source taunt reducer drifted: ${needle}`);
  }
}
for (const needle of [
  "export function tickForcedTarget",
  "mob.forcedTargetTimer -= DT",
  "mob.forcedTargetId = null",
]) {
  if (!targeting.includes(needle)) {
    throw new Error(`source forced-target timer drifted: ${needle}`);
  }
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /tauntTargetIndex[\s\S]*?m4AbilityCatalog\.metric\([\s\S]*?"range"[\s\S]*?absoluteAngleDifference[\s\S]*?startOfflineTauntCast[\s\S]*?knownAbilityPartitionContains[\s\S]*?catalogAdmission[\s\S]*?setAbilityCooldownExpiration/,
  "WOS56 Taunt admission, range, facing, and cooldown route are missing",
);
requireText(
  world,
  /startOfflineTauntCast[\s\S]*?stateTopThreatValue[\s\S]*?entityForcedTargetIds[\s\S]*?entityForcedTargetTimers[\s\S]*?enterIdleMobAggroWithoutSocial[\s\S]*?entityFleeTimers/,
  "WOS56 Taunt threat, idle, and flee reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?tauntAbilityCode[\s\S]*?startOfflineTauntCast[\s\S]*?applySupportedCastCommand[\s\S]*?tauntPayloadAbilityIsExact[\s\S]*?startOfflineTauntCast/,
  "WOS56 Taunt slot and typed command routes are missing",
);
requireText(
  world,
  /stepOfflineEastbrookMobMeleePursuit[\s\S]*?stepOfflineEastbrookForcedTarget[\s\S]*?entityForcedTargetTimers[\s\S]*?mobLifecycle\.fixedTickMicros/,
  "WOS56 forced-target timer is not advanced by the Eastbrook mob tick",
);
requireText(
  world,
  /pub tauntCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?entityForcedTargetTimers[\s\S]*?entityFleeTimers/,
  "WOS56 Taunt command, timer, and flee regression coverage is missing",
);

process.stdout.write(`WOS56 Taunt static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
