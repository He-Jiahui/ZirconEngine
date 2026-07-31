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
const abilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const earthShockSource = abilityBlock("earth_shock", "lightning_shield");
for (const needle of [
  "class: 'shaman'", "learnLevel: 4", "cost: 30", "castTime: 0", "cooldown: 6",
  "range: 20", "school: 'nature'", "type: 'directDamage', min: 19, max: 22",
  "rank: 2", "level: 10", "cost: 45", "min: 33, max: 38",
  "rank: 3", "level: 16", "cost: 65", "min: 54, max: 61",
]) {
  if (!earthShockSource.includes(needle)) throw new Error(`source Earth Shock drifted: ${needle}`);
}
requireText(
  casting,
  /SHAMAN_SHOCK_COOLDOWN_IDS = \['earth_shock', 'flame_shock', 'frost_shock'\][\s\S]*?isShamanShock[\s\S]*?sharedCooldown[\s\S]*?SHAMAN_SHOCK_COOLDOWN_IDS\.find[\s\S]*?for \(const id of SHAMAN_SHOCK_COOLDOWN_IDS\) p\.cooldowns\.set\(id, cooldown\)/,
  "source Shaman Shock shared cooldown drifted",
);
requireText(
  casting,
  /firesProjectile = ability\.projectile \?\? ability\.school !== 'physical'[\s\S]*?spendAbilityCost[\s\S]*?armAbilityCooldown[\s\S]*?scheduleProjectile[\s\S]*?isSpellResisted[\s\S]*?ctx\.runEffects/,
  "source instant nonphysical projectile ordering drifted",
);
requireText(
  dispatch,
  /case 'directDamage':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?directHitBonus[\s\S]*?ctx\.dealDamage/,
  "source direct-damage dispatch drifted",
);

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'healing_wave',[\s\S]*?'earth_shock'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Earth Shock projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const earthShock = m4.entries.find((entry) => entry.id === "earth_shock");
if (!earthShock || earthShock.index !== 34 || earthShock.definition.school !== "nature" ||
    earthShock.definition.castTime !== 0 || earthShock.definition.cooldown !== 6 ||
    earthShock.definition.effects?.[0]?.type !== "directDamage" ||
    earthShock.definition.effects?.[0]?.min !== 19 || earthShock.definition.ranks?.length !== 2) {
  throw new Error("M4 Earth Shock source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /shamanShockCooldownIsActive[\s\S]*?earth_shock[\s\S]*?flame_shock[\s\S]*?frost_shock[\s\S]*?setAbilityCooldownExpiration/, "Earth Shock shared cooldown projection is missing");
requireText(world, /earthShockAbilityCode\([\s\S]*?startOfflineEarthShockCast/, "Earth Shock cast reducer is missing");
requireText(world, /startOfflineEarthShockCast[\s\S]*?shamanShockCooldownIsActive[\s\S]*?entityResources\[casterIndex\][\s\S]*?armShamanShockCooldown[\s\S]*?appendOfflineAbilityProjectile/, "Earth Shock must bill, arm the shared cooldown and launch immediately");
requireText(world, /landOfflineEarthShockProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?enterOfflineSpellProjectileCombat/, "Earth Shock projectile impact is missing");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?earthShockAbilityCode\(\)[\s\S]*?earthShockProjectileProfileIsValid/, "Earth Shock projectile snapshot validation is missing");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?earthShockAbilityCode\(\)[\s\S]*?landOfflineEarthShockProjectile/, "Earth Shock projectile dispatch is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?earthShockAbilityCode\(\)[\s\S]*?startOfflineEarthShockCast[\s\S]*?applySupportedCastCommand[\s\S]*?earthShockPayloadAbilityIsExact/, "Earth Shock slot and typed routes are missing");
requireText(world, /pub earthShockCommandStateTest\(\): int[\s\S]*?earth_shock[\s\S]*?offlineProjectileSourceIds[\s\S]*?abilityCooldownExpiresAt[\s\S]*?rngDraws/, "Earth Shock state regression coverage is missing");

process.stdout.write(`WOS94 Earth Shock static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
