import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const abilities = JSON.parse(read("contracts", "m4_abilities.json"));
const sunder = abilities.entries.find((entry) => entry.id === "sunder_armor")?.definition;
if (!sunder || sunder.class !== "warrior" || sunder.learnLevel !== 5 ||
    sunder.specs?.length !== 1 || sunder.specs[0] !== "prot" || sunder.cost !== 15 ||
    sunder.castTime !== 0 || sunder.cooldown !== 0 || sunder.range !== 0 ||
    sunder.school !== "physical" || sunder.requiresTarget !== true ||
    sunder.threat?.flat !== 100 || sunder.effects?.length !== 1 ||
    sunder.effects[0].type !== "sunder" || sunder.effects[0].armor !== 25 ||
    sunder.effects[0].maxStacks !== 5 || sunder.ranks?.[0]?.rank !== 2 ||
    sunder.ranks[0].level !== 16 || sunder.ranks[0].threatFlat !== 130 ||
    sunder.ranks[0].effects?.[0]?.armor !== 40 ||
    sunder.ranks[0].effects[0].maxStacks !== 5) {
  throw new Error("M4 Sunder Armor contract drifted from the source-pinned profile");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /writer\.u16\(<uint>78, 1, 1\)[\s\S]*?entitySunderArmorStacks[\s\S]*?entitySunderArmorRemaining[\s\S]*?entitySunderArmorValues/,
  "WOS59 must retain the durable Sunder Armor tail in the current codec",
);
requireText(
  world,
  /schemaVersion != <uint>58 &&[\s\S]*?schemaVersion != <uint>59 && schemaVersion != <uint>60 &&[\s\S]*?schemaVersion != <uint>61[\s\S]*?schemaVersion >= <uint>59[\s\S]*?historicalSunderIndex/,
  "WOS59 must decode its Sunder tail and default legacy snapshots",
);
requireText(
  world,
  /sunderArmorStateIsValid[\s\S]*?stacks < 0 \|\| stacks > 5[\s\S]*?remaining > 30\.0[\s\S]*?stacks == 0/,
  "Sunder state invariants must bound and pair stacks, duration, and value",
);
requireText(
  world,
  /sunderArmorAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityCode\("sunder_armor"\)[\s\S]*?m4AbilityCatalog\.indexOf\("sunder_armor"\)/,
  "Sunder Armor must have a catalog-backed identity",
);
requireText(
  world,
  /startOfflineSunderArmorCast[\s\S]*?catalogAdmission\(state, casterIndex, abilityCode, "", false\)[\s\S]*?sunderArmorTargetIndex[\s\S]*?entityResources\[casterIndex\] = <int>state\.entityResources\[casterIndex\] - cost[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?sourceSwingMissChance[\s\S]*?enterOfflineSunderArmorCombat/,
  "Sunder Armor must validate, bill, draw one melee miss roll, and enter combat",
);
requireText(
  world,
  /if \(missed\) \{ return; \}[\s\S]*?entitySunderArmorStacks\[targetIndex\][\s\S]*?entitySunderArmorRemaining\[targetIndex\] = 30\.0[\s\S]*?entitySunderArmorValues\[targetIndex\][\s\S]*?state\.setThreat/,
  "A landed Sunder Armor must refresh/cap its aura before adding flat threat",
);
requireText(
  world,
  /sunderArmorGlobalCooldownSeconds[\s\S]*?1\.5[\s\S]*?mageSpellHasteMultiplier[\s\S]*?0\.75/,
  "Sunder Armor must preserve the source hasted GCD floor",
);
requireText(
  world,
  /sunderArmorReduction[\s\S]*?0\.02 \* <float>stacks[\s\S]*?effectiveOfflineArmor[\s\S]*?sunderArmorReduction[\s\S]*?prepareOfflineEastbrookAutoTarget[\s\S]*?target\.armor = effectiveOfflineArmor/,
  "Physical auto-attack targets must consume active Sunder Armor stacks",
);
requireText(
  world,
  /eviscerateGlobalCooldownSeconds[\s\S]*?effectiveOfflineArmor\(state, targetIndex\)[\s\S]*?gougeGlobalCooldownSeconds[\s\S]*?effectiveOfflineArmor\(state, targetIndex\)/,
  "Direct retained physical strikes must consume active Sunder Armor stacks",
);
requireText(
  world,
  /ageOfflineSunderArmor[\s\S]*?remaining > 0\.05[\s\S]*?entitySunderArmorStacks\[index\] = 0[\s\S]*?fixedTick[\s\S]*?ageOfflineSunderArmor\(state\)/,
  "Sunder Armor must age and clear from the authoritative fixed tick",
);
requireText(
  world,
  /sunderArmorPayloadAbilityIsExact[\s\S]*?startOfflineSunderArmorCast[\s\S]*?sunderArmorCommandStateTest[\s\S]*?entitySunderArmorStacks\[1\] != 5[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Sunder Armor self-test must cover slot, cap, snapshot, expiry, and typed command paths",
);
requireText(
  world,
  /if \(sunderArmorCommandStateTest\(\) != 1\) \{[\s\S]*?return -65;/,
  "world selfTest must execute the Sunder Armor closure",
);

const main = read("scripts", "woc_game", "src", "main.zr");
if ((main.match(/world_state[^\r\n]*WOS78/g) ?? []).length !== 2) {
  throw new Error("main schema metadata must publish WOS72 in both runtime paths");
}

const contract = read("contracts", "world-state.md");
requireText(
  contract,
  /WOS71 adds the Sunder Armor tail[\s\S]*?exactly one player melee[\s\S]*?0\.02 \* stacks/,
  "world-state contract must document the WOS71 Sunder closure and armor rule",
);

process.stdout.write("WOS71 Sunder Armor runtime static guards passed\n");
