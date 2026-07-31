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
requireText(
  classes,
  /moonfire:\s*\{[\s\S]*?learnLevel: 4,[\s\S]*?cost: 25,[\s\S]*?castTime: 0,[\s\S]*?range: 30,[\s\S]*?school: 'arcane',[\s\S]*?directDamage', min: 9, max: 12[\s\S]*?dot', total: 12, duration: 9, interval: 3[\s\S]*?rank: 2,[\s\S]*?level: 10,[\s\S]*?total: 24, duration: 12, interval: 3[\s\S]*?rank: 3,[\s\S]*?level: 16,[\s\S]*?total: 40, duration: 12, interval: 3/,
  "source Moonfire definition drifted",
);
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?scheduleProjectile\(ctx, p, target, \(src, tgt\) => \{[\s\S]*?isSpellResisted\(ctx\.rng, src\.level, tgt\.level, src\.hitBonus\)[\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\);/,
  "source Moonfire projectile and resist ordering drifted",
);
requireText(
  effects,
  /case 'dot':[\s\S]*?const hybrid = res\.effects\.some[\s\S]*?const dotBase = Math\.max\(1, Math\.round\(dotTotal \/ \(eff\.duration \/ eff\.interval\)\)\)[\s\S]*?tickTimer: eff\.interval/,
  "source Moonfire periodic profile capture drifted",
);

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const moonfire = m4.entries.find((entry) => entry.id === "moonfire");
if (!moonfire || moonfire.index !== 16 || moonfire.definition.class !== "druid" ||
    moonfire.definition.learnLevel !== 4 || moonfire.definition.cost !== 25 ||
    moonfire.definition.castTime !== 0 || moonfire.definition.range !== 30 ||
    moonfire.definition.school !== "arcane" ||
    moonfire.definition.effects?.[0]?.type !== "directDamage" ||
    moonfire.definition.effects?.[0]?.min !== 9 || moonfire.definition.effects?.[0]?.max !== 12 ||
    moonfire.definition.effects?.[1]?.type !== "dot" ||
    moonfire.definition.effects?.[1]?.total !== 12 || moonfire.definition.effects?.[1]?.duration !== 9 ||
    moonfire.definition.effects?.[1]?.interval !== 3) {
  throw new Error("M4 Moonfire projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /moonfireAbilityCode\([\s\S]*?abilityCode\("moonfire"\)[\s\S]*?m4AbilityCatalog\.indexOf\("moonfire"\)/, "Moonfire identity is missing");
requireText(world, /startOfflineMoonfireCast[\s\S]*?catalogAdmission[\s\S]*?cast\.gcdRemaining[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_ARCANE/, "Moonfire instant start must charge, arm GCD and queue an arcane projectile");
requireText(world, /landOfflineMoonfireProjectile[\s\S]*?spellResist\.resolve[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?applyOfflineMoonfireDot/, "Moonfire landing reducer is missing");
requireText(world, /offlineDotStateIsValid[\s\S]*?moonfireAbilityCode\(\)/, "WOS57 periodic rows must validate Moonfire profiles");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?moonfireAbilityCode\(\)[\s\S]*?m4AbilityCatalog\.metric/, "Periodic DoT threat must follow its retained ability identity");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?moonfireAbilityCode\(\)[\s\S]*?landOfflineMoonfireProjectile[\s\S]*?landOfflineRangedProjectile/, "Moonfire projectile dispatch is missing");
requireText(world, /writer\.u16\(<uint>67, 1, 1\)[\s\S]*?offlineDotAbilityCodes/, "Moonfire must preserve the WOS57 periodic tail before later schema details");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?moonfireAbilityCode\(\)[\s\S]*?startOfflineMoonfireCast[\s\S]*?applySupportedCastCommand[\s\S]*?moonfirePayloadAbilityIsExact/, "Moonfire slot and typed routes are missing");
requireText(world, /pub moonfireCommandStateTest\(\): int[\s\S]*?offlineDotTargetIds[\s\S]*?stepOfflineEastbrookDots/, "Moonfire state regression coverage is missing");
requireText(world, /if \(moonfireCommandStateTest\(\) != 1\) \{\s*return -61;\s*\}/, "Moonfire self-test route is missing");

process.stdout.write(`WOS65 Moonfire static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
