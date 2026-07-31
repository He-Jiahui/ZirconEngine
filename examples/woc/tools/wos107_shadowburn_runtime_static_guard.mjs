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
const start = classes.indexOf("  shadowburn: {");
const end = classes.indexOf("  summon_imp: {", start);
const shadowburn = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 14", "cost: 70", "castTime: 0",
  "cooldown: 15", "range: 20", "school: 'shadow'", "requiresTarget: true",
  "type: 'directDamage', min: 56, max: 66",
]) {
  if (!shadowburn.includes(needle)) throw new Error(`source Shadowburn drifted: ${needle}`);
}
requireText(
  casting,
  /if \(!ability\.offGcd\) p\.gcdRemaining = Math\.max\(p\.gcdRemaining, gcd\);[\s\S]*?applyAbility\(ctx, p, meta, instantResolved, castTargetId\)/,
  "source instant GCD ordering drifted",
);
requireText(
  casting,
  /if \(target && firesProjectile\) \{[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?armAbilityCooldown\(p, ability\.id, res\.cooldown[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source instant projectile, cooldown and resist ordering drifted",
);
requireText(
  dispatch,
  /case 'directDamage':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?directHitBonus\([\s\S]*?ctx\.rng\.chance[\s\S]*?Math\.round\(dmg\)/,
  "source direct spell damage ordering drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/searing_pain',[\s\S]*?'shadowburn'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Shadowburn projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "shadowburn");
if (!entry || entry.index !== 47 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "shadow" || entry.definition.learnLevel !== 14 ||
    entry.definition.cost !== 70 || entry.definition.castTime !== 0 ||
    entry.definition.cooldown !== 15 || !entry.definition.requiresTarget ||
    entry.definition.effects?.[0]?.type !== "directDamage" ||
    entry.definition.effects[0].min !== 56 || entry.definition.effects[0].max !== 66 ||
    (entry.definition.ranks?.length ?? 0) !== 0) {
  throw new Error("M4 Shadowburn projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /shadowburnAbilityCode\([\s\S]*?shadowburnPayloadAbilityIsExact[\s\S]*?shadowburnProjectileProfileIsValid/, "Shadowburn identity and snapshot profile are missing");
requireText(world, /startOfflineShadowburnCast[\s\S]*?abilityCooldownExpiresAt[\s\S]*?gcdRemaining = shadowburnGlobalCooldownSeconds[\s\S]*?entityResources[\s\S]*?setAbilityCooldownExpiration[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW/, "Shadowburn instant admission must arm GCD/cooldown and queue a Shadow projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?shadowburnAbilityCode\([\s\S]*?shadowburnProjectileProfileIsValid/, "Shadowburn in-flight state validation is missing");
requireText(world, /landOfflineShadowburnProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "shadow"[\s\S]*?timedSpell\.resolveTimedSpellHit/, "Shadowburn landing must resolve one resist followed by direct Shadow damage");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?shadowburnAbilityCode\(\)[\s\S]*?landOfflineShadowburnProjectile/, "Shadowburn projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?shadowburnAbilityCode\(\)[\s\S]*?startOfflineShadowburnCast[\s\S]*?applySupportedCastCommand[\s\S]*?shadowburnPayloadAbilityIsExact/, "Shadowburn command routes are missing");
requireText(world, /pub shadowburnCommandStateTest\(\): int[\s\S]*?shadowburn[\s\S]*?abilityCooldownExpiresAt[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/, "Shadowburn state regression coverage is missing");

process.stdout.write(`WOS107 Shadowburn static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
