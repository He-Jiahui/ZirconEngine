import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import fs from "node:fs";

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
const start = classes.indexOf("  swiftmend: {");
const end = classes.indexOf("  metamorphosis: {", start);
const swiftmend = classes.slice(start, end);
for (const needle of [
  "class: 'druid'", "learnLevel: 10", "cost: 55", "castTime: 0",
  "cooldown: 8", "range: 30", "school: 'nature'", "requiresTarget: true",
  "targetType: 'friendly'", "type: 'consumeAura'", "auraKind: 'hot'",
  "min: 105", "max: 125",
]) {
  if (!swiftmend.includes(needle)) throw new Error(`source Swiftmend drifted: ${needle}`);
}
requireText(
  casting,
  /ability\.targetType === 'friendly'[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?armAbilityCooldown\(p, ability\.id, res\.cooldown[\s\S]*?ctx\.runEffects\(p, meta, target, res\);[\s\S]*?return;[\s\S]*?const firesProjectile/,
  "source helpful spell lifecycle must resolve before projectile dispatch",
);
requireText(
  dispatch,
  /function consumeMatchingAura[\s\S]*?a\.kind !== 'dot' && a\.kind !== 'hot'[\s\S]*?matchesKind = eff\.auraKind !== undefined && a\.kind === eff\.auraKind/,
  "source consumeAura hot matching drifted",
);
requireText(
  dispatch,
  /case 'consumeAura':[\s\S]*?const consumed = target\.auras\[auraIdx\];[\s\S]*?target\.auras\.splice\(auraIdx, 1\)[\s\S]*?if \(eff\.heal\)[\s\S]*?ctx\.rng\.range\(eff\.heal\.min, eff\.heal\.max\)[\s\S]*?ctx\.applyHeal\(p, target, healAmount, ability\.name, ability\.id\)/,
  "source Swiftmend must consume before its direct-heal range and heal application",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!generator.includes("'swiftmend'") || !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79") ||
    !zrGenerator.includes("'auraKind'") || !zrGenerator.includes("effect.heal?.[field]")) {
  throw new Error("M4 Swiftmend projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (item) => item.id === "swiftmend",
);
if (!entry || entry.index !== 52 || entry.definition.class !== "druid" ||
    entry.definition.school !== "nature" || entry.definition.learnLevel !== 10 ||
    entry.definition.cost !== 55 || entry.definition.castTime !== 0 ||
    entry.definition.cooldown !== 8 || entry.definition.range !== 30 ||
    !entry.definition.requiresTarget || entry.definition.targetType !== "friendly" ||
    entry.definition.effects?.[0]?.type !== "consumeAura" ||
    entry.definition.effects[0].auraKind !== "hot" ||
    entry.definition.effects[0].heal?.min !== 105 || entry.definition.effects[0].heal?.max !== 125) {
  throw new Error("M4 Swiftmend projection drifted");
}
const effects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
requireText(effects, /index == 52[\s\S]*?return "consumeAura";/,
  "generated Swiftmend effect type is missing");
requireText(effects, /index == 52[\s\S]*?field == "max"\) \{ return 125\.0; \}[\s\S]*?field == "min"\) \{ return 105\.0; \}/,
  "generated Swiftmend heal metrics are missing");
requireText(effects, /index == 52[\s\S]*?field == "auraKind"\) \{ return "hot"; \}/,
  "generated Swiftmend aura-kind selector is missing");

const numeric = read("scripts", "woc_game", "src", "combat", "effect_numeric_dispatch_state.zr");
requireText(numeric, /consumeAuraMatchIndex[\s\S]*?effects\.text\(abilityIndex, rank, effectIndex, "auraKind"\)[\s\S]*?dispatchConsumeAura[\s\S]*?removeAuraAt[\s\S]*?dispatchHeal/,
  "numeric dispatch must consume the matching aura before Swiftmend healing");
requireText(numeric, /effectType == "consumeAura"[\s\S]*?dispatchConsumeAura/,
  "numeric dispatch must route consumeAura effects");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /swiftmendAbilityCode\([\s\S]*?swiftmendPayloadAbilityIsExact[\s\S]*?swiftmendProfileIsValid[\s\S]*?"auraKind"\) == "hot"/,
  "Swiftmend identity and source profile are missing");
requireText(world, /applyOfflineDirectHeal[\s\S]*?healState\.applyHeal/,
  "Swiftmend must reuse the authoritative direct-healing kernel");
requireText(world, /applyOfflineSwiftmendEffect[\s\S]*?removeOfflineRejuvenationHotAt[\s\S]*?applyOfflineDirectHeal/,
  "Swiftmend must consume the first hot before its range and critical healing draws");
requireText(world, /startOfflineSwiftmendCast[\s\S]*?setAbilityCooldownExpiration[\s\S]*?applyOfflineSwiftmendEffect/,
  "Swiftmend must bill, cooldown, and resolve in the helpful-spell path");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?swiftmendAbilityCode\(\)[\s\S]*?startOfflineSwiftmendCast[\s\S]*?applySupportedCastCommand[\s\S]*?swiftmendPayloadAbilityIsExact/,
  "Swiftmend action-slot and typed command routes are missing");
requireText(world, /pub swiftmendCommandStateTest\(\): int[\s\S]*?offlineHotTargetIds\.length != 1[\s\S]*?swiftmendBytes/,
  "Swiftmend regression must cover ordered hot consumption and typed commands");
requireText(world, /if \(swiftmendCommandStateTest\(\) != 1\) \{[\s\S]*?return -106;/,
  "world selfTest must execute Swiftmend");

process.stdout.write(`WOS112 Swiftmend static guards passed (${commit.slice(0, 15)})\n`);
