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
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const locomotion = source("src/sim/mob/locomotion.ts");
const sim = source("src/sim/sim.ts");

for (const needle of [
  "gouge: {",
  "id: 'gouge'",
  "learnLevel: 6",
  "cost: 45",
  "cooldown: 10",
  "school: 'physical'",
  "requiresTarget: true",
  "awardsCombo: 1",
  "{ type: 'directDamage', min: 8, max: 9 }",
  "{ type: 'incapacitate', duration: 4 }",
  "{ type: 'directDamage', min: 15, max: 17 }",
]) {
  if (!classes.includes(needle)) {
    throw new Error(`source Gouge definition drifted: ${needle}`);
  }
}
for (const needle of [
  "case 'directDamage': {",
  "let dmg = ctx.rng.range(eff.min, eff.max);",
  "const finalDamage = Math.round(dmg);",
  "case 'incapacitate': {",
  "id: `${ability.id}_incap`",
  "kind: 'incapacitate'",
  "breaksOnDamage: true",
  "if (ability.awardsCombo && !comboAwarded)",
]) {
  if (!dispatch.includes(needle)) {
    throw new Error(`source Gouge effect reducer drifted: ${needle}`);
  }
}
for (const needle of [
  "if (ctx.isStunned(mob)) {",
  "tickForcedTarget(mob);",
  "if (ctx.updateFearMovement(mob)) return;",
]) {
  if (!locomotion.includes(needle)) {
    throw new Error(`source Gouge incapacitate mob branch drifted: ${needle}`);
  }
}
requireText(
  sim,
  /this\.updateMob\(e\);[\s\S]*?updateAuras\(this\.ctx, e\);/,
  "source mob aura ageing order drifted",
);

const m4SourceGenerator = read("tools", "m4_ability_codegen.mjs");
const m4ZrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
const m4Contract = JSON.parse(read("contracts", "m4_abilities.json"));
const m4Effects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'gouge'/.test(m4SourceGenerator) ||
!m4SourceGenerator.includes("EXPECTED_ABILITY_COUNT = 78")) {
  throw new Error("M4 WOC-only Gouge projection scope is missing");
}
if (!m4ZrGenerator.includes("document.entries.length === 78")) {
  throw new Error("M4 Zr Gouge projection count is missing");
}
const gougeEntry = m4Contract.entries.find((entry) => entry.id === "gouge");
if (!gougeEntry || gougeEntry.index !== 22 || gougeEntry.scenarios.length !== 0 ||
    gougeEntry.definition.cost !== 45 || gougeEntry.definition.cooldown !== 10 ||
    gougeEntry.definition.effects[0].min !== 8 ||
    gougeEntry.definition.effects[0].max !== 9 ||
    gougeEntry.definition.effects[1].duration !== 4 ||
    gougeEntry.definition.ranks.length !== 1 ||
    gougeEntry.definition.ranks[0].effects[0].min !== 15 ||
    gougeEntry.definition.ranks[0].effects[0].max !== 17) {
  throw new Error("M4 Gouge retained source projection drifted");
}
requireText(
  m4Effects,
  /if \(index == 22\) \{[\s\S]*?return "directDamage";[\s\S]*?return "incapacitate";[\s\S]*?if \(field == "max"\) \{ return 9\.0; \}[\s\S]*?if \(field == "min"\) \{ return 8\.0; \}[\s\S]*?if \(field == "duration"\) \{ return 4\.0; \}/,
  "generated M4 Gouge effect projection is missing",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /gougeAbilityCode[\s\S]*?gougeTargetIndex[\s\S]*?m4AbilityCatalog\.metric\([\s\S]*?"range"[\s\S]*?startOfflineGougeCast/,
  "WOS60 Gouge catalog and target admission is missing",
);
requireText(
  world,
  /startOfflineGougeCast[\s\S]*?abilityCooldownExpiresAt[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?armorReductionFromArmor[\s\S]*?clearOfflineBreakableIncapacitateOnDamage[\s\S]*?applyOfflineGougeIncapacitate[\s\S]*?setAbilityCooldownExpiration/,
  "WOS60 Gouge damage, break, incap and cooldown reducer is incomplete",
);
requireText(
  world,
  /applyOfflineMotionAura[\s\S]*?entityMotionAuraOffsets[\s\S]*?applyOfflineGougeIncapacitate[\s\S]*?ccContract\.motionAuraKindCode\("incapacitate"\)[\s\S]*?clearOfflineBreakableIncapacitateOnDamage[\s\S]*?removeMotionAuraAt/,
  "WOS60 Gouge motion-aura persistence is missing",
);
requireText(
  world,
  /stepOfflineEastbrookMobMeleePursuit[\s\S]*?stepOfflineEastbrookForcedTarget[\s\S]*?motionAuraEntityIsStunned[\s\S]*?ageMotionAurasForEntity/,
  "WOS60 Gouge mob incapacitate tick ordering is missing",
);
const breakCalls = world.match(
  /clearOfflineBreakableIncapacitateOnDamage\(state, (?:targetIndex|playerIndex),/g,
)?.length ?? 0;
if (breakCalls < 7) {
  throw new Error("WOS60 retained positive-damage paths do not break Gouge");
}
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?gougeAbilityCode[\s\S]*?startOfflineGougeCast[\s\S]*?applySupportedCastCommand[\s\S]*?gougePayloadAbilityIsExact[\s\S]*?startOfflineGougeCast/,
  "WOS60 Gouge slot and typed command routes are missing",
);
requireText(
  world,
  /pub gougeCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepOfflineEastbrookMobMeleePursuit[\s\S]*?appendTypedCastTargetCommandForTest/,
  "WOS60 Gouge command, persistence and incapacitate coverage is missing",
);

process.stdout.write(`WOS60 Gouge static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
