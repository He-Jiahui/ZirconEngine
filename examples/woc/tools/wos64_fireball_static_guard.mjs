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
const effects = source("src/sim/combat/effect_dispatch.ts");
const auras = source("src/sim/combat/auras.ts");
requireText(
  classes,
  /fireball:\s*\{[\s\S]*?learnLevel: 1,[\s\S]*?cost: 30,[\s\S]*?castTime: 1\.5,[\s\S]*?range: 30,[\s\S]*?school: 'fire',[\s\S]*?directDamage', min: 16, max: 25[\s\S]*?dot', total: 2, duration: 4, interval: 2/,
  "source Fireball definition drifted",
);
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?scheduleProjectile\(ctx, p, target, \(src, tgt\) => \{[\s\S]*?isSpellResisted\(ctx\.rng, src\.level, tgt\.level, src\.hitBonus\)[\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\);/,
  "source Fireball projectile and resist ordering drifted",
);
requireText(
  effects,
  /case 'dot':[\s\S]*?const hybrid = res\.effects\.some[\s\S]*?const dotBase = Math\.max\(1, Math\.round\(dotTotal \/ \(eff\.duration \/ eff\.interval\)\)\)[\s\S]*?remaining: eff\.duration,[\s\S]*?tickInterval: eff\.interval,[\s\S]*?tickTimer: eff\.interval,[\s\S]*?sourceId: p\.id/,
  "source Fireball DoT snapshot application drifted",
);
requireText(
  auras,
  /a\.remaining -= DT;[\s\S]*?a\.tickTimer = \(a\.tickTimer \?\? a\.tickInterval\) - DT;[\s\S]*?a\.tickTimer <= CAST_COMPLETE_EPS[\s\S]*?a\.tickTimer \+= a\.tickInterval[\s\S]*?a\.kind === 'dot'[\s\S]*?ctx\.dealDamage[\s\S]*?a\.remaining <= CAST_COMPLETE_EPS/,
  "source DoT 20 Hz aging and expiry ordering drifted",
);

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const fireball = m4.entries.find((entry) => entry.id === "fireball");
if (!fireball || fireball.index !== 1 || fireball.definition.class !== "mage" ||
    fireball.definition.cost !== 30 || fireball.definition.castTime !== 1.5 ||
    fireball.definition.school !== "fire" ||
    fireball.definition.effects?.[0]?.type !== "directDamage" ||
    fireball.definition.effects?.[0]?.min !== 16 || fireball.definition.effects?.[0]?.max !== 25 ||
    fireball.definition.effects?.[1]?.type !== "dot" ||
    fireball.definition.effects?.[1]?.total !== 2 || fireball.definition.effects?.[1]?.duration !== 4 ||
    fireball.definition.effects?.[1]?.interval !== 2) {
  throw new Error("M4 Fireball projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /fireballAbilityCode\([\s\S]*?abilityCode\("fireball"\)[\s\S]*?m4AbilityCatalog\.indexOf\("fireball"\)/, "Fireball identity is missing");
requireText(world, /startOfflineFireballCast[\s\S]*?cast\.armTimed[\s\S]*?entityCastTargetIds/, "Fireball start reducer is missing");
requireText(world, /completeOfflineFireballCast[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/, "Fireball completion must queue a fire projectile");
requireText(world, /landOfflineFireballProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?applyOfflineFireballDot/, "Fireball landing reducer is missing");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?offlineDotTickTimers[\s\S]*?settleOfflineEastbrookLethal/, "Fireball DoT lifecycle is missing");
requireText(world, /writer\.u16\(<uint>67, 1, 1\)[\s\S]*?offlineDotTargetIds[\s\S]*?schemaVersion != <uint>57 && schemaVersion != <uint>58 &&[\s\S]*?schemaVersion != <uint>59 && schemaVersion != <uint>60 &&[\s\S]*?schemaVersion != <uint>61[\s\S]*?schemaVersion >= <uint>57/, "WOS57 Fireball DoT codec is missing");
requireText(world, /stepRetainedCasting[\s\S]*?completedAbility == fireballAbilityCode\(\)[\s\S]*?completeOfflineFireballCast/, "Fireball must complete before target lock cleanup");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?fireballAbilityCode\(\)[\s\S]*?startOfflineFireballCast[\s\S]*?applySupportedCastCommand[\s\S]*?fireballPayloadAbilityIsExact/, "Fireball slot and typed routes are missing");
requireText(world, /pub fireballCommandStateTest\(\): int[\s\S]*?offlineDotTargetIds[\s\S]*?stepOfflineEastbrookDots/, "Fireball state regression coverage is missing");
requireText(world, /if \(fireballCommandStateTest\(\) != 1\) \{\s*return -60;\s*\}/, "Fireball self-test route is missing");

process.stdout.write(`WOS64 Fireball static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
