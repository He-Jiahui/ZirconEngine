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
const online = source("src/net/online.ts");
const start = classes.indexOf("  rain_of_fire: {");
const end = classes.indexOf("  spell_lock: {", start);
const rain = classes.slice(start, end);
for (const needle of [
  "class: 'warlock'", "learnLevel: 18", "cost: 85", "castTime: 0",
  "cooldown: 10", "range: 30", "school: 'fire'", "requiresTarget: false",
  "targetMode: 'position'", "channel: { duration: 4, ticks: 4 }",
  "type: 'aoeDamage', min: 14, max: 18, radius: 7",
]) {
  if (!rain.includes(needle)) throw new Error(`source Rain of Fire drifted: ${needle}`);
}
requireText(
  online,
  /castAbilityAt\(abilityId: string, aim: \{ x: number; z: number \}\)[\s\S]*?cmd: 'castAt', ability: abilityId, x: aim\.x, z: aim\.z/,
  "source castAt command ABI drifted",
);
requireText(
  casting,
  /Ground-targeted channels \(Rain of Fire[\s\S]*?targetMode === 'position'[\s\S]*?channelTickBonus[\s\S]*?rng\.range\(eff\.min, eff\.max\)[\s\S]*?Math\.round\(dmg\)/,
  "source positioned-channel pulse order drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/demon_skin',[\s\S]*?'rain_of_fire'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("'targetMode'") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Rain of Fire scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (item) => item.id === "rain_of_fire",
);
if (!entry || entry.index !== 49 || entry.definition.class !== "warlock" ||
    entry.definition.targetMode !== "position" || entry.definition.cost !== 85 ||
    entry.definition.cooldown !== 10 || entry.definition.channel?.duration !== 4 ||
    entry.definition.channel?.ticks !== 4 || entry.definition.effects?.[0]?.type !== "aoeDamage" ||
    entry.definition.effects[0].min !== 14 || entry.definition.effects[0].max !== 18 ||
    entry.definition.effects[0].radius !== 7) {
  throw new Error("M4 Rain of Fire projection drifted");
}

const binary = read("scripts", "woc_game", "src", "protocol", "binary.zr");
requireText(binary, /pub readF64LeAt\([\s\S]*?reader\.readF64\(1, 1, 2, 4, 8\)/,
  "castAt must use the canonical finite f64 decoder");
const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /rainOfFireAbilityCode\([\s\S]*?rainOfFirePayloadAbilityIsExact[\s\S]*?rainOfFireProfileIsValid/,
  "Rain of Fire identity/profile is missing");
requireText(world, /startOfflineRainOfFireCast[\s\S]*?math\.sqrt[\s\S]*?range \/ distance[\s\S]*?armChannel[\s\S]*?setAbilityCooldownExpiration[\s\S]*?entityCastAimPresent/,
  "Rain of Fire must clamp and snapshot the source aim before the channel starts");
requireText(world, /launchOfflineRainOfFireChannelTick[\s\S]*?channelTickBonus[\s\S]*?resolveOfflineGroundAoEPulse/,
  "Rain of Fire ticks must use channel spell power and the retained AoE projection");
requireText(world, /applyCommands[\s\S]*?castAtCommand[\s\S]*?applySupportedCastAtCommand[\s\S]*?readF64LeAt[\s\S]*?startOfflineRainOfFireCast/,
  "Rain of Fire castAt reducer is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?rainOfFireAbilityCode\(\)[\s\S]*?applySupportedCastCommand[\s\S]*?rainOfFirePayloadAbilityIsExact/,
  "Rain of Fire must preserve source fallback-to-feet routes");
requireText(world, /stepRetainedCasting[\s\S]*?rainOfFireAbilityCode\(\)[\s\S]*?launchOfflineRainOfFireChannelTick[\s\S]*?clearRainOfFireCastAim/,
  "Rain of Fire channel ticks and aim cleanup are missing");
requireText(world, /pub rainOfFireCommandStateTest\(\): int[\s\S]*?appendTypedCastAtCommandForTest[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?entityChannelTicksLeft[\s\S]*?rngDraws == <uint>4/,
  "Rain of Fire regression coverage is missing");
requireText(world, /if \(rainOfFireCommandStateTest\(\) != 1\) \{[\s\S]*?return -103;/,
  "world selfTest must execute Rain of Fire");

process.stdout.write(`WOS109 Rain of Fire static guards passed (${commit.slice(0, 15)})\n`);
