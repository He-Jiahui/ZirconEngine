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
const spellResistSource = source("src/sim/combat/spell_resist.ts");
requireText(
  classes,
  /frostbolt:\s*\{[\s\S]*?learnLevel: 4,[\s\S]*?cost: 25,[\s\S]*?castTime: 1\.5,[\s\S]*?cooldown: 0,[\s\S]*?range: 30,[\s\S]*?school: 'frost',[\s\S]*?effects: \[[\s\S]*?directDamage', min: 18, max: 20[\s\S]*?slow', mult: 0\.6, duration: 5/,
  "source Frostbolt definition drifted",
);
requireText(
  casting,
  /if \(p\.castRemaining <= CAST_COMPLETE_EPS\) \{[\s\S]*?const castId = p\.castingAbility;[\s\S]*?const res = ctx\.resolvedAbility\(castId, p\.id\);[\s\S]*?applyAbility\(ctx, p, meta, resolved\);[\s\S]*?p\.castTargetId = null;/,
  "source timed-cast completion and target-clear ordering drifted",
);
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?scheduleProjectile\(ctx, p, target, \(src, tgt\) => \{[\s\S]*?isSpellResisted\(ctx\.rng, src\.level, tgt\.level, src\.hitBonus\)[\s\S]*?ctx\.enterCombat\(src, tgt\);[\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\);/,
  "source spell-projectile launch, resist or landing ordering drifted",
);
requireText(
  spellResistSource,
  /isSpellResisted[\s\S]*?!rng\.chance\(effectiveSpellHit\(casterLevel, targetLevel, hitBonus\)\)/,
  "source spell-resist one-draw gate drifted",
);
requireText(
  effects,
  /case 'directDamage':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?ctx\.rng\.chance[\s\S]*?ctx\.dealDamage\([\s\S]*?case 'slow':[\s\S]*?kind: 'slow',[\s\S]*?value: eff\.mult,[\s\S]*?duration: eff\.duration/,
  "source Frostbolt direct-damage or slow dispatch drifted",
);

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const frostbolt = m4.entries.find((entry) => entry.id === "frostbolt");
if (!frostbolt || frostbolt.index !== 2 || frostbolt.definition.class !== "mage" ||
    frostbolt.definition.learnLevel !== 4 || frostbolt.definition.cost !== 25 ||
    frostbolt.definition.castTime !== 1.5 || frostbolt.definition.cooldown !== 0 ||
    frostbolt.definition.range !== 30 || frostbolt.definition.school !== "frost" ||
    frostbolt.definition.effects?.[0]?.type !== "directDamage" ||
    frostbolt.definition.effects?.[0]?.min !== 18 || frostbolt.definition.effects?.[0]?.max !== 20 ||
    frostbolt.definition.effects?.[1]?.type !== "slow" ||
    frostbolt.definition.effects?.[1]?.mult !== 0.6 || frostbolt.definition.effects?.[1]?.duration !== 5) {
  throw new Error("M4 Frostbolt projection drifted");
}

const catalog = read("scripts", "woc_game", "src", "generated", "m4_ability_catalog.zr");
const generatedEffects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
requireText(catalog, /if \(id == "frostbolt"\) \{ return 2; \}/, "M4 Frostbolt index is missing");
requireText(generatedEffects, /pub typeAt[\s\S]*?if \(index == 2\)[\s\S]*?return "directDamage";[\s\S]*?return "slow";/, "M4 Frostbolt effect types are missing");
requireText(generatedEffects, /pub metric[\s\S]*?if \(index == 2\)[\s\S]*?if \(field == "max"\) \{ return 20\.0; \}[\s\S]*?if \(field == "min"\) \{ return 18\.0; \}[\s\S]*?if \(field == "duration"\) \{ return 5\.0; \}[\s\S]*?if \(field == "mult"\) \{ return 0\.6; \}/, "M4 Frostbolt effect metrics are missing");

const ccGenerator = read("tools", "cc_contract_codegen.mjs");
const cc = read("scripts", "woc_game", "src", "generated", "cc_contract.zr");
const motion = read("scripts", "woc_game", "src", "world", "motion_aura_state.zr");
requireText(ccGenerator, /slow: 7/, "slow must have a stable persisted motion-aura code");
requireText(cc, /if \(kind == "slow"\) \{ return <uint>7; \}/, "generated slow motion-aura code is missing");
requireText(cc, /isMotionSlowKindCode\(code: uint\).*?<uint>7/, "generated slow motion-aura predicate is missing");
requireText(motion, /pub movementMultiplier[\s\S]*?isMotionSlowKindCode/, "motion aura state does not derive slow movement");

const timedSpell = read("scripts", "woc_game", "src", "combat", "timed_spell_state.zr");
requireText(
  timedSpell,
  /pub resolveTimedSpellHit[\s\S]*?takeRandomUnit[\s\S]*?directHitBonus[\s\S]*?takeRandomUnit[\s\S]*?roundPositive/,
  "timed spell kernel must consume range then spell-crit RNG and apply spell scaling",
);
requireText(timedSpell, /pub contractTest\(\): int[\s\S]*?resolveTimedSpellHit/, "timed spell kernel lacks regression coverage");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /var timedSpell = %import\("combat\/timed_spell_state"\);/, "WorldState must use the timed-spell kernel");
requireText(world, /var spellResist = %import\("combat\/spell_resist_state"\);/, "WorldState must use the spell-resist kernel");
requireText(world, /frostboltAbilityCode\([\s\S]*?abilityCode\("frostbolt"\)[\s\S]*?m4AbilityCatalog\.indexOf\("frostbolt"\)/, "Frostbolt catalog identity is missing");
requireText(world, /frostboltPayloadAbilityIsExact[\s\S]*?abilityLength == <uint>9/, "Frostbolt typed payload admission is missing");
requireText(
  world,
  /startOfflineFrostboltCast[\s\S]*?catalogAdmission[\s\S]*?m4AbilityEffects\.typeAt\(abilityIndex, rank, 0\) != "directDamage"[\s\S]*?m4AbilityEffects\.typeAt\(abilityIndex, rank, 1\) != "slow"[\s\S]*?cast\.armTimed[\s\S]*?state\.entityCastTargetIds/,
  "Frostbolt start reducer must admit, validate effects, arm and lock its target",
);
requireText(
  world,
  /mageSpellHasteMultiplier[\s\S]*?entitySpellHaste[\s\S]*?frostboltSpellHasteMultiplier[\s\S]*?mageSpellHasteMultiplier[\s\S]*?frostboltCastSeconds[\s\S]*?frostboltGlobalCooldownSeconds[\s\S]*?gcd < 1\.0/,
  "Frostbolt must shorten its cast and GCD from retained spell haste with the source GCD floor",
);
requireText(
  world,
  /frostboltCompletionTargetIndex[\s\S]*?range \+ 2\.0[\s\S]*?completeOfflineFrostboltCast[\s\S]*?frostboltCompletionTargetIndex[\s\S]*?entityResources\[casterIndex\][\s\S]*?appendOfflineAbilityProjectile[\s\S]*?projectileTravel\.projectileSpeed\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FROST[\s\S]*?<uint>rank[\s\S]*?"castTime"/,
  "Frostbolt completion must charge then queue its source projectile profile",
);
const frostboltCompletionStart = world.indexOf("completeOfflineFrostboltCast");
const frostboltCompletionEnd = world.indexOf("fearAbilityCode");
if (frostboltCompletionStart < 0 || frostboltCompletionEnd <= frostboltCompletionStart ||
    /nextAuthoritativeRandomUnit|resolveTimedSpellHit|applyOfflineMotionAura/.test(
      world.slice(frostboltCompletionStart, frostboltCompletionEnd),
    )) {
  throw new Error("Frostbolt cast completion must not resolve projectile RNG or effects");
}
requireText(
  world,
  /landOfflineFrostboltProjectile[\s\S]*?SpellResistState[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?spellResist\.resolve[\s\S]*?TimedSpellResult[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?timedSpell\.resolveTimedSpellHit[\s\S]*?applyOfflineMotionAura[\s\S]*?motionAuraKindCode\("slow"\)[\s\S]*?settleOfflineEastbrookLethal/,
  "Frostbolt landing must resolve resist, direct spell damage and slow in source order",
);
requireText(
  world,
  /stepOfflineEastbrookProjectiles[\s\S]*?offlineProjectileAbilityCodes\[projectileIndex\][\s\S]*?frostboltAbilityCode\(\)[\s\S]*?landOfflineFrostboltProjectile[\s\S]*?landOfflineRangedProjectile/,
  "Frostbolt projectile must dispatch before the generic ranged landing path",
);
requireText(
  world,
  /writer\.u16\(<uint>67, 1, 1\)[\s\S]*?offlineProjectileAbilityCodes[\s\S]*?offlineProjectileRanks[\s\S]*?offlineProjectileCastTimes[\s\S]*?schemaVersion != <uint>56[\s\S]*?schemaVersion >= <uint>56[\s\S]*?reader\.u16[\s\S]*?offlineProjectileAbilityCodes\.add\(<uint>0\)/,
  "WOS56 Frostbolt projectile closure codec or legacy migration is missing",
);
requireText(
  world,
  /stepRetainedCasting[\s\S]*?completedAbility == frostboltAbilityCode\(\)[\s\S]*?completeOfflineFrostboltCast[\s\S]*?entityCastTargetIds\[index\] = <uint>0/,
  "Frostbolt must resolve before its locked target is cleared",
);
requireText(
  world,
  /stepOfflineEastbrookMobMeleePursuit[\s\S]*?motionAuras\.movementMultiplier[\s\S]*?initializeMobMeleePursuit/,
  "Frostbolt slow must change retained mob pursuit speed",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?frostboltAbilityCode\(\)[\s\S]*?startOfflineFrostboltCast[\s\S]*?applySupportedCastCommand[\s\S]*?frostboltPayloadAbilityIsExact/,
  "Frostbolt slot and typed routes are missing",
);
requireText(world, /pub frostboltCommandStateTest\(\): int[\s\S]*?offlineProjectileAbilityCodes[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepOfflineEastbrookProjectiles[\s\S]*?motionAuraKindCode\("slow"\)/, "Frostbolt projectile state regression coverage is missing");
requireText(world, /if \(frostboltCommandStateTest\(\) != 1\) \{\s*return -59;\s*\}/, "Frostbolt self-test route is missing");

process.stdout.write(`WOS63 Frostbolt static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
