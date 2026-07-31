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
const auras = source("src/sim/combat/auras.ts");
const start = classes.indexOf("  siphon_life: {");
const end = classes.indexOf("  conflagrate: {", start);
const siphonLife = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 10", "cost: 45", "castTime: 0",
  "cooldown: 0", "range: 30", "school: 'shadow'", "requiresTarget: true",
  "type: 'dot'", "total: 60", "duration: 30", "interval: 3", "leechPct: 1",
]) {
  if (!siphonLife.includes(needle)) throw new Error(`source Siphon Life drifted: ${needle}`);
}
requireText(
  casting,
  /if \(target && firesProjectile\) \{[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?armAbilityCooldown\(p, ability\.id, res\.cooldown[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source spell projectile lifecycle drifted",
);
requireText(
  auras,
  /else if \(a\.kind === 'dot'\) \{[\s\S]*?ctx\.dealDamage\([\s\S]*?if \(a\.leechPct !== undefined\)[\s\S]*?Math\.round\(tickDamage \* a\.leechPct\)[\s\S]*?src\.hp \+= healed[\s\S]*?ctx\.healingThreat\(src, src, healed\)[\s\S]*?if \(e\.dead\) return/,
  "source Siphon Life tick damage, lethal and healing order drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!generator.includes("'siphon_life'") || !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79") || !zrGenerator.includes("'leechPct'")) {
  throw new Error("M4 Siphon Life projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (item) => item.id === "siphon_life",
);
if (!entry || entry.index !== 51 || entry.definition.class !== "warlock" ||
    entry.definition.school !== "shadow" || entry.definition.learnLevel !== 10 ||
    entry.definition.cost !== 45 || entry.definition.castTime !== 0 ||
    entry.definition.cooldown !== 0 || entry.definition.range !== 30 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "dot" ||
    entry.definition.effects[0].total !== 60 || entry.definition.effects[0].duration !== 30 ||
    entry.definition.effects[0].interval !== 3 || entry.definition.effects[0].leechPct !== 1) {
  throw new Error("M4 Siphon Life projection drifted");
}
const effects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
requireText(effects, /index == 51[\s\S]*?field == "duration"\) \{ return 30\.0; \}[\s\S]*?field == "interval"\) \{ return 3\.0; \}[\s\S]*?field == "leechPct"\) \{ return 1\.0; \}[\s\S]*?field == "total"\) \{ return 60\.0; \}/,
  "generated Siphon Life effect metrics are missing");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /siphonLifeAbilityCode\([\s\S]*?siphonLifePayloadAbilityIsExact[\s\S]*?pureDotAbilityIndex[\s\S]*?m4AbilityCatalog\.indexOf\("siphon_life"\)[\s\S]*?pureDotRankLevel[\s\S]*?return 10;/,
  "Siphon Life identity and rank mapping are missing");
requireText(world, /pureDotProjectileProfileIsValid[\s\S]*?abilityCode != siphonLifeAbilityCode\(\)[\s\S]*?"leechPct"\) == 1\.0/,
  "Siphon Life in-flight profile must validate its source leech fraction");
requireText(world, /startOfflineSiphonLifeCast[\s\S]*?startOfflinePureDotCast[\s\S]*?siphonLifeAbilityCode/,
  "Siphon Life must use the instant pure-DoT projectile path");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?curseOfAgonyAbilityCode\(\) \|\|[\s\S]*?siphonLifeAbilityCode\(\)[\s\S]*?pureDotProjectileProfileIsValid/,
  "Siphon Life in-flight state validation is missing");
requireText(world, /offlineDotStateIsValid[\s\S]*?abilityCode != siphonLifeAbilityCode\(\)[\s\S]*?"leechPct"\) == 1\.0/,
  "Siphon Life persisted DoT validation is missing");
requireText(world, /landOfflineSiphonLifeProjectile[\s\S]*?landOfflinePureDotProjectile[\s\S]*?siphonLifeAbilityCode/,
  "Siphon Life projectile landing must reuse pure-DoT resistance and snapshot logic");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?var abilityCode = [\s\S]*?settleOfflineEastbrookLethal[\s\S]*?abilityCode == siphonLifeAbilityCode\(\)[\s\S]*?"leechPct"[\s\S]*?applyOfflineDrainLifeHealingThreat/,
  "Siphon Life tick must heal after damage and lethal settlement");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?siphonLifeAbilityCode\(\)[\s\S]*?landOfflineSiphonLifeProjectile/,
  "Siphon Life projectile landing dispatch is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?siphonLifeAbilityCode\(\)[\s\S]*?startOfflineSiphonLifeCast[\s\S]*?applySupportedCastCommand[\s\S]*?siphonLifePayloadAbilityIsExact/,
  "Siphon Life action-slot and typed command routes are missing");
requireText(world, /pub siphonLifeCommandStateTest\(\): int[\s\S]*?entityHp\[0\] != sourceHpBeforeTick \+ profile\.damage[\s\S]*?lethal\.entityHp\[0\] != 86[\s\S]*?siphonBytes/,
  "Siphon Life regression must cover normal, lethal and typed paths");
requireText(world, /if \(siphonLifeCommandStateTest\(\) != 1\) \{[\s\S]*?return -105;/,
  "world selfTest must execute Siphon Life");

process.stdout.write(`WOS111 Siphon Life static guards passed (${commit.slice(0, 15)})\n`);
