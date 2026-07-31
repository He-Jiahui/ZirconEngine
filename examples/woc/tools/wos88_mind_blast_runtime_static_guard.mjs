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
const mindBlastSource = abilityBlock("mind_blast", "heal");
for (const needle of [
  "class: 'priest'", "learnLevel: 5", "cost: 50", "castTime: 1.5", "cooldown: 8",
  "range: 30", "school: 'shadow'", "type: 'directDamage', min: 42, max: 46",
  "rank: 2", "level: 14", "cost: 70", "min: 60, max: 66",
  "rank: 3", "level: 20", "cost: 95", "min: 86, max: 94",
]) {
  if (!mindBlastSource.includes(needle)) throw new Error(`source Mind Blast drifted: ${needle}`);
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
if (!/WOC_RETAINED_ABILITY_IDS\s*=\s*\[[\s\S]*?'smite',[\s\S]*?'mind_blast'/.test(sourceGenerator) ||
!sourceGenerator.includes("EXPECTED_ABILITY_COUNT = 79") ||
!zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Mind Blast projection scope is missing");
}
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const mindBlast = m4.entries.find((entry) => entry.id === "mind_blast");
if (!mindBlast || mindBlast.index !== 28 || mindBlast.scenarios.length !== 0 ||
    mindBlast.definition.school !== "shadow" || mindBlast.definition.castTime !== 1.5 ||
    mindBlast.definition.cooldown !== 8 ||
    mindBlast.definition.effects?.[0]?.type !== "directDamage" ||
    mindBlast.definition.effects?.[0]?.min !== 42 || mindBlast.definition.effects?.[0]?.max !== 46) {
  throw new Error("M4 Mind Blast source projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /mindBlastAbilityCode\([\s\S]*?startOfflineMindBlastCast[\s\S]*?completeOfflineMindBlastCast/, "Mind Blast cast reducer is missing");
requireText(world, /startOfflineMindBlastCast[\s\S]*?abilityCooldownExpiresAt[\s\S]*?cast\.armTimed[\s\S]*?setAbilityCooldownExpiration/, "Mind Blast must require and set its source cooldown");
requireText(world, /completeOfflineMindBlastCast[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_SHADOW/, "Mind Blast completion must queue a Shadow projectile");
requireText(world, /landOfflineMindBlastProjectile[\s\S]*?spellResist\.resolve[\s\S]*?school = "shadow"[\s\S]*?timedSpell\.resolveTimedSpellHit/, "Mind Blast landing must resolve one resist followed by direct Shadow damage");
requireText(world, /stepRetainedCasting[\s\S]*?mindBlastAbilityCode\(\)[\s\S]*?completeOfflineMindBlastCast/, "Mind Blast completion must be registered in the retained cast step");
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?mindBlastAbilityCode\(\)[\s\S]*?landOfflineMindBlastProjectile/, "Mind Blast projectile landing must be dispatched");
requireText(world, /pub mindBlastCommandStateTest\(\): int[\s\S]*?mind_blast[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?abilityCooldownExpiresAt[\s\S]*?stepRetainedCasting[\s\S]*?stepOfflineEastbrookProjectiles/, "Mind Blast state regression coverage is missing");

process.stdout.write(`WOS88 Mind Blast static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
