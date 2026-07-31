import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
const autoAttack = read("scripts", "woc_game", "src", "combat", "auto_attack_state.zr");

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "current WOS writer is missing");
requireText(
  world,
  /schemaVersion != <uint>53 &&\s*schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS53-WOS55 decoder admission is missing",
);
requireText(
  world,
  /if \(schemaVersion >= <uint>54\) \{\s*state\.offlineProjectileSchoolCodes\.add\(reader\.byte\(1\)\);\s*\} else \{\s*state\.offlineProjectileSchoolCodes\.add\(OFFLINE_PROJECTILE_SCHOOL_PHYSICAL\);/,
  "WOS53 physical school migration is missing",
);
requireText(
  world,
  /offlineProjectileWands: container\.Array<bool>;\s*pub var offlineProjectileSchoolCodes: container\.Array<uint>;/,
  "projectile school column is missing",
);
requireText(
  world,
  /writer\.byte\(<uint>state\.offlineProjectileSchoolCodes\[projectileIndex\], 1\);/,
  "projectile school encoding is missing",
);
requireText(
  world,
  /configureOfflineCasterWand[\s\S]*?rangedWand = true;[\s\S]*?rangedMaximumRange = 30\.0;[\s\S]*?rangedMinimum = 3\.0;[\s\S]*?rangedMaximum = 6\.0;[\s\S]*?rangedSpeed = 1\.8/,
  "caster wand source profile is incomplete",
);
requireText(
  world,
  /if \(classId == "mage"\) \{ return "arcane"; \}[\s\S]*?if \(classId == "priest"\) \{ return "holy"; \}[\s\S]*?if \(classId == "warlock"\) \{ return "shadow"; \}/,
  "caster school mapping is incomplete",
);
requireText(
  world,
  /forms\.formAbilityCode\("form_bear"\)[\s\S]*?forms\.formAbilityCode\("form_cat"\)[\s\S]*?forms\.formAbilityCode\("form_travel"\)/,
  "druid wand form gate is incomplete",
);
requireText(
  world,
  /stepOfflineTravelFormAutoAttack[\s\S]*?entitySwingTimer[\s\S]*?entityOffhandSwingTimer[\s\S]*?entityAutoAttack\[playerIndex\] = false/,
  "travel-form auto-attack cancellation is missing",
);
requireText(
  world,
  /startOfflineAutoAttack\(state: WorldState, actorIndex: int\)[\s\S]*?forms\.isTravelFormAbilityCode\(/,
  "travel-form startAutoAttack gate is missing",
);
requireText(
  world,
  /commandId == attackCommand[\s\S]*?startOfflineAutoAttack\(this, actorIndex\);/,
  "attack command must share the source startAutoAttack helper",
);
requireText(
  world,
  /var travelAttack = new WorldState\(\);[\s\S]*?travelAttack\.entityAggroTargetIds\[1\] != <uint>0/,
  "travel-form attack regression coverage is missing",
);
requireText(
  autoAttack,
  /offhandWeaponMinimum: float;[\s\S]*?offhandWeaponSpeed: float;[\s\S]*?hasOffhandWeapon: bool;[\s\S]*?dualWielding: bool;/,
  "dual-wield actor projection is missing",
);
requireText(
  autoAttack,
  /actor\.swingTimer = actor\.swingTimer - 0\.05;[\s\S]*?actor\.offhandSwingTimer = actor\.offhandSwingTimer - 0\.05;[\s\S]*?actor\.swingTimer > 0\.0 && \(!actor\.dualWielding \|\| !actor\.hasOffhandWeapon \|\|[\s\S]*?actor\.offhandSwingTimer > 0\.0\)/,
  "dual-wield ready-hand timing is missing",
);
requireText(
  autoAttack,
  /if \(whiteDualWieldPenalty\) \{ missChance = missChance \+ 0\.1; \}/,
  "dual-wield white miss penalty is missing",
);
requireText(
  autoAttack,
  /offhandMeleeSwing[\s\S]*?0\.5,[\s\S]*?actor\.offhandWeaponSpeed,[\s\S]*?true,[\s\S]*?actor\.offhandSwingTimer = actor\.offhandWeaponSpeed/,
  "offhand damage multiplier or cadence is missing",
);
requireText(
  world,
  /actor\.offhandWeaponMinimum = <float>state\.entityOffhandWeaponMinimum\[playerIndex\];[\s\S]*?actor\.dualWielding = <bool>state\.entityDualWielding\[playerIndex\];[\s\S]*?actor\.offhandSwingTimer = <float>state\.entityOffhandSwingTimer\[playerIndex\];[\s\S]*?state\.entityOffhandSwingTimer\[playerIndex\] = actor\.offhandSwingTimer;/,
  "world-state offhand bridge is incomplete",
);
requireText(
  world,
  /playerDualMelee[\s\S]*?entityHasOffhandWeapon\[0\] = true;[\s\S]*?entityDualWielding\[0\] = true;[\s\S]*?entityOffhandSwingTimer\[0\] != 1\.5/,
  "dual-wield world-state coverage is missing",
);
requireText(
  world,
  /stepOfflineHunterAutoShot\(state\);\s*stepOfflineCasterWandAutoAttack\(state\);/,
  "ranged auto dispatch is missing",
);
requireText(world, /offlineHunterAutoShotStateTest[\s\S]*?form_moonkin/, "ranged state coverage is missing");
requireText(main, /\\"world_state\\":\\"WOS67\\"/, "package WOS64 identity is missing");
requireText(protocol, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(protocol, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");

process.stdout.write("WOS54 ranged-auto compatibility guards passed\n");
