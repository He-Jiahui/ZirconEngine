import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const abilities = JSON.parse(read("contracts", "m4_abilities.json"));
const consecration = abilities.entries.find((entry) => entry.id === "consecration")?.definition;
if (!consecration || consecration.class !== "paladin" || consecration.learnLevel !== 8 ||
    consecration.cost !== 60 || consecration.castTime !== 0 || consecration.cooldown !== 8 ||
    consecration.range !== 0 || consecration.school !== "holy" ||
    consecration.requiresTarget !== false || consecration.effects?.length !== 1 ||
    consecration.effects[0].type !== "groundAoE" || consecration.effects[0].min !== 28 ||
    consecration.effects[0].max !== 34 || consecration.effects[0].radius !== 8 ||
    consecration.effects[0].duration !== 10 || consecration.effects[0].interval !== 2) {
  throw new Error("M4 Consecration contract drifted from the source-pinned profile");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /writer\.u16\(<uint>71, 1, 1\)[\s\S]*?offlineGroundEffectStateIsValid[\s\S]*?offlineGroundEffectSourceIds[\s\S]*?offlineGroundEffectTickTimers/, "WOS61 must retain Consecration ground state in the current codec");
requireText(world, /schemaVersion != <uint>60 &&[\s\S]*?schemaVersion != <uint>61[\s\S]*?schemaVersion >= <uint>61[\s\S]*?offlineGroundEffectSourceIds/, "WOS61 must decode ground state and default WOS2-WOS60");
requireText(world, /consecrationAbilityCode\(\)[\s\S]*?knownAbilityCatalog\.abilityCode\("consecration"\)[\s\S]*?startOfflineConsecrationCast[\s\S]*?setAbilityCooldownExpiration[\s\S]*?resolveOfflineGroundAoEPulse[\s\S]*?appendOfflineConsecrationGroundEffect/, "Consecration must bill, cooldown, pulse, and persist its zone");
requireText(world, /stepOfflineConsecrationGroundEffects[\s\S]*?index = state\.offlineGroundEffectSourceIds\.length - 1[\s\S]*?remaining[\s\S]*?timer[\s\S]*?resolveOfflineGroundAoEPulse[\s\S]*?fixedTick[\s\S]*?stepOfflineConsecrationGroundEffects\(state\)[\s\S]*?stepOfflineEastbrookProjectiles/, "Consecration must preserve source zone ordering and cadence");
requireText(world, /consecrationPayloadAbilityIsExact[\s\S]*?startOfflineConsecrationCast/, "Consecration must route typed commands");
requireText(world, /consecrationCommandStateTest[\s\S]*?offlineGroundEffectSourceIds\.length != 1[\s\S]*?stepOfflineConsecrationGroundEffects[\s\S]*?appendTypedCastCommandForTest/, "Consecration self-test must cover command, snapshot, pulse, expiry, and typed routes");
requireText(world, /if \(consecrationCommandStateTest\(\) != 1\) \{[\s\S]*?return -67;/, "world selfTest must execute Consecration");

const main = read("scripts", "woc_game", "src", "main.zr");
if ((main.match(/world_state[^\r\n]*WOS71/g) ?? []).length !== 2) throw new Error("main must publish WOS71");
requireText(read("contracts", "world-state.md"), /WOS73 adds schema 61[\s\S]*?Consecration[\s\S]*?before projectiles/, "contract must document WOS73");
process.stdout.write("WOS73 Consecration runtime static guards passed\n");
