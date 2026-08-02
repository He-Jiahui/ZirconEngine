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
const sourceAbilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const renewSource = sourceAbilityBlock("renew", "mind_blast");
for (const needle of [
  "class: 'priest'", "learnLevel: 8", "cost: 30", "castTime: 0", "range: 30",
  "school: 'holy'", "targetType: 'friendly'", "total: 45, duration: 15, interval: 3",
  "rank: 2", "level: 14", "cost: 50", "total: 90, duration: 15, interval: 3",
  "rank: 3", "level: 20", "cost: 75", "total: 140, duration: 15, interval: 3",
]) {
  if (!renewSource.includes(needle)) throw new Error(`source Renew drifted: ${needle}`);
}
requireText(
  casting,
  /ability\.targetType === 'friendly'[\s\S]*?spendAbilityCost\(ctx, p, meta, res\);[\s\S]*?ctx\.runEffects\(p, meta, target, res\);[\s\S]*?return;/,
  "source friendly instant lifecycle drifted",
);
requireText(
  effects,
  /case 'hot':[\s\S]*?const hybridHeal = res\.effects\.some[\s\S]*?const hotSp = hybridHeal \? 0 : hotTickBonus\(p\.spellPower, eff\.duration, eff\.interval\);[\s\S]*?tickTimer: eff\.interval/,
  "source pure-HoT application snapshot drifted",
);

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'shadow_word_pain',[\s\S]*?'renew'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 93")) {
  throw new Error("M4 Renew projection scope is missing");
}
if (!zrGenerator.includes("document.entries.length === 93")) {
  throw new Error("M4 Renew Zr projection count is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const renew = m4.entries.find((entry) => entry.id === "renew");
if (!renew || renew.index !== 25 || renew.scenarios.length !== 0 ||
    renew.definition.targetType !== "friendly" || renew.definition.school !== "holy" ||
    renew.definition.effects?.[0]?.total !== 45) {
  throw new Error("M4 Renew source projection drifted");
}

const pureHot = read("scripts", "woc_game", "src", "combat", "pure_hot_profile_state.zr");
requireText(
  pureHot,
  /class PureHotProfile[\s\S]*?pub resolvePureHotProfile\([\s\S]*?pub pureHotProfileMatches\(/,
  "pure-HoT profile module is missing",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /offlineHotRanks[\s\S]*?offlineHotSnapshotPowers/, "WOS64 HoT snapshot columns are missing");
requireText(world, /writer\.u16\(<uint>state\.offlineHotTargetIds\.length, 1, 1\)[\s\S]*?offlineHotSnapshotPowers/, "WOS64 HoT snapshot tail is missing");
requireText(world, /schemaVersion != <uint>64[\s\S]*?schemaVersion >= <uint>64/, "WOS64 decoder migration is missing");
requireText(world, /renewAbilityCode\([\s\S]*?startOfflineRenewCast/, "Renew cast reducer is missing");
requireText(world, /renewGlobalCooldownSeconds[\s\S]*?0\.75/, "Renew must preserve the source GCD floor");
requireText(world, /applyOfflinePureHot[\s\S]*?targetId[\s\S]*?sourceId[\s\S]*?abilityCode/, "Renew must apply through the generic pure-HoT reducer");
requireText(world, /offlineHotTargetIds\[index\] == targetId[\s\S]*?offlineHotSourceIds\[index\] == sourceId[\s\S]*?offlineHotAbilityCodes\[index\] == abilityCode/, "Renew must replace only the same-source aura");
requireText(world, /pub renewCommandStateTest\(\): int[\s\S]*?renew/, "Renew state regression coverage is missing");

const main = read("scripts", "woc_game", "src", "main.zr");
if (!/\\"world_state\\":\\"WOS83\\"/.test(main)) {
  throw new Error("package state identity must advance to WOS64");
}
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
if (!protocol.includes('WORLD_STATE_FORMAT: &str = "WOS83"') ||
    !protocol.includes("WORLD_STATE_SCHEMA_VERSION: u16 = 83")) {
  throw new Error("native state identity must advance to WOS74");
}

process.stdout.write(`WOS85 Renew static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
