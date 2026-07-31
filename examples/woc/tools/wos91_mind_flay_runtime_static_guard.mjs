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
const abilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const mindFlaySource = abilityBlock("mind_flay", "flash_heal");
for (const needle of [
  "class: 'priest'", "learnLevel: 14", "cost: 45", "castTime: 0",
  "channel: { duration: 3, ticks: 3 }", "cooldown: 0", "range: 20",
  "school: 'shadow'", "type: 'drainTick', min: 12, max: 12, healFrac: 0",
]) {
  if (!mindFlaySource.includes(needle)) throw new Error(`source Mind Flay drifted: ${needle}`);
}
requireText(
  casting,
  /function applyChannelTick[\s\S]*?scheduleProjectile\(ctx, p, target[\s\S]*?eff\.type === 'drainTick'[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\) \+ channelSp[\s\S]*?Math\.round\(dmg \* eff\.healFrac\)/,
  "source single-target drain channel ordering drifted",
);

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'flash_heal',[\s\S]*?'mind_flay'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Mind Flay projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const mindFlay = m4.entries.find((entry) => entry.id === "mind_flay");
if (!mindFlay || mindFlay.index !== 31 || mindFlay.scenarios.length !== 0 ||
    mindFlay.definition.school !== "shadow" || mindFlay.definition.channel?.duration !== 3 ||
    mindFlay.definition.channel?.ticks !== 3 || mindFlay.definition.effects?.[0]?.type !== "drainTick" ||
    mindFlay.definition.effects?.[0]?.healFrac !== 0) {
  throw new Error("M4 Mind Flay source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /mindFlayAbilityCode\([\s\S]*?startOfflineMindFlayCast[\s\S]*?launchOfflineMindFlayChannelTick/, "Mind Flay channel reducer is missing");
requireText(world, /startOfflineMindFlayCast[\s\S]*?cast\.armChannel[\s\S]*?entityResources\[casterIndex\]/, "Mind Flay must bill at channel start and arm the source channel");
requireText(world, /launchOfflineMindFlayChannelTick[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW/, "Mind Flay ticks must queue Shadow projectiles");
requireText(world, /landOfflineMindFlayProjectile[\s\S]*?healFrac"\) != 0\.0[\s\S]*?channelTickBonus[\s\S]*?nextAuthoritativeRandomUnit/, "Mind Flay landing must use one range draw, channel scaling and no self heal");
requireText(world, /stepRetainedCasting[\s\S]*?mindFlayAbilityCode\(\)[\s\S]*?launchOfflineMindFlayChannelTick/, "Mind Flay pulse dispatch must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?mindFlayAbilityCode\(\)[\s\S]*?landOfflineMindFlayProjectile/, "Mind Flay projectile landing must be dispatched");
requireText(world, /pub mindFlayCommandStateTest\(\): int[\s\S]*?mind_flay[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepRetainedCasting[\s\S]*?stepOfflineEastbrookProjectiles/, "Mind Flay state regression coverage is missing");

process.stdout.write(`WOS91 Mind Flay static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
