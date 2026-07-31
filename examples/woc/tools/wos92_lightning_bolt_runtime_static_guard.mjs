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
const abilityBlock = (id, nextId) => {
  const start = classes.indexOf(`  ${id}: {`);
  const end = classes.indexOf(`  ${nextId}: {`, start);
  if (start < 0 || end < 0) throw new Error(`source ${id} block is missing`);
  return classes.slice(start, end);
};
const lightningSource = abilityBlock("lightning_bolt", "rockbiter_weapon");
for (const needle of [
  "class: 'shaman'", "learnLevel: 1", "cost: 15", "castTime: 1.5", "cooldown: 0",
  "range: 30", "school: 'nature'", "projectileFx: 'lightning'",
  "type: 'directDamage', min: 15, max: 17",
  "rank: 2", "level: 8", "cost: 25", "castTime: 2.0", "min: 26, max: 30",
  "rank: 3", "level: 14", "cost: 40", "castTime: 2.5", "min: 45, max: 51",
  "rank: 4", "level: 20", "cost: 60", "castTime: 3.0", "min: 75, max: 85",
]) {
  if (!lightningSource.includes(needle)) throw new Error(`source Lightning Bolt drifted: ${needle}`);
}
requireText(
  casting,
  /const firesProjectile = ability\.projectile \?\? ability\.school !== 'physical';[\s\S]*?isSpellResisted\([\s\S]*?ctx\.runEffects\(src, meta, tgt, res, !isSpell\)/,
  "source spell projectile and resist ordering drifted",
);
requireText(
  dispatch,
  /case 'directDamage':[\s\S]*?ctx\.rng\.range\(eff\.min, eff\.max\)[\s\S]*?directHitBonus\([\s\S]*?ctx\.rng\.chance[\s\S]*?Math\.round\(dmg\)/,
  "source direct spell damage ordering drifted",
);

const sourceGenerator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'mind_flay',[\s\S]*?'lightning_bolt'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Lightning Bolt projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const lightningBolt = m4.entries.find((entry) => entry.id === "lightning_bolt");
if (!lightningBolt || lightningBolt.index !== 32 || lightningBolt.scenarios.length !== 0 ||
    lightningBolt.definition.class !== "shaman" || lightningBolt.definition.school !== "nature" ||
    lightningBolt.definition.castTime !== 1.5 || lightningBolt.definition.effects?.[0]?.type !== "directDamage" ||
    lightningBolt.definition.effects?.[0]?.min !== 15 || lightningBolt.definition.effects?.[0]?.max !== 17) {
  throw new Error("M4 Lightning Bolt source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /lightningBoltAbilityCode\([\s\S]*?startOfflineLightningBoltCast[\s\S]*?completeOfflineLightningBoltCast/, "Lightning Bolt cast reducer is missing");
requireText(world, /startOfflineLightningBoltCast[\s\S]*?cast\.armTimed[\s\S]*?entityCastTargetIds/, "Lightning Bolt must arm its rank-aware cast and target lock");
requireText(world, /completeOfflineLightningBoltCast[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE/, "Lightning Bolt completion must queue a Nature projectile");
requireText(world, /landOfflineLightningBoltProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "nature"[\s\S]*?timedSpell\.resolveTimedSpellHit/, "Lightning Bolt landing must resolve one resist followed by direct Nature damage");
requireText(world, /stepRetainedCasting[\s\S]*?lightningBoltAbilityCode\(\)[\s\S]*?completeOfflineLightningBoltCast/, "Lightning Bolt completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?lightningBoltAbilityCode\(\)[\s\S]*?landOfflineLightningBoltProjectile/, "Lightning Bolt projectile landing must be dispatched");
requireText(world, /pub lightningBoltCommandStateTest\(\): int[\s\S]*?lightning_bolt[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?stepRetainedCasting[\s\S]*?stepOfflineEastbrookProjectiles/, "Lightning Bolt state regression coverage is missing");

process.stdout.write(`WOS92 Lightning Bolt static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
