import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_COMMIT = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = path.resolve(root, "..", "..");
const sourceRoot = path.resolve(workspaceRoot, "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const source = (file) => execFileSync("git", ["-C", sourceRoot, "show", `${SOURCE_COMMIT}:${file}`], { encoding: "utf8" });
const requireText = (text, pattern, message) => { if (!pattern.test(text)) throw new Error(message); };

const classes = source("src/sim/content/classes.ts");
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const motion = source("src/sim/player_motion.ts");
const start = classes.indexOf("  ghost_wolf: {");
const end = classes.indexOf("  stormstrike: {", start);
const ghostWolf = classes.slice(start, end);
for (const needle of ["class: 'shaman'", "learnLevel: 16", "cost: 35", "castTime: 2.0", "cooldown: 0", "range: 0", "school: 'nature'", "requiresTarget: false", "type: 'selfBuff', kind: 'buff_speed', value: 1.4, duration: 3600"]) {
  if (!ghostWolf.includes(needle)) throw new Error(`source Ghost Wolf drifted: ${needle}`);
}
requireText(dispatch, /case 'selfBuff':[\s\S]*?ability\.id === 'ghost_wolf'[\s\S]*?existing >= 0[\s\S]*?p\.auras\.splice[\s\S]*?ctx\.applyAura/, "source Ghost Wolf toggle dispatch drifted");
requireText(motion, /a\.kind === 'slow'[\s\S]*?Math\.min[\s\S]*?a\.kind === 'buff_speed'[\s\S]*?Math\.max[\s\S]*?return slow \* speed/, "source movement aura composition drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/frostbrand_weapon',[\s\S]*?'ghost_wolf'/.test(generator) || !generator.includes("EXPECTED_ABILITY_COUNT = 79") || !zrGenerator.includes("document.entries.length === 79")) throw new Error("M4 Ghost Wolf scope is missing");
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "ghost_wolf");
if (!entry || entry.index !== 39 || entry.definition.school !== "nature" || entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "selfBuff") throw new Error("M4 Ghost Wolf projection drifted");

const ccGenerator = read("tools", "cc_contract_codegen.mjs");
const cc = read("scripts", "woc_game", "src", "generated", "cc_contract.zr");
if (!ccGenerator.includes("buff_speed: 8") || !cc.includes('if (kind == "buff_speed") { return <uint>8; }') || !cc.includes("pub isMotionSpeedBuffKindCode(code: uint): bool { return code == <uint>8; }")) throw new Error("Ghost Wolf motion kind contract is missing");
const motionState = read("scripts", "woc_game", "src", "world", "motion_aura_state.zr");
requireText(motionState, /movementMultiplier[\s\S]*?values: container\.Array<float>[\s\S]*?isMotionSpeedBuffKindCode[\s\S]*?return slow \* speed/, "WOS motion speed composition is missing");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /ghostWolfAbilityCode\([\s\S]*?startOfflineGhostWolfCast[\s\S]*?completeOfflineGhostWolfCast/, "Ghost Wolf timed reducer is missing");
requireText(world, /toggleOfflineGhostWolf[\s\S]*?removeMotionAuraAt[\s\S]*?motionAuraKindCode\("buff_speed"\)/, "Ghost Wolf toggle state is missing");
requireText(world, /startOfflineGhostWolfCast[\s\S]*?armTimed[\s\S]*?completeOfflineGhostWolfCast[\s\S]*?entityResources/, "Ghost Wolf delayed cost path is missing");
requireText(world, /ghostWolfMotionStateIsValid[\s\S]*?ghostWolfAbilityCode\(\)[\s\S]*?3600\.0[\s\S]*?1\.4/, "Ghost Wolf persisted row validation is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?ghostWolfAbilityCode\(\)[\s\S]*?startOfflineGhostWolfCast[\s\S]*?applySupportedCastCommand[\s\S]*?ghostWolfPayloadAbilityIsExact/, "Ghost Wolf command routes are missing");
requireText(world, /pub ghostWolfCommandStateTest\(\): int[\s\S]*?ghost_wolf[\s\S]*?entityMotionAura[\s\S]*?retainedPlayerMovementSpeedMultiplier[\s\S]*?appendTypedCastCommandForTest/, "Ghost Wolf state regression coverage is missing");
process.stdout.write(`WOS99 Ghost Wolf static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
