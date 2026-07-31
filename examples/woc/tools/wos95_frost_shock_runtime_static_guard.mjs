import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_COMMIT = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = path.resolve(root, "..", "..");
const sourceRoot = path.resolve(workspaceRoot, "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const source = (file) => execFileSync("git", ["-C", sourceRoot, "show", `${SOURCE_COMMIT}:${file}`], { encoding: "utf8" });
const requireText = (text, pattern, message) => { if (!pattern.test(text)) throw new Error(message); };

const classes = source("src/sim/content/classes.ts");
const casting = source("src/sim/combat/casting_lifecycle.ts");
const start = classes.indexOf("  frost_shock: {");
const end = classes.indexOf("  frostbrand_weapon: {", start);
const frostShock = classes.slice(start, end);
for (const needle of ["class: 'shaman'", "learnLevel: 8", "cost: 50", "castTime: 0", "cooldown: 6", "range: 20", "school: 'frost'", "type: 'directDamage', min: 36, max: 42", "type: 'slow', mult: 0.5, duration: 8"]) {
  if (!frostShock.includes(needle)) throw new Error(`source Frost Shock drifted: ${needle}`);
}
requireText(casting, /SHAMAN_SHOCK_COOLDOWN_IDS = \['earth_shock', 'flame_shock', 'frost_shock'\][\s\S]*?for \(const id of SHAMAN_SHOCK_COOLDOWN_IDS\) p\.cooldowns\.set\(id, cooldown\)/, "source Shock cooldown drifted");
requireText(casting, /firesProjectile = ability\.projectile \?\? ability\.school !== 'physical'[\s\S]*?isSpellResisted[\s\S]*?ctx\.runEffects/, "source spell projectile ordering drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/earth_shock',[\s\S]*?'frost_shock'/.test(generator) || !generator.includes("EXPECTED_ABILITY_COUNT = 79") || !zrGenerator.includes("document.entries.length === 79")) throw new Error("M4 Frost Shock scope is missing");
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "frost_shock");
if (!entry || entry.index !== 35 || entry.definition.school !== "frost" || entry.definition.effects?.[1]?.type !== "slow") throw new Error("M4 Frost Shock projection drifted");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /frostShockAbilityCode\([\s\S]*?startOfflineFrostShockCast/, "Frost Shock reducer is missing");
requireText(world, /startOfflineFrostShockCast[\s\S]*?shamanShockCooldownIsActive[\s\S]*?armShamanShockCooldown[\s\S]*?appendOfflineAbilityProjectile/, "Frost Shock admission is missing");
requireText(world, /landOfflineFrostShockProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?applyOfflineMotionAura/, "Frost Shock impact is missing");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?frostShockAbilityCode\(\)[\s\S]*?frostShockProjectileProfileIsValid/, "Frost Shock projectile snapshot validation is missing");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?frostShockAbilityCode\(\)[\s\S]*?landOfflineFrostShockProjectile/, "Frost Shock dispatch is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?frostShockAbilityCode\(\)[\s\S]*?startOfflineFrostShockCast[\s\S]*?applySupportedCastCommand[\s\S]*?frostShockPayloadAbilityIsExact/, "Frost Shock command routes are missing");
requireText(world, /pub frostShockCommandStateTest\(\): int[\s\S]*?frost_shock[\s\S]*?offlineProjectileSourceIds[\s\S]*?abilityCooldownExpiresAt[\s\S]*?rngDraws/, "Frost Shock state regression coverage is missing");
process.stdout.write(`WOS95 Frost Shock static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
