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
const casting = source("src/sim/combat/casting_lifecycle.ts");
const motion = source("src/sim/player_motion.ts");
const locomotion = source("src/sim/mob/locomotion.ts");
const start = classes.indexOf("  prowl: {");
const end = classes.indexOf("  rake: {", start);
if (start < 0 || end < start) throw new Error("source Prowl block is missing");
const prowl = classes.slice(start, end);
for (const needle of [
  "name: 'Stalk'", "class: 'druid'", "learnLevel: 5", "cost: 0",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: false", "requiresForm: 'cat'", "requiresOutOfCombat: true",
  "type: 'selfBuff'", "kind: 'stealth'", "value: 0.5", "duration: 3600",
]) {
  if (!prowl.includes(needle)) throw new Error(`source Prowl drifted: ${needle}`);
}
requireText(
  effects,
  /function isStealthToggle[\s\S]*?e\.type === 'selfBuff' && e\.kind === 'stealth'[\s\S]*?function preservesStealth[\s\S]*?isStealthToggle\(ability\)/,
  "source stealth-toggle preservation drifted",
);
requireText(
  effects,
  /case 'selfBuff':[\s\S]*?kind === 'stealth'[\s\S]*?existing >= 0[\s\S]*?p\.stealthed = false[\s\S]*?kind: eff\.kind[\s\S]*?value: eff\.value/,
  "source self-buff stealth toggle dispatch drifted",
);
requireText(casting, /ability\.requiresOutOfCombat && p\.inCombat/,
  "source out-of-combat cast admission drifted");
requireText(motion, /a\.kind === 'slow' \|\| a\.kind === 'stealth'/,
  "source stealth movement-slow fold drifted");
requireText(locomotion, /a\.kind === 'stealth'[\s\S]*?stealthDetectionRadius\(mob, e, radius\)/,
  "source idle stealth detection fold drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/demoralizing_roar',[\s\S]*?'prowl'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Prowl projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "prowl",
);
if (!entry || entry.index !== 60 || entry.definition.class !== "druid" ||
    entry.definition.learnLevel !== 5 || entry.definition.cost !== 0 ||
    entry.definition.castTime !== 0 || entry.definition.cooldown !== 0 ||
    entry.definition.range !== 0 || entry.definition.school !== "physical" ||
    entry.definition.requiresTarget || entry.definition.requiresForm !== "cat" ||
    !entry.definition.requiresOutOfCombat ||
    entry.definition.effects?.[0]?.type !== "selfBuff" ||
    entry.definition.effects[0].kind !== "stealth" ||
    entry.definition.effects[0].value !== 0.5 ||
    entry.definition.effects[0].duration !== 3600) {
  throw new Error("M4 Prowl projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const motionState = read("scripts", "woc_game", "src", "world", "motion_aura_state.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
requireText(world, /writer\.u16\(<uint>78, 1, 1\)/,
  "current encoder schema is missing");
requireText(world, /schemaVersion != <uint>67 &&\s*schemaVersion != <uint>68 &&\s*schemaVersion != <uint>69/,
  "current decoder admission is missing");
requireText(world, /schemaVersion >= <uint>67[\s\S]*?entityProwlRemaining[\s\S]*?entityProwlValues/,
  "WOS67 Prowl decoder tail is missing");
requireText(world, /prowlStateIsValid[\s\S]*?entityProwlRemaining[\s\S]*?entityProwlValues/,
  "Prowl state validity boundary is missing");
requireText(world, /prowlAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("prowl"\)/,
  "Prowl catalog identity is missing");
requireText(world, /prowlProfileIsValid[\s\S]*?"stealth"[\s\S]*?0\.5[\s\S]*?3600\.0/,
  "Prowl profile must retain the source self-buff fields");
requireText(world, /startOfflineProwlCast[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?entityInCombat[\s\S]*?prowlProfileIsValid/,
  "Prowl must use Cat/out-of-combat admission");
requireText(world, /startFormCast[\s\S]*?clearOfflineProwl[\s\S]*?completeFormCast/,
  "non-stealth form actions must reveal Prowl");
requireText(world, /stepOfflineEastbrookMobIdleAggro[\s\S]*?candidate\.stealthed = offlineProwlIsActive/,
  "idle aggro must receive live Prowl detection state");
requireText(world, /ageOfflineProwl[\s\S]*?remaining > 0\.05[\s\S]*?clearOfflineProwl\(state, index\)/,
  "Prowl fixed-tick expiry is missing");
requireText(world, /fixedTick[\s\S]*?ageOfflineDemoralizingRoar\(state\);[\s\S]*?ageOfflineProwl\(state\)/,
  "fixed tick must age Prowl before retained player and mob work");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?prowlAbilityCode\(\)[\s\S]*?startOfflineProwlCast/,
  "Prowl action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?prowlPayloadAbilityIsExact[\s\S]*?startOfflineProwlCast/,
  "Prowl typed routing is missing");
requireText(world, /pub prowlCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_cat"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastCommandForTest/,
  "Prowl state regression coverage is missing");
requireText(world, /if \(prowlCommandStateTest\(\) != 1\) \{[\s\S]*?return -114;/,
  "world selfTest must execute Prowl");
requireText(motionState, /movementMultiplierWithStealth[\s\S]*?stealthMultiplier[\s\S]*?slow = stealthMultiplier/,
  "motion state must fold Prowl into the source slow-before-speed order");
if (!main.includes('\\"world_state\\":\\"WOS78\\"') ||
    !protocol.includes('WORLD_STATE_FORMAT: &str = "WOS78"') ||
    !protocol.includes('WORLD_STATE_SCHEMA_VERSION: u16 = 78')) {
  throw new Error("WOC package metadata still advertises the prior WOS schema");
}

process.stdout.write(`WOS120 Prowl static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
