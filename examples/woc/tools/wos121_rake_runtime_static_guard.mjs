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
const start = classes.indexOf("  rake: {");
const end = classes.indexOf("  claw: {", start);
if (start < 0 || end < start) throw new Error("source Rake block is missing");
const rake = classes.slice(start, end);
for (const needle of [
  "name: 'Flense'", "class: 'druid'", "learnLevel: 5", "cost: 35",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: true", "awardsCombo: 1", "requiresForm: 'cat'",
  "requiresStealth: true", "type: 'weaponStrike', bonus: 8",
  "type: 'dot', total: 30, duration: 9, interval: 3", "rank: 2", "level: 18",
  "type: 'weaponStrike', bonus: 12", "type: 'dot', total: 48, duration: 9, interval: 3",
]) {
  if (!rake.includes(needle)) throw new Error(`source Rake drifted: ${needle}`);
}
requireText(casting, /ability\.requiresStealth && !p\.auras\.some\(\(a\) => a\.kind === 'stealth'\)/,
  "source stealth admission drifted");
requireText(effects, /if \(!preservesStealth\(ability\)\) ctx\.breakStealth\(p\)/,
  "source opener reveal ordering drifted");
requireText(effects, /case 'weaponStrike':[\s\S]*?if \(hit && ability\.awardsCombo\)[\s\S]*?case 'dot':[\s\S]*?ctx\.applyAura/,
  "source weapon-strike/combo/dot dispatch drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/prowl',[\s\S]*?'rake'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Rake projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "rake",
);
if (!entry || entry.index !== 61 || entry.definition.cost !== 35 ||
    !entry.definition.requiresStealth || entry.definition.requiresForm !== "cat" ||
    entry.definition.awardsCombo !== 1 || entry.definition.effects?.[0]?.bonus !== 8 ||
    entry.definition.effects[1]?.total !== 30 || entry.definition.ranks?.[0]?.effects?.[1]?.total !== 48) {
  throw new Error("M4 Rake projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /rakeAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("rake"\)/,
  "Rake catalog identity is missing");
requireText(world, /startOfflineRakeCast[\s\S]*?offlineProwlIsActive[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?clearOfflineProwl[\s\S]*?weaponStrike[\s\S]*?applyOfflineRakeDot/,
  "Rake Cat/stealth opener reducer is missing");
requireText(world, /applyOfflineRakeDot[\s\S]*?rakeDotProfileIsValid[\s\S]*?offlineDotAbilityCodes/,
  "Rake durable bleed row is missing");
requireText(world, /offlineDotStateIsValid[\s\S]*?rakeAbilityCode\(\)[\s\S]*?rakeDotProfileIsValid/,
  "Rake bleed validation is missing");
requireText(world, /stepOfflineEastbrookDots[\s\S]*?rakeAbilityCode\(\)[\s\S]*?m4AbilityCatalog\.indexOf\("rake"\)/,
  "Rake bleed tick threat mapping is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?rakeAbilityCode\(\)[\s\S]*?startOfflineRakeCast/,
  "Rake action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?rakePayloadAbilityIsExact[\s\S]*?startOfflineRakeCast/,
  "Rake typed routing is missing");
requireText(world, /pub rakeCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_cat"\)[\s\S]*?appendTypedCastCommandForTest/,
  "Rake state regression coverage is missing");
requireText(world, /if \(rakeCommandStateTest\(\) != 1\) \{[\s\S]*?return -115;/,
  "world selfTest must execute Rake");

process.stdout.write(`WOS121 Rake static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
