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
const start = classes.indexOf("  flame_shock: {");
const end = classes.indexOf("  flametongue_weapon: {", start);
const flameShock = classes.slice(start, end);
for (const needle of ["class: 'shaman'", "learnLevel: 8", "cost: 35", "castTime: 0", "cooldown: 6", "range: 20", "school: 'fire'", "type: 'directDamage', min: 25, max: 25", "type: 'dot', total: 28, duration: 12, interval: 3", "level: 16", "cost: 55", "type: 'directDamage', min: 42, max: 42", "type: 'dot', total: 48, duration: 12, interval: 3"]) {
  if (!flameShock.includes(needle)) throw new Error(`source Flame Shock drifted: ${needle}`);
}
requireText(casting, /SHAMAN_SHOCK_COOLDOWN_IDS = \['earth_shock', 'flame_shock', 'frost_shock'\][\s\S]*?for \(const id of SHAMAN_SHOCK_COOLDOWN_IDS\) p\.cooldowns\.set\(id, cooldown\)/, "source Shock cooldown drifted");
requireText(casting, /firesProjectile = ability\.projectile \?\? ability\.school !== 'physical'[\s\S]*?isSpellResisted[\s\S]*?ctx\.runEffects/, "source spell projectile ordering drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/frost_shock',[\s\S]*?'flame_shock'/.test(generator) || !generator.includes("EXPECTED_ABILITY_COUNT = 79") || !zrGenerator.includes("document.entries.length === 79")) throw new Error("M4 Flame Shock scope is missing");
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "flame_shock");
if (!entry || entry.index !== 36 || entry.definition.school !== "fire" || entry.definition.effects?.[0]?.type !== "directDamage" || entry.definition.effects?.[1]?.type !== "dot") throw new Error("M4 Flame Shock projection drifted");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /flameShockAbilityCode\([\s\S]*?startOfflineFlameShockCast/, "Flame Shock reducer is missing");
requireText(world, /startOfflineFlameShockCast[\s\S]*?shamanShockCooldownIsActive[\s\S]*?armShamanShockCooldown[\s\S]*?appendOfflineAbilityProjectile/, "Flame Shock admission is missing");
requireText(world, /landOfflineFlameShockProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?applyOfflineFlameShockDot/, "Flame Shock impact is missing");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?flameShockAbilityCode\(\)[\s\S]*?landOfflineFlameShockProjectile/, "Flame Shock projectile dispatch is missing");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?flameShockAbilityCode\(\)[\s\S]*?m4AbilityCatalog\.indexOf\("flame_shock"\)/, "Flame Shock periodic threat is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?flameShockAbilityCode\(\)[\s\S]*?startOfflineFlameShockCast[\s\S]*?applySupportedCastCommand[\s\S]*?flameShockPayloadAbilityIsExact/, "Flame Shock command routes are missing");
requireText(world, /pub flameShockCommandStateTest\(\): int[\s\S]*?flame_shock[\s\S]*?offlineProjectileSourceIds[\s\S]*?offlineDotDamages[\s\S]*?abilityCooldownExpiresAt[\s\S]*?rngDraws/, "Flame Shock state regression coverage is missing");
process.stdout.write(`WOS96 Flame Shock static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
