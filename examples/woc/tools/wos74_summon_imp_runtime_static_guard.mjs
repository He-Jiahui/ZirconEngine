import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const abilities = JSON.parse(read("contracts", "m4_abilities.json"));
const summonImp = abilities.entries.find((entry) => entry.id === "summon_imp")?.definition;
if (!summonImp || summonImp.class !== "warlock" || summonImp.learnLevel !== 1 ||
    summonImp.cost !== 50 || summonImp.castTime !== 5 || summonImp.cooldown !== 0 ||
    summonImp.range !== 0 || summonImp.school !== "shadow" ||
    summonImp.requiresTarget !== false || summonImp.effects?.length !== 1 ||
    summonImp.effects[0].type !== "summonDemon" || summonImp.effects[0].mobId !== "emberkin") {
  throw new Error("M4 Summon Imp contract drifted from the source-pinned profile");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /OFFLINE_EMBERKIN_TEMPLATE_CODE: uint = <uint>48[\s\S]*?OFFLINE_EMBERKIN_HP_BASE: int = 30[\s\S]*?OFFLINE_EMBERKIN_MOVE_SPEED: float = 5\.2[\s\S]*?OFFLINE_EMBERKIN_PRESENTATION_SCALE: float = 0\.55/, "WOS74 must pin the Emberkin source profile");
requireText(world, /summonImpAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityCode\("summon_imp"\)[\s\S]*?m4AbilityCatalog\.text\(abilityIndex, "class"\) != "warlock"/, "Summon Imp must resolve its generated catalog identity");
requireText(world, /startOfflineSummonImpCast[\s\S]*?summonImpProfileIsValid[\s\S]*?cast\.armTimed\([\s\S]*?summonImpCastSeconds[\s\S]*?summonImpGlobalCooldownSeconds/, "Summon Imp must use the source timed cast and haste-aware GCD");
requireText(world, /completeOfflineSummonImpCast[\s\S]*?entityResources\[casterIndex\] = <int>state\.entityResources\[casterIndex\] - cost[\s\S]*?retireOfflineOwnedPet[\s\S]*?spawnOfflineEmberkin/, "Summon Imp must bill on completion then replace the live pet");
requireText(world, /spawnOfflineEmberkin[\s\S]*?state\.spawnMob\([\s\S]*?terrainGround\.builtinGroundHeight[\s\S]*?entityOwnerIds\[petIndex\][\s\S]*?entityHostile\[petIndex\] = false[\s\S]*?entityPresentationScales\[petIndex\]/, "Summon Imp must create a friendly source-profile Emberkin");
requireText(world, /retireOfflineOwnedPet[\s\S]*?entityDead\[petIndex\] = true[\s\S]*?clearStateThreat/, "WOS74 must retain the temporary inert-row replacement boundary");
requireText(world, /summonImpPayloadAbilityIsExact[\s\S]*?startOfflineSummonImpCast[\s\S]*?completedAbility == summonImpAbilityCode\(\)[\s\S]*?completeOfflineSummonImpCast/, "Summon Imp must route typed casts and completion");
requireText(world, /summonImpCommandStateTest[\s\S]*?appendCastSlotCommand[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastCommandForTest[\s\S]*?offlineOwnedLivingPetIndex/, "Summon Imp self-test must cover bar, persistence, typed replacement, and one live pet");
requireText(world, /if \(summonImpCommandStateTest\(\) != 1\) \{[\s\S]*?return -68;/, "world selfTest must execute Summon Imp");

const main = read("scripts", "woc_game", "src", "main.zr");
if ((main.match(/world_state[^\r\n]*WOS71/g) ?? []).length !== 2) throw new Error("WOS74 must retain the current WOS71 schema");
requireText(read("contracts", "world-state.md"), /WOS74 retains M4 Warlock `summon_imp`[\s\S]*?inert, dead, owner-bound row/, "contract must document WOS74 and the current entity-removal boundary");
process.stdout.write("WOS74 Summon Imp runtime static guards passed\n");
