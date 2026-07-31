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
const sim = source("src/sim/sim.ts");
const casting = source("src/sim/combat/casting_lifecycle.ts");
const start = classes.indexOf("  demoralizing_roar: {");
const end = classes.indexOf("  cat_form: {", start);
if (start < 0 || end < start) throw new Error("source Demoralizing Roar block is missing");
const roar = classes.slice(start, end);
for (const needle of [
  "name: 'Craven Roar'", "class: 'druid'", "learnLevel: 10", "cost: 10",
  "castTime: 0", "cooldown: 0", "range: 0", "school: 'physical'",
  "requiresTarget: false", "requiresForm: 'bear'", "type: 'aoeAttackPower'",
  "amount: 20", "duration: 20", "radius: 8", "rank: 2", "level: 16",
  "amount: 35",
]) {
  if (!roar.includes(needle)) throw new Error(`source Demoralizing Roar drifted: ${needle}`);
}
requireText(
  effects,
  /case 'aoeAttackPower':[\s\S]*?hostilesInRadius\(p, p\.pos, eff\.radius\)[\s\S]*?kind: 'debuff_ap'[\s\S]*?value: eff\.amount \?\? 0[\s\S]*?addThreat\(m, p\.id, 10 \* ctx\.threatMod\(p, ability\.school\)\)/,
  "source aoeAttackPower hostile aura, value or threat dispatch drifted",
);
requireText(
  sim,
  /effectiveAttackPower\(e: Entity\)[\s\S]*?else if \(a\.kind === 'debuff_ap'\) attackPower -= a\.value[\s\S]*?return Math\.max\(0, attackPower\)/,
  "source debuff_ap effective attack-power fold drifted",
);
requireText(
  casting,
  /const gcd = Math\.max\(MIN_GCD, ctx\.playerGcdFor\(meta\.cls\) \/ spellHasteMult\(p\)\)[\s\S]*?if \(!ability\.offGcd\) p\.gcdRemaining = Math\.max\(p\.gcdRemaining, gcd\)/,
  "source instant GCD lifecycle drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/growl',[\s\S]*?'demoralizing_roar'/.test(generator) ||
    !generator.includes("EXPECTED_ABILITY_COUNT = 79") ||
    !zrGenerator.includes("document.entries.length === 79")) {
  throw new Error("M4 Demoralizing Roar projection scope is missing");
}
const entry = JSON.parse(read("contracts", "m4_abilities.json")).entries.find(
  (value) => value.id === "demoralizing_roar",
);
if (!entry || entry.index !== 59 || entry.definition.class !== "druid" ||
    entry.definition.learnLevel !== 10 || entry.definition.cost !== 10 ||
    entry.definition.castTime !== 0 || entry.definition.cooldown !== 0 ||
    entry.definition.range !== 0 || entry.definition.school !== "physical" ||
    entry.definition.requiresTarget || entry.definition.requiresForm !== "bear" ||
    entry.definition.effects?.[0]?.type !== "aoeAttackPower" ||
    entry.definition.effects[0].amount !== 20 || entry.definition.effects[0].duration !== 20 ||
    entry.definition.effects[0].radius !== 8 || entry.definition.ranks?.length !== 1 ||
    entry.definition.ranks[0].rank !== 2 || entry.definition.ranks[0].level !== 16 ||
    entry.definition.ranks[0].cost !== 10 ||
    entry.definition.ranks[0].effects?.[0]?.amount !== 35 ||
    entry.definition.ranks[0].effects[0].duration !== 20 ||
    entry.definition.ranks[0].effects[0].radius !== 8) {
  throw new Error("M4 Demoralizing Roar projection drifted");
}

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
requireText(world, /writer\.u16\(<uint>71, 1, 1\)/,
  "WOS66 encoder schema is missing");
requireText(world, /schemaVersion != <uint>65 && schemaVersion != <uint>66/,
  "WOS66 decoder admission is missing");
requireText(world, /schemaVersion >= <uint>66[\s\S]*?entityDemoralizingRoarRemaining[\s\S]*?entityDemoralizingRoarValues/,
  "WOS66 Demoralizing Roar decoder tail is missing");
requireText(world, /demoralizingRoarAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("demoralizing_roar"\)/,
  "Demoralizing Roar catalog identity is missing");
requireText(world, /startOfflineDemoralizingRoarCast[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?"aoeAttackPower"[\s\S]*?entityDemoralizingRoarRemaining[\s\S]*?enterOfflineSunderArmorCombat[\s\S]*?\+ 10\.0/,
  "Demoralizing Roar must use Bear admission, source range aura and threat settlement");
requireText(world, /effectiveOfflineAttackPower[\s\S]*?entityDemoralizingRoarValues[\s\S]*?return attackPower > 0\.0 \? attackPower : 0\.0/,
  "Demoralizing Roar effective attack-power fold is missing");
requireText(world, /resolveOfflineEastbrookMobSwingRequests[\s\S]*?swing\.effectiveAttackPower = effectiveOfflineAttackPower\(state, mobIndex\)/,
  "mob swing must read effective attack power");
requireText(world, /ageOfflineDemoralizingRoar[\s\S]*?remaining > 0\.05[\s\S]*?entityDemoralizingRoarValues\[index\] = 0\.0/,
  "Demoralizing Roar expiry is missing");
requireText(world, /fixedTick[\s\S]*?ageOfflineSunderArmor\(state\);[\s\S]*?ageOfflineDemoralizingRoar\(state\)/,
  "fixed tick must age Demoralizing Roar before mob lifecycle work");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?demoralizingRoarAbilityCode\(\)[\s\S]*?startOfflineDemoralizingRoarCast/,
  "Demoralizing Roar action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?demoralizingRoarPayloadAbilityIsExact[\s\S]*?startOfflineDemoralizingRoarCast/,
  "Demoralizing Roar typed routing is missing");
requireText(world, /pub demoralizingRoarCommandStateTest\(\): int[\s\S]*?forms\.formAbilityCode\("form_bear"\)[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?appendTypedCastCommandForTest/,
  "Demoralizing Roar state regression coverage is missing");
requireText(world, /if \(demoralizingRoarCommandStateTest\(\) != 1\) \{[\s\S]*?return -113;/,
  "world selfTest must execute Demoralizing Roar");
if (!main.includes('\\"world_state\\":\\"WOS71\\"')) {
  throw new Error("WOC package metadata still advertises the prior WOS schema");
}

process.stdout.write(`WOS119 Demoralizing Roar static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
