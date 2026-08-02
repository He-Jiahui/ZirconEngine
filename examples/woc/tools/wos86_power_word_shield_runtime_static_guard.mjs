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
const effects = source("src/sim/combat/effect_dispatch.ts");
const stacking = source("src/sim/combat/aura_stacking.ts");
const damage = source("src/sim/combat/damage.ts");
const sim = source("src/sim/sim.ts");
const abilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const shieldSource = abilityBlock("power_word_shield", "renew");
for (const needle of [
  "class: 'priest'", "learnLevel: 6", "cost: 45", "castTime: 0", "cooldown: 6",
  "range: 30", "school: 'holy'", "targetType: 'friendly'",
  "type: 'absorb', amount: 48, duration: 30", "rank: 2", "level: 12", "cost: 70",
  "type: 'absorb', amount: 90, duration: 30", "rank: 3", "level: 18", "cost: 100",
  "type: 'absorb', amount: 145, duration: 30",
]) {
  if (!shieldSource.includes(needle)) {
    throw new Error(`source Power Word Shield drifted: ${needle}`);
  }
}
requireText(
  effects,
  /case 'absorb':[\s\S]*?ctx\.applyAura\(shieldTarget,[\s\S]*?kind: 'absorb',[\s\S]*?value: eff\.amount,[\s\S]*?sourceId: p\.id/,
  "source absorb dispatch drifted",
);
requireText(
  stacking,
  /existing\.id !== aura\.id[\s\S]*?existing\.sourceId === aura\.sourceId/,
  "source same-source aura replacement drifted",
);
requireText(
  damage,
  /for \(let i = target\.auras\.length - 1; i >= 0 && amount > 0; i--\)[\s\S]*?a\.kind !== 'absorb'[\s\S]*?const soaked = Math\.min\(a\.value, amount\)[\s\S]*?target\.auras\.splice\(i, 1\)/,
  "source reverse-order absorb consumption drifted",
);
requireText(
  sim,
  /updateTimers\(p\);[\s\S]*?updateAuras\(this\.ctx, p\);[\s\S]*?this\.updateMob\(e\);/,
  "source player aura phase must precede mob damage",
);

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'renew',[\s\S]*?'power_word_shield'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 83")) {
  throw new Error("M4 Power Word Shield projection scope is missing");
}
if (!zrGenerator.includes("document.entries.length === 83")) {
  throw new Error("M4 Power Word Shield Zr projection count is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const shield = m4.entries.find((entry) => entry.id === "power_word_shield");
if (!shield || shield.index !== 26 || shield.scenarios.length !== 0 ||
    shield.definition.cooldown !== 6 || shield.definition.targetType !== "friendly" ||
    shield.definition.effects?.[0]?.type !== "absorb" ||
    shield.definition.effects?.[0]?.amount !== 48 ||
    shield.definition.effects?.[0]?.duration !== 30) {
  throw new Error("M4 Power Word Shield source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /offlineAbsorbTargetIds[\s\S]*?offlineAbsorbSourceIds[\s\S]*?offlineAbsorbAbilityCodes[\s\S]*?offlineAbsorbRanks[\s\S]*?offlineAbsorbAmounts[\s\S]*?offlineAbsorbRemaining/, "WOS66 absorb rows are missing");
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)[\s\S]*?offlineAbsorbTargetIds[\s\S]*?offlineAbsorbRemaining/, "current schema absorb tail is missing");
requireText(world, /schemaVersion != <uint>66[\s\S]*?schemaVersion >= <uint>66/, "WOS66 decoder migration is missing");
requireText(world, /powerWordShieldAbilityCode\([\s\S]*?startOfflinePowerWordShieldCast/, "Power Word Shield cast reducer is missing");
requireText(world, /abilityCooldownExpiresAt\(state, casterIndex, abilityCode\) > state\.timeMicros[\s\S]*?setAbilityCooldownExpiration\(state, casterIndex, abilityCode, state\.timeMicros \+ <uint>6000000\)/, "Power Word Shield must honor its source cooldown");
requireText(world, /applyOfflinePowerWordShield[\s\S]*?offlineAbsorbTargetIds\[index\] == targetId[\s\S]*?offlineAbsorbSourceIds\[index\] == sourceId[\s\S]*?offlineAbsorbAbilityCodes\[index\] == abilityCode[\s\S]*?removeOfflineAbsorbAt[\s\S]*?offlineAbsorbAmounts\.add/, "Power Word Shield must replace only the same-source row then append");
requireText(world, /applyOfflineAbsorbDamage[\s\S]*?index = state\.offlineAbsorbTargetIds\.length - 1[\s\S]*?soaked[\s\S]*?removeOfflineAbsorbAt/, "Power Word Shield damage must consume rows in reverse insertion order");
requireText(world, /resolveOfflineEastbrookMobSwingRequests[\s\S]*?applyOfflineAbsorbDamage[\s\S]*?clearOfflineBreakableIncapacitateOnDamage[\s\S]*?applyOfflineDamageCastPushback/, "Eastbrook mob swings must apply only post-absorb damage follow-ups");
requireText(world, /ageOfflineAbsorbs\(state\);[\s\S]*?stepOfflineEastbrookMobMeleePursuit\(state\);/, "absorb expiry must run before retained mob damage");
requireText(world, /pub powerWordShieldCommandStateTest\(\): int[\s\S]*?power_word_shield[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?applyOfflineAbsorbDamage/, "Power Word Shield state regression coverage is missing");

const main = read("scripts", "woc_game", "src", "main.zr");
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
const packageState = main.match(/\\"world_state\\":\\"WOS(\d+)\\"/);
const nativeFormat = protocol.match(/WORLD_STATE_FORMAT: &str = "WOS(\d+)"/);
const nativeVersion = protocol.match(/WORLD_STATE_SCHEMA_VERSION: u16 = (\d+)/);
if (!packageState || !nativeFormat || !nativeVersion ||
    packageState[1] !== nativeFormat[1] || nativeFormat[1] !== nativeVersion[1]) {
  throw new Error("package and native world-state identities must agree");
}

process.stdout.write(`WOS86 Power Word Shield static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
