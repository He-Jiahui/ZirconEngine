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
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const start = classes.indexOf("  entangling_roots: {");
const end = classes.indexOf("  bear_form: {", start);
const roots = classes.slice(start, end);
for (const needle of [
  "class: 'druid'", "learnLevel: 8", "cost: 35", "castTime: 1.5",
  "cooldown: 0", "range: 30", "school: 'nature'", "requiresTarget: true",
  "type: 'root', duration: 12", "rank: 2", "level: 16", "cost: 50",
  "type: 'dot', total: 32, duration: 12, interval: 3",
]) {
  if (!roots.includes(needle)) throw new Error(`source Entangling Roots drifted: ${needle}`);
}
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source spell projectile and resist ordering drifted",
);
requireText(
  dispatch,
  /case 'root':[\s\S]*?ctx\.applyRootAura\([\s\S]*?ctx\.enterCombat\(/,
  "source root dispatch drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/starfire',[\s\S]*?'entangling_roots'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Entangling Roots projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "entangling_roots",
);
if (!entry || entry.index !== 56 || entry.definition.class !== "druid" ||
    entry.definition.school !== "nature" || entry.definition.learnLevel !== 8 ||
    entry.definition.cost !== 35 || entry.definition.castTime !== 1.5 ||
    entry.definition.cooldown !== 0 || entry.definition.range !== 30 ||
    !entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "root" ||
    entry.definition.effects[0].duration !== 12 || entry.definition.ranks?.length !== 1 ||
    entry.definition.ranks[0].rank !== 2 || entry.definition.ranks[0].level !== 16 ||
    entry.definition.ranks[0].cost !== 50 || entry.definition.ranks[0].effects?.[0]?.type !== "root" ||
    entry.definition.ranks[0].effects[0].duration !== 12 ||
    entry.definition.ranks[0].effects[1]?.type !== "dot" ||
    entry.definition.ranks[0].effects[1].total !== 32 ||
    entry.definition.ranks[0].effects[1].duration !== 12 ||
    entry.definition.ranks[0].effects[1].interval !== 3) {
  throw new Error("M4 Entangling Roots projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /entanglingRootsAbilityCode\([\s\S]*?entanglingRootsPayloadAbilityIsExact[\s\S]*?entanglingRootsProjectileProfileIsValid/,
  "Entangling Roots identity and snapshot profile are missing");
requireText(world, /motionAuraEntityIsRooted[\s\S]*?motionAuras\.isRooted[\s\S]*?stepOfflineEastbrookMobMeleePursuit[\s\S]*?pursuit\.rooted = rooted/,
  "Entangling Roots must suppress Eastbrook movement through the root aura");
requireText(world, /startOfflineEntanglingRootsCast[\s\S]*?entanglingRootsTargetIndex[\s\S]*?cast\.armTimed[\s\S]*?entanglingRootsGlobalCooldownSeconds/,
  "Entangling Roots cast admission is missing");
requireText(world, /completeOfflineEntanglingRootsCast[\s\S]*?entityResources[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE/,
  "Entangling Roots completion must queue a Nature projectile");
requireText(world, /offlineProjectileStateIsValid[\s\S]*?entanglingRootsAbilityCode\([\s\S]*?entanglingRootsProjectileProfileIsValid/,
  "Entangling Roots in-flight state validation is missing");
requireText(world, /landOfflineEntanglingRootsProjectile[\s\S]*?spellResist\.resolve[\s\S]*?applyOfflineEntanglingRootsRoot[\s\S]*?appendOfflineEntanglingRootsDot/,
  "Entangling Roots landing must resolve resist before root then Rank 2 dot");
requireText(world, /offlineDotStateIsValid[\s\S]*?entanglingRootsDotProfileIsValid[\s\S]*?stepOfflineEastbrookDots[\s\S]*?entanglingRootsAbilityCode\(\)/,
  "Entangling Roots periodic state and threat validation are missing");
requireText(world, /stepRetainedCasting[\s\S]*?entanglingRootsAbilityCode\(\)[\s\S]*?completeOfflineEntanglingRootsCast/,
  "Entangling Roots completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?entanglingRootsAbilityCode\(\)[\s\S]*?landOfflineEntanglingRootsProjectile/,
  "Entangling Roots projectile landing must be dispatched");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?entanglingRootsAbilityCode\(\)[\s\S]*?startOfflineEntanglingRootsCast[\s\S]*?applySupportedCastCommand[\s\S]*?entanglingRootsPayloadAbilityIsExact/,
  "Entangling Roots command routes are missing");
requireText(world, /pub entanglingRootsCommandStateTest\(\): int[\s\S]*?m4AbilityCatalog\.indexOf\("entangling_roots"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastTargetCommandForTest/,
  "Entangling Roots state regression coverage is missing");
requireText(world, /if \(entanglingRootsCommandStateTest\(\) != 1\) \{[\s\S]*?return -110;/,
  "world selfTest must execute Entangling Roots");

process.stdout.write(`WOS116 Entangling Roots static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
