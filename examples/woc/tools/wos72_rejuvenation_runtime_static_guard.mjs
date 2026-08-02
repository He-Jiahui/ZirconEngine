import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const abilities = JSON.parse(read("contracts", "m4_abilities.json"));
const rejuvenation = abilities.entries.find((entry) => entry.id === "rejuvenation")?.definition;
if (!rejuvenation || rejuvenation.class !== "druid" || rejuvenation.learnLevel !== 4 ||
    rejuvenation.cost !== 25 || rejuvenation.castTime !== 0 || rejuvenation.cooldown !== 0 ||
    rejuvenation.range !== 30 || rejuvenation.school !== "nature" ||
    rejuvenation.requiresTarget !== true || rejuvenation.targetType !== "friendly" ||
    rejuvenation.effects?.length !== 1 || rejuvenation.effects[0].type !== "hot" ||
    rejuvenation.effects[0].total !== 32 || rejuvenation.effects[0].duration !== 12 ||
    rejuvenation.effects[0].interval !== 3 || rejuvenation.ranks?.length !== 3 ||
    rejuvenation.ranks[0].rank !== 2 || rejuvenation.ranks[0].level !== 10 ||
    rejuvenation.ranks[0].cost !== 40 || rejuvenation.ranks[0].effects?.[0]?.total !== 56 ||
    rejuvenation.ranks[1].rank !== 3 || rejuvenation.ranks[1].level !== 16 ||
    rejuvenation.ranks[1].cost !== 60 || rejuvenation.ranks[1].effects?.[0]?.total !== 88 ||
    rejuvenation.ranks[2].rank !== 4 || rejuvenation.ranks[2].level !== 20 ||
    rejuvenation.ranks[2].cost !== 80 || rejuvenation.ranks[2].effects?.[0]?.total !== 116) {
  throw new Error("M4 Rejuvenation contract drifted from the source-pinned profile");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /entitySunderArmorValues[\s\S]*?offlineHotStateIsValid[\s\S]*?offlineHotTargetIds[\s\S]*?offlineHotTickTimers/,
  "WOS60 must retain the durable Rejuvenation HoT tail after Sunder Armor",
);
requireText(
  world,
  /schemaVersion != <uint>59 && schemaVersion != <uint>60 &&[\s\S]*?schemaVersion != <uint>61[\s\S]*?schemaVersion >= <uint>60[\s\S]*?offlineHotTargetIds[\s\S]*?offlineHotStateIsValid/,
  "WOS60 must decode its HoT tail while defaulting WOS2-WOS59 snapshots",
);
requireText(
  world,
  /offlineHotStateIsValid[\s\S]*?OFFLINE_DOT_PENDING_MAX[\s\S]*?rejuvenationAbilityCode\(\)[\s\S]*?rejuvenationHotProfileIsValid[\s\S]*?timer > interval/,
  "Rejuvenation state invariants must bound and validate each persisted aura row",
);
requireText(
  world,
  /rejuvenationAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityCode\("rejuvenation"\)[\s\S]*?m4AbilityCatalog\.indexOf\("rejuvenation"\)/,
  "Rejuvenation must have a catalog-backed identity",
);
requireText(
  world,
  /startOfflineRejuvenationCast[\s\S]*?catalogAdmission\(state, casterIndex, abilityCode, "", false\)[\s\S]*?rejuvenationResolvedTargetIndex[\s\S]*?pureHotProfiles\.resolvePureHotProfile[\s\S]*?spendOfflineAbilityResource\(state, casterIndex, cost\)[\s\S]*?applyOfflinePureHot/,
  "Rejuvenation must validate a friendly target, freeze its pure-HoT spell scaling, bill, and apply",
);
requireText(
  world,
  /applyOfflinePureHot[\s\S]*?offlineResolvedHotTotal[\s\S]*?resolvePureHotProfileWithResolvedTotal/,
  "Rejuvenation must resolve talent-adjusted total before capturing its Aura snapshot",
);
requireText(
  world,
  /rejuvenationGlobalCooldownSeconds[\s\S]*?1\.5[\s\S]*?mageSpellHasteMultiplier[\s\S]*?0\.75/,
  "Rejuvenation must use the source hasted global-cooldown floor",
);
requireText(
  world,
  /applyOfflineRejuvenationHot[\s\S]*?removeOfflineRejuvenationHotAt[\s\S]*?offlineHotTargetIds\.length >= <int>OFFLINE_DOT_PENDING_MAX[\s\S]*?offlineHotTickTimers\.add\(interval\)/,
  "Rejuvenation must replace same-target rows and initialize its bounded periodic closure",
);
requireText(
  world,
  /applyOfflineRejuvenationHotTick[\s\S]*?healState\.applyHotTick[\s\S]*?mobs\.healerThreat/,
  "Rejuvenation tick resolution must use the no-RNG hot-heal and effective-healing-threat projection",
);
requireText(
  world,
  /stepOfflineRejuvenationHots[\s\S]*?remaining[\s\S]*?timer[\s\S]*?applyOfflineRejuvenationHotTick/,
  "Rejuvenation's periodic scheduler must age the row and invoke the hot tick",
);
requireText(
  world,
  /stepRetainedPlayerTicks\(state\);[\s\S]*?stepOfflineRejuvenationHots\(state\);[\s\S]*?stepOfflineTravelFormAutoAttack\(state\)/,
  "Rejuvenation must advance in the player-aura phase before retained mob work",
);
requireText(
  world,
  /rejuvenationPayloadAbilityIsExact[\s\S]*?startOfflineRejuvenationCast[\s\S]*?rejuvenationCommandStateTest[\s\S]*?offlineHotTargetIds\.length != 1[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Rejuvenation self-test must cover slot, snapshot, tick, replacement, expiry, and typed command paths",
);
requireText(
  world,
  /if \(rejuvenationCommandStateTest\(\) != 1\) \{[\s\S]*?return -66;/,
  "world selfTest must execute the Rejuvenation closure",
);

const main = read("scripts", "woc_game", "src", "main.zr");
if ((main.match(/world_state[^\r\n]*WOS83/g) ?? []).length !== 2) {
  throw new Error("main schema metadata must publish WOS72 in both runtime paths");
}

const contract = read("contracts", "world-state.md");
requireText(
  contract,
  /WOS72 adds schema 60[\s\S]*?Rejuvenation[\s\S]*?without RNG or heal crit\/absorb processing/,
  "world-state contract must document the WOS72 Rejuvenation closure",
);

process.stdout.write("WOS72 Rejuvenation runtime static guards passed\n");
