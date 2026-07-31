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
const start = classes.indexOf("  flametongue_weapon: {");
const end = classes.indexOf("  frost_shock: {", start);
const flametongue = classes.slice(start, end);
for (const needle of ["class: 'shaman'", "learnLevel: 5", "cost: 25", "castTime: 0", "cooldown: 0", "range: 0", "school: 'fire'", "requiresTarget: false", "type: 'imbue', bonus: 8, duration: 300", "rank: 2", "level: 18", "cost: 40", "bonus: 13"]) {
  if (!flametongue.includes(needle)) throw new Error(`source Flametongue Weapon drifted: ${needle}`);
}
requireText(dispatch, /case 'imbue':[\s\S]*?a\.kind === 'imbue' && a\.id !== ability\.id[\s\S]*?ctx\.applyAura\(p, \{[\s\S]*?kind: 'imbue'[\s\S]*?value: eff\.bonus/, "source imbue replacement drifted");
requireText(autoAttack, /weapon imbues[\s\S]*?a\.kind === 'imbue'[\s\S]*?\+\s*imbueBonus[\s\S]*?ctx\.dealDamage\(attacker, target, dealtAmount, crit, 'physical'/, "source imbue swing damage drifted");

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
if (!/flame_shock',[\s\S]*?'flametongue_weapon'/.test(generator) || !generator.includes("EXPECTED_ABILITY_COUNT = 79") || !zrGenerator.includes("document.entries.length === 79")) throw new Error("M4 Flametongue Weapon scope is missing");
const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const entry = m4.entries.find((value) => value.id === "flametongue_weapon");
if (!entry || entry.index !== 37 || entry.definition.school !== "fire" || entry.definition.requiresTarget || entry.definition.effects?.[0]?.type !== "imbue") throw new Error("M4 Flametongue Weapon projection drifted");

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(world, /flametongueWeaponAbilityCode\([\s\S]*?startOfflineFlametongueWeaponCast/, "Flametongue Weapon reducer is missing");
requireText(world, /startOfflineFlametongueWeaponCast[\s\S]*?entityResources\[casterIndex\][\s\S]*?applyOfflineFlametongueWeapon/, "Flametongue Weapon admission is missing");
requireText(world, /offlineImbueBonus[\s\S]*?flametongueWeaponAbilityCode\(\)[\s\S]*?effects\.metric/, "Flametongue Weapon swing projection is missing");
requireText(world, /imbueStateIsValid[\s\S]*?flametongueWeaponAbilityCode\(\)[\s\S]*?flametongueWeaponProfileIsValid/, "Flametongue Weapon snapshot validation is missing");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?flametongueWeaponAbilityCode\(\)[\s\S]*?startOfflineFlametongueWeaponCast[\s\S]*?applySupportedCastCommand[\s\S]*?flametongueWeaponPayloadAbilityIsExact/, "Flametongue Weapon command routes are missing");
requireText(world, /pub flametongueWeaponCommandStateTest\(\): int[\s\S]*?flametongue_weapon[\s\S]*?entityImbueAbilityCodes[\s\S]*?imbueBonus[\s\S]*?appendTypedCastCommandForTest/, "Flametongue Weapon state regression coverage is missing");
process.stdout.write(`WOS97 Flametongue Weapon static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
