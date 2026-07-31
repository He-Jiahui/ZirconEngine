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
requireText(
  classes,
  /arcane_explosion:\s*\{[\s\S]*?specs: \['arcane'\],[\s\S]*?cost: 60,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 0,[\s\S]*?requiresTarget: false,[\s\S]*?effects: \[\{ type: 'aoeDamage', min: 26, max: 31, radius: 10 \}\]/,
  "source Arcane Explosion definition drifted",
);
requireText(
  casting,
  /if \(!ability\.offGcd\) p\.gcdRemaining = Math\.max\(p\.gcdRemaining, gcd\);[\s\S]*?applyAbility\(ctx, p, meta, instantResolved, castTargetId\);/,
  "source instant-cast GCD and effect dispatch drifted",
);
requireText(
  effects,
  /case 'aoeDamage':[\s\S]*?const aoeCenter = p\.castAim \?\? p\.pos;[\s\S]*?const aoeTargets: Entity\[\] = \[\];[\s\S]*?for \(const m of ctx\.hostilesInRadius\(p, aoeCenter, eff\.radius\)\)[\s\S]*?const aoeCrit =[\s\S]*?eff\.canCrit \?\? false[\s\S]*?for \(const m of aoeTargets\)[\s\S]*?ctx\.dealDamage\(/,
  "source AoE collection, crit, or damage order drifted",
);

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const arcaneEntry = m4.entries.find((entry) => entry.id === "arcane_explosion");
if (!arcaneEntry || arcaneEntry.index !== 11 || arcaneEntry.definition.specs?.[0] !== "arcane" ||
    arcaneEntry.definition.cost !== 60 || arcaneEntry.definition.castTime !== 0 ||
    arcaneEntry.definition.cooldown !== 0 || arcaneEntry.definition.requiresTarget !== false ||
    arcaneEntry.definition.effects?.[0]?.type !== "aoeDamage" ||
    arcaneEntry.definition.effects?.[0]?.min !== 26 ||
    arcaneEntry.definition.effects?.[0]?.max !== 31 ||
    arcaneEntry.definition.effects?.[0]?.radius !== 10) {
  throw new Error("M4 Arcane Explosion projection drifted");
}

const catalog = read("scripts", "woc_game", "src", "generated", "m4_ability_catalog.zr");
const generatedEffects = read("scripts", "woc_game", "src", "generated", "m4_ability_effects.zr");
requireText(catalog, /if \(id == "arcane_explosion"\) \{ return 11; \}/, "M4 Arcane Explosion index is missing");
requireText(generatedEffects, /pub typeAt[\s\S]*?if \(index == 11\)[\s\S]*?return "aoeDamage";/, "M4 Arcane Explosion effect type is missing");
requireText(generatedEffects, /pub metric[\s\S]*?if \(index == 11\)[\s\S]*?if \(field == "max"\) \{ return 31\.0; \}[\s\S]*?if \(field == "min"\) \{ return 26\.0; \}[\s\S]*?if \(field == "radius"\) \{ return 10\.0; \}/, "M4 Arcane Explosion effect metrics are missing");

const ground = read("scripts", "woc_game", "src", "combat", "ground_aoe_state.zr");
requireText(
  ground,
  /targetQualifies\([\s\S]*?state\.targetIds\[targetIndex\] != sourceId[\s\S]*?targetHostile[\s\S]*?targetLineOfSight/,
  "instant AoE must share one target eligibility predicate",
);
requireText(
  ground,
  /pub eligibleTargetCount\([\s\S]*?targetQualifies\(/,
  "instant AoE must count exactly the targets that consume RNG",
);
requireText(
  ground,
  /pub castInstantAoE\([\s\S]*?pulseValues\([\s\S]*?true[\s\S]*?\n\}/,
  "instant AoE kernel is missing or queues a persistent ground effect",
);
requireText(
  ground,
  /pub contractTest\(\): int[\s\S]*?castInstantAoE[\s\S]*?instant\.effectIds\.length != 0/,
  "instant AoE kernel regression coverage is missing",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /var groundAoE = %import\("combat\/ground_aoe_state"\);/, "WorldState must use the combat-owned AoE kernel");
requireText(world, /arcaneExplosionAbilityCode\([\s\S]*?abilityCode\("arcane_explosion"\)[\s\S]*?m4AbilityCatalog\.indexOf\("arcane_explosion"\)/, "Arcane Explosion catalog identity is missing");
requireText(world, /arcaneExplosionPayloadAbilityIsExact\([\s\S]*?abilityLength == <uint>16/, "Arcane Explosion typed payload admission is missing");
requireText(
  world,
  /startOfflineArcaneExplosionCast\([\s\S]*?catalogAdmission\([\s\S]*?m4AbilityEffects\.typeAt\(abilityIndex, rank, 0\) != "aoeDamage"[\s\S]*?spellScaling\.directHitBonus[\s\S]*?groundAoE\.eligibleTargetCount[\s\S]*?groundAoE\.castInstantAoE[\s\S]*?settleOfflineEastbrookLethal/,
  "Arcane Explosion world reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?arcaneExplosionAbilityCode\(\)[\s\S]*?startOfflineArcaneExplosionCast[\s\S]*?applySupportedCastCommand[\s\S]*?arcaneExplosionPayloadAbilityIsExact/,
  "Arcane Explosion slot and typed routes are missing",
);
requireText(world, /pub arcaneExplosionCommandStateTest\(\): int[\s\S]*?arcane_explosion/, "Arcane Explosion state regression coverage is missing");
requireText(world, /if \(arcaneExplosionCommandStateTest\(\) != 1\) \{\s*return -58;\s*\}/, "Arcane Explosion self-test route is missing");

process.stdout.write(`WOS62 Arcane Explosion static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
