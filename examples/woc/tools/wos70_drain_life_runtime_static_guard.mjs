import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const abilities = JSON.parse(read("contracts", "m4_abilities.json"));
const drainLife = abilities.entries.find((entry) => entry.id === "drain_life")?.definition;
if (!drainLife || drainLife.class !== "warlock" || drainLife.learnLevel !== 10 ||
    drainLife.cost !== 35 || drainLife.castTime !== 0 || drainLife.cooldown !== 0 ||
    drainLife.range !== 20 || drainLife.school !== "shadow" ||
    drainLife.requiresTarget !== true || drainLife.channel?.duration !== 5 ||
    drainLife.channel?.ticks !== 5 || drainLife.effects?.length !== 1 ||
    drainLife.effects[0].type !== "drainTick" || drainLife.effects[0].min !== 7 ||
    drainLife.effects[0].max !== 7 || drainLife.effects[0].healFrac !== 1) {
  throw new Error("M4 Drain Life contract drifted from the source-pinned rank-one profile");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /drainLifeAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityCode\("drain_life"\)[\s\S]*?m4AbilityCatalog\.indexOf\("drain_life"\)/,
  "Drain Life must have a catalog-backed identity",
);
requireText(
  world,
  /startOfflineDrainLifeCast[\s\S]*?catalogAdmission\(state, casterIndex, abilityCode, "", false\)[\s\S]*?channelDuration[\s\S]*?channelTicks[\s\S]*?cast\.armChannel[\s\S]*?entityResources\[casterIndex\] = <int>state\.entityResources\[casterIndex\] - cost/,
  "Drain Life must bill at channel start and arm a target-locked five-tick channel",
);
requireText(
  world,
  /stepRetainedCasting[\s\S]*?completedAbility == drainLifeAbilityCode\(\)[\s\S]*?cast\.channelTicks > channelTicks[\s\S]*?launchOfflineDrainLifeChannelTick[\s\S]*?entityCastTargetIds\[index\] = <uint>0/,
  "Drain Life must launch every consumed channel tick and clear its target after completion",
);
requireText(
  world,
  /offlineProjectileStateIsValid[\s\S]*?abilityCode == drainLifeAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW[\s\S]*?drainLifeProjectileProfileIsValid/,
  "Drain Life projectile persistence validation is missing",
);
requireText(
  world,
  /landOfflineDrainLifeProjectile[\s\S]*?spellScaling\.channelTickBonus[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?clearOfflineBreakableIncapacitateOnDamage[\s\S]*?applyOfflineDrainLifeHealingThreat/,
  "Drain Life landing must consume one range roll, deal damage, and self-heal",
);
requireText(
  world,
  /applyOfflineDrainLifeHealingThreat[\s\S]*?healState\.applyHealingThreat[\s\S]*?state\.setThreat/,
  "Drain Life must preserve source healing-threat distribution",
);
requireText(
  world,
  /applyOfflineDamageCastPushback[\s\S]*?casting\.pushbackCast[\s\S]*?resolveOfflineEastbrookMobSwingRequests[\s\S]*?applyOfflineDamageCastPushback\(state, playerIndex, dealt\)/,
  "Eastbrook damage must propagate source channel pushback",
);
requireText(
  world,
  /drainLifePayloadAbilityIsExact[\s\S]*?startOfflineDrainLifeCast[\s\S]*?drainLifeCommandStateTest[\s\S]*?entityCastChannelTicks[\s\S]*?offlineProjectileSourceIds\.length != 4[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Drain Life state self-test must cover command, channel, projectile, heal, flush, and typed paths",
);
requireText(
  world,
  /if \(drainLifeCommandStateTest\(\) != 1\) \{[\s\S]*?return -64;/,
  "world selfTest must execute the Drain Life closure",
);

const contract = read("contracts", "world-state.md");
requireText(
  contract,
  /WOS70 Drain Life runtime closure is not a codec revision[\s\S]*?single-player Eastbrook/,
  "world-state contract must state the Drain Life closure and retained scope",
);

process.stdout.write("WOS70 Drain Life runtime static guards passed\n");
