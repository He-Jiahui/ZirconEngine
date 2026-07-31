import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const commit = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = path.resolve(root, "..", "..", "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const source = (file) => execFileSync(
  "git", ["-C", sourceRoot, "show", `${commit}:${file}`], { encoding: "utf8" },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const classes = source("src/sim/content/classes.ts");
const casting = source("src/sim/combat/casting_lifecycle.ts");
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const start = classes.indexOf("  conflagrate: {");
const end = classes.indexOf("  moonkin_form: {", start);
const conflagrate = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 10", "cost: 55", "castTime: 0",
  "cooldown: 6", "range: 30", "school: 'fire'", "requiresTarget: true",
  "type: 'consumeAura'", "auraIds: ['immolate']", "deal: { min: 54, max: 64 }",
]) {
  if (!conflagrate.includes(needle)) throw new Error(`source Conflagrate drifted: ${needle}`);
}
requireText(
  casting,
  /if \(target && firesProjectile\) \{[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?armAbilityCooldown\(p, ability\.id, res\.cooldown[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source projectile resource, cooldown and resist order drifted",
);
requireText(
  dispatch,
  /function consumeMatchingAura\([\s\S]*?a\.kind !== 'dot' && a\.kind !== 'hot'[\s\S]*?eff\.auraIds\?\.includes\(a\.id\)[\s\S]*?a\.sourceId === caster\.id[\s\S]*?case 'consumeAura':[\s\S]*?target\.auras\.splice\(auraIdx, 1\)[\s\S]*?ctx\.rng\.range\(eff\.deal\.min, eff\.deal\.max\)[\s\S]*?ctx\.rng\.chance/,
  "source same-caster consumeAura ordering drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/rain_of_fire',[\s\S]*?'conflagrate'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79") ||
    !zrGenerator.includes("consumeAuraIdAt") || !zrGenerator.includes("effect.deal")) {
  throw new Error("M4 Conflagrate projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (item) => item.id === "conflagrate",
);
if (!entry || entry.index !== 50 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "fire" || entry.definition.learnLevel !== 10 ||
    entry.definition.cost !== 55 || entry.definition.castTime !== 0 ||
    entry.definition.cooldown !== 6 || entry.definition.range !== 30 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "consumeAura" ||
    entry.definition.effects[0].auraIds?.[0] !== "immolate" ||
    entry.definition.effects[0].deal?.min !== 54 || entry.definition.effects[0].deal?.max !== 64) {
  throw new Error("M4 Conflagrate projection drifted");
}
const effects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
requireText(effects, /pub consumeAuraIdAt[\s\S]*?index == 50[\s\S]*?slot == 0\) \{ return "immolate"; \}/,
  "generated consumeAura id projection is missing");
requireText(effects, /index == 50[\s\S]*?field == "max"\) \{ return 64\.0; \}[\s\S]*?field == "min"\) \{ return 54\.0; \}/,
  "generated consumeAura deal metrics are missing");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /conflagrateAbilityCode\([\s\S]*?conflagratePayloadAbilityIsExact[\s\S]*?conflagrateProfileIsValid[\s\S]*?conflagrateProjectileProfileIsValid/,
  "Conflagrate identity and generated profile are missing");
requireText(world, /startOfflineConflagrateCast[\s\S]*?abilityCooldownExpiresAt[\s\S]*?gcdRemaining = conflagrateGlobalCooldownSeconds[\s\S]*?entityResources[\s\S]*?setAbilityCooldownExpiration[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/,
  "Conflagrate must arm its instant GCD/cooldown and launch the Fire projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?conflagrateAbilityCode\([\s\S]*?conflagrateProjectileProfileIsValid/,
  "Conflagrate in-flight snapshot validation is missing");
requireText(world, /landOfflineConflagrateProjectile[\s\S]*?spellResist\.resolve[\s\S]*?offlineDotTargetIds[\s\S]*?offlineDotSourceIds[\s\S]*?removeOfflineDotAt[\s\S]*?if \(!consumed\)[\s\S]*?timedSpell\.resolveTimedSpellHit/,
  "Conflagrate must resist first, consume its own Immolate, then resolve direct damage");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?conflagrateAbilityCode\(\)[\s\S]*?landOfflineConflagrateProjectile/,
  "Conflagrate projectile landing dispatch is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?conflagrateAbilityCode\(\)[\s\S]*?startOfflineConflagrateCast[\s\S]*?applySupportedCastCommand[\s\S]*?conflagratePayloadAbilityIsExact/,
  "Conflagrate command routes are missing");
requireText(world, /pub conflagrateCommandStateTest\(\): int[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?rngDraws != <uint>3[\s\S]*?rngDraws != <uint>4[\s\S]*?offlineDotSourceIds\[0\] == <uint>777/,
  "Conflagrate regression must cover own, missing and foreign Immolate rows");
requireText(world, /if \(conflagrateCommandStateTest\(\) != 1\) \{[\s\S]*?return -104;/,
  "world selfTest must execute Conflagrate");

process.stdout.write(`WOS110 Conflagrate static guards passed (${commit.slice(0, 15)})\n`);
