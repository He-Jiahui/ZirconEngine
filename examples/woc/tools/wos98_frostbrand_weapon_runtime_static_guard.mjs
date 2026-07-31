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
const autoAttack = source("src/sim/combat/auto_attack.ts");
const start = classes.indexOf("  frostbrand_weapon: {");
const end = classes.indexOf("  ghost_wolf: {", start);
const frostbrand = classes.slice(start, end);
for (const needle of ["class: 'shaman'", "learnLevel: 5", "cost: 25", "castTime: 0", "cooldown: 0", "range: 0", "school: 'frost'", "requiresTarget: false", "type: 'imbue', bonus: 8, duration: 300", "rank: 2", "level: 20", "cost: 40", "bonus: 13"]) {
  if (!frostbrand.includes(needle)) throw new Error(`source Frostbrand Weapon drifted: ${needle}`);
}
requireText(dispatch, /case 'imbue':[\s\S]*?a\.kind === 'imbue' && a\.id !== ability\.id[\s\S]*?ctx\.applyAura\(p, \{[\s\S]*?kind: 'imbue'[\s\S]*?value: eff\.bonus/, "source imbue replacement drifted");
requireText(autoAttack, /weapon imbues[\s\S]*?a\.kind === 'imbue'[\s\S]*?\+\s*imbueBonus[\s\S]*?ctx\.dealDamage\(attacker, target, dealtAmount, crit, 'physical'/, "source imbue swing damage drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/flametongue_weapon',[\s\S]*?'frostbrand_weapon'/.test(generator) || !generator.includes("EXPECTED_ABILITY_COUNT = 79") || !zrGenerator.includes("document.entries.length === 79")) throw new Error("M4 Frostbrand Weapon scope is missing");
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "frostbrand_weapon");
if (!entry || entry.index !== 38 || entry.definition.school !== "frost" || entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "imbue") throw new Error("M4 Frostbrand Weapon projection drifted");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /frostbrandWeaponAbilityCode\([\s\S]*?startOfflineFrostbrandWeaponCast/, "Frostbrand Weapon reducer is missing");
requireText(world, /startOfflineFrostbrandWeaponCast[\s\S]*?entityResources\[casterIndex\][\s\S]*?applyOfflineFrostbrandWeapon/, "Frostbrand Weapon admission is missing");
requireText(world, /offlineImbueBonus[\s\S]*?frostbrandWeaponAbilityCode\(\)[\s\S]*?effects\.metric/, "Frostbrand Weapon swing projection is missing");
requireText(world, /imbueStateIsValid[\s\S]*?frostbrandWeaponAbilityCode\(\)[\s\S]*?frostbrandWeaponProfileIsValid/, "Frostbrand Weapon snapshot validation is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?frostbrandWeaponAbilityCode\(\)[\s\S]*?startOfflineFrostbrandWeaponCast[\s\S]*?applySupportedCastCommand[\s\S]*?frostbrandWeaponPayloadAbilityIsExact/, "Frostbrand Weapon command routes are missing");
requireText(world, /pub frostbrandWeaponCommandStateTest\(\): int[\s\S]*?frostbrand_weapon[\s\S]*?entityImbueAbilityCodes[\s\S]*?imbueBonus[\s\S]*?appendTypedCastCommandForTest/, "Frostbrand Weapon state regression coverage is missing");
process.stdout.write(`WOS98 Frostbrand Weapon static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
