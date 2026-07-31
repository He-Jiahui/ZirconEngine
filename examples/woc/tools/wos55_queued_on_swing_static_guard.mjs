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

const castingLifecycle = source("src/sim/combat/casting_lifecycle.ts");
const autoAttackSource = source("src/sim/combat/auto_attack.ts");
const damageSource = source("src/sim/combat/damage.ts");
const spiritSource = source("src/sim/spirit.ts");
for (const needle of [
  "if (ability.onNextSwing) {",
  "const toggledOff = p.queuedOnSwing === ability.id;",
  "p.queuedOnSwing = toggledOff ? null : ability.id;",
  "p.queuedOnSwingFree = true;",
  "p.queuedOnSwingCostMultiplier = cheap;",
  "if (!p.autoAttack && target) ctx.startAutoAttack(p.id);",
]) {
  if (!castingLifecycle.includes(needle)) {
    throw new Error(`source queued-on-swing arming behavior drifted: ${needle}`);
  }
}

for (const needle of [
  "Math.ceil(queued.cost * (p.queuedOnSwingCostMultiplier ?? 1))",
  "p.queuedOnSwing = null;",
  "delete p.queuedOnSwingFree;",
  "delete p.queuedOnSwingCostMultiplier;",
]) {
  if (!autoAttackSource.includes(needle)) {
    throw new Error(`source queued-on-swing consumption behavior drifted: ${needle}`);
  }
}

for (const needle of [
  "e.autoAttack = false;",
  "e.queuedOnSwing = null;",
  "delete e.queuedOnSwingFree;",
  "delete e.queuedOnSwingCostMultiplier;",
]) {
  if (!damageSource.includes(needle)) {
    throw new Error(`source death queued-on-swing cleanup drifted: ${needle}`);
  }
}

for (const needle of [
  "p.autoAttack = false;",
  "p.queuedOnSwing = null;",
]) {
  if (!spiritSource.includes(needle)) {
    throw new Error(`source spirit queued-on-swing cleanup drifted: ${needle}`);
  }
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const autoAttack = read("scripts", "woc_game", "src", "combat", "auto_attack_state.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "current WOS writer is missing");
requireText(
  world,
  /schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS55 decoder admission is missing",
);
requireText(
  world,
  /entityQueuedOnSwingAbilityCodes: container\.Array<uint>;[\s\S]*?entityQueuedOnSwingFree: container\.Array<bool>;[\s\S]*?entityQueuedOnSwingCostMultipliers: container\.Array<float>;/,
  "WOS55 queue columns are missing",
);
requireText(
  world,
  /while \(queuedOnSwingIndex < entityCount\)[\s\S]*?writer\.u16\(<uint>state\.entityQueuedOnSwingAbilityCodes\[queuedOnSwingIndex\], 1, 1\);[\s\S]*?writer\.byte\(<bool>state\.entityQueuedOnSwingFree\[queuedOnSwingIndex\][\s\S]*?writer\.fixed6\([\s\S]*?entityQueuedOnSwingCostMultipliers\[queuedOnSwingIndex\]/,
  "WOS55 canonical tail is missing",
);
requireText(
  world,
  /if \(schemaVersion >= <uint>55\) \{[\s\S]*?entityQueuedOnSwingAbilityCodes\.add\(reader\.u16\(1, 1\)\);[\s\S]*?appendDefaultQueuedOnSwingColumns\(state\);/,
  "WOS2-WOS54 queued-on-swing migration is missing",
);
requireText(
  world,
  /queuedOnSwingStateIsValid[\s\S]*?queuedOnSwingM4AbilityIndex[\s\S]*?m4AbilityCatalog\.flag\(index, "onNextSwing"\)/,
  "WOS55 must admit only M4 on-next-swing rows",
);
requireText(
  world,
  /prepareOfflineAutoActor[\s\S]*?actor\.hasQueuedAbility = queuedAbilityCode != <uint>0;[\s\S]*?actor\.queuedCostMultiplier =/,
  "world-to-auto queued-on-swing projection is missing",
);
requireText(
  world,
  /commitOfflineAutoActor[\s\S]*?entityQueuedOnSwingAbilityCodes\[playerIndex\][\s\S]*?actor\.resolvedQueuedAbility[\s\S]*?setAbilityCooldownExpiration\(/,
  "queued-on-swing cooldown writeback is missing",
);
requireText(
  world,
  /startQueuedOnSwingCast[\s\S]*?knownAbilityPartitionContains[\s\S]*?catalogAdmission[\s\S]*?dx \* dx \+ dz \* dz > 25\.0[\s\S]*?togglingOff[\s\S]*?startOfflineAutoAttack/,
  "queued-on-swing command admission is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?queuedOnSwingM4AbilityIndex\(abilityCode\) >= 0[\s\S]*?startQueuedOnSwingCast[\s\S]*?applySupportedCastCommand[\s\S]*?queuedOnSwingPayloadAbilityCode/,
  "slot and typed cast routes are missing",
);
requireText(
  autoAttack,
  /queuedCostMultiplier: float;[\s\S]*?resolvedQueuedAbility: string;[\s\S]*?ceilPositive\([\s\S]*?catalog\.metric\(abilityIndex, actor\.level, "cost"\) \*[\s\S]*?actor\.queuedCostMultiplier[\s\S]*?actor\.queuedCostMultiplier = 1\.0/,
  "mainhand billing multiplier or queue cleanup is missing",
);
requireText(
  autoAttack,
  /if \(actor\.dualWielding && actor\.hasOffhandWeapon && actor\.offhandSwingTimer <= 0\.0\) \{[\s\S]*?offhandMeleeSwing\(actor, target, events\);/,
  "offhand white swing must remain separate from queued consumption",
);
requireText(
  world,
  /pub queuedOnSwingCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastCommandForTest[\s\S]*?abilityCooldownExpiresAt\(restored, 0, raptor\) != <uint>6000000/,
  "WOS55 queue/toggle/cooldown regression coverage is missing",
);
requireText(
  world,
  /clearQueuedOnSwing\(state: WorldState, index: int\): void[\s\S]*?entityQueuedOnSwingAbilityCodes\[index\] = <uint>0;[\s\S]*?entityQueuedOnSwingFree\[index\] = false;[\s\S]*?entityQueuedOnSwingCostMultipliers\[index\] = 1\.0;/,
  "WOS55 terminal queue cleanup helper is missing",
);
requireText(
  world,
  /clearOfflineCombatPosture[\s\S]*?clearQueuedOnSwing\(state, playerIndex\);[\s\S]*?applyOfflineMobMeleePlayerDeath[\s\S]*?clearQueuedOnSwing\(state, playerIndex\);[\s\S]*?clearDeadCasting[\s\S]*?clearQueuedOnSwing\(state, index\);/,
  "WOS55 death, spirit, and resurrection cleanup routes are missing",
);
requireText(
  world,
  /pub queuedOnSwingTerminalStateTest\(\): int[\s\S]*?applyOfflineMobMeleePlayerDeath[\s\S]*?reviveOfflinePlayerAt[\s\S]*?clearDeadCasting/,
  "WOS55 terminal cleanup regression coverage is missing",
);
requireText(main, /\\"world_state\\":\\"WOS67\\"/, "package WOS64 identity is missing");
requireText(protocol, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(protocol, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");

process.stdout.write(`WOS55 queued-on-swing static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
