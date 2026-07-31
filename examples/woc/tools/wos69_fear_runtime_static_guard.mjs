import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const abilities = JSON.parse(read("contracts", "m4_abilities.json"));
const fear = abilities.entries.find((entry) => entry.id === "fear")?.definition;
if (!fear || fear.class !== "warlock" || fear.learnLevel !== 14 || fear.cost !== 40 ||
    fear.castTime !== 1.5 || fear.cooldown !== 0 || fear.range !== 20 ||
    fear.school !== "shadow" || fear.requiresTarget !== true || fear.fearDr !== true ||
    fear.effects?.length !== 1 || fear.effects[0].type !== "incapacitate" ||
    fear.effects[0].duration !== 8) {
  throw new Error("M4 Fear contract drifted from the source-pinned rank-one profile");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /fearAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityCode\("fear"\)[\s\S]*?m4AbilityCatalog\.indexOf\("fear"\)/,
  "Fear must have a catalog-backed identity",
);
requireText(
  world,
  /startOfflineFearCast[\s\S]*?catalogAdmission\(state, casterIndex, abilityCode, "", false\)[\s\S]*?fearTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?fearCastSeconds[\s\S]*?fearGlobalCooldownSeconds[\s\S]*?entityCastTargetIds\[casterIndex\] = targetId/,
  "Fear must arm a target-locked source-timed cast",
);
requireText(
  world,
  /completeOfflineFearCast[\s\S]*?entityCastTargetIds\[casterIndex\] = <uint>0[\s\S]*?entityResources\[casterIndex\][\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW[\s\S]*?0\.0,[\s\S]*?0\.0/,
  "Fear must spend at completion and snapshot a zero-damage Shadow projectile",
);
requireText(
  world,
  /offlineProjectileStateIsValid[\s\S]*?abilityCode == fearAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW[\s\S]*?fearProjectileProfileIsValid/,
  "Fear projectile persistence validation is missing",
);
requireText(
  world,
  /landOfflineFearProjectile[\s\S]*?spellResist\.resolve[\s\S]*?offlineFearDuration[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?applyOfflineMotionAuraWithDetails[\s\S]*?"incapacitate"[\s\S]*?0\.1/,
  "Fear landing must resolve spell resistance, direction, and graded break data",
);
requireText(
  world,
  /clearOfflineBreakableIncapacitateOnDamage[\s\S]*?abilityCode != gougeCode && abilityCode != fearCode[\s\S]*?amount \/ \(scale \* <float>maximum\)[\s\S]*?nextAuthoritativeRandomUnit\(state\) >= chance[\s\S]*?removeMotionAuraAt/,
  "Fear damage break must retain the source graded probability reducer",
);
requireText(
  world,
  /stepOfflineEastbrookFearMovement[\s\S]*?fleeSpeed\.fleeSpeed[\s\S]*?projectileTravel\.projectileTickSeconds\(\)[\s\S]*?terrainGround\.builtinGroundHeight[\s\S]*?entityFacing\[index\] = direction/,
  "Fear must use the retained flee-speed movement path",
);
requireText(
  world,
  /stepOfflineEastbrookMobMeleePursuit[\s\S]*?!stepOfflineEastbrookFearMovement\(state, index\)[\s\S]*?!motionAuraEntityIsStunned/,
  "Fear movement must suppress ordinary pursuit and melee",
);
requireText(
  world,
  /fearPayloadAbilityIsExact[\s\S]*?startOfflineFearCast[\s\S]*?completedAbility == fearAbilityCode\(\)[\s\S]*?completeOfflineFearCast/,
  "Fear must route both typed commands and completed casts",
);
requireText(
  world,
  /fearCommandStateTest[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW[\s\S]*?entityMotionAuraBreakChanceScales[\s\S]*?stepOfflineEastbrookMobMeleePursuit[\s\S]*?clearOfflineBreakableIncapacitateOnDamage[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Fear state self-test must cover cast, persistence, movement, break, and typed command paths",
);
requireText(
  world,
  /if \(fearCommandStateTest\(\) != 1\) \{[\s\S]*?return -63;/,
  "world selfTest must execute the Fear closure",
);

const contract = read("contracts", "world-state.md");
requireText(
  contract,
  /The Fear runtime closure[\s\S]*?single-player Eastbrook[\s\S]*?PvP hostility[\s\S]*?not\s+active/,
  "world-state contract must state the Fear closure and retained PvP boundary",
);

process.stdout.write("WOS69 Fear runtime static guards passed\n");
