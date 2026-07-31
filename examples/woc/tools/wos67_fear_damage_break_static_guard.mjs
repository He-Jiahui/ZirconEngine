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

const sourceTypes = source("src/sim/types.ts");
const sourceEffects = source("src/sim/combat/effect_dispatch.ts");
const sourceDamage = source("src/sim/combat/damage.ts");
requireText(
  sourceTypes,
  /min\(1, amount \/ \(breakChanceScale \* maxHp\)\)[\s\S]*?breakChanceScale\?: number;/,
  "source Fear damage-break contract drifted",
);
requireText(
  sourceEffects,
  /export const FEAR_BREAK_CHANCE_SCALE = 0\.1;[\s\S]*?kind: 'incapacitate',[\s\S]*?breaksOnDamage: true,[\s\S]*?breakChanceScale: ability\.fearDr \? FEAR_BREAK_CHANCE_SCALE : undefined/,
  "source Fear aura dispatch drifted",
);
requireText(
  sourceDamage,
  /breakable\.breakChanceScale !== undefined[\s\S]*?target\.maxHp > 0[\s\S]*?ctx\.rng\.chance\(Math\.min\(1, amount \/ \(breakable\.breakChanceScale \* target\.maxHp\)\)\)/,
  "source graded damage break ordering drifted",
);

const aura = read("scripts", "woc_game", "src", "combat", "effect_aura_dispatch_state.zr");
requireText(
  aura,
  /auraBreakChanceScales: container\.Array<float>[\s\S]*?removeAuraAt[\s\S]*?auraBreakChanceScales\.removeAt\(index\)[\s\S]*?appendAuraWithBreakChance[\s\S]*?auraBreakChanceScales\.add\(breakChanceScale\)/,
  "aura state must preserve the graded break scale through insertion and removal",
);
requireText(
  aura,
  /dispatchIncapacitate[\s\S]*?applyAuraWithBreakChance[\s\S]*?abilityId == "fear" \? 0\.1 : 0\.0/,
  "Fear aura dispatch must encode its source break scale",
);
requireText(
  aura,
  /fear\.target\.auraBreakChanceScales\[0\], 0\.1/,
  "aura dispatch regression coverage is missing",
);

const damage = read("scripts", "woc_game", "src", "combat", "damage_state.zr");
requireText(
  damage,
  /auraBreakChanceScales: container\.Array<float>[\s\S]*?addAuraWithBreakChance[\s\S]*?auraBreakChanceScales\.add\(breakChanceScale\)[\s\S]*?removeAuraAt[\s\S]*?auraBreakChanceScales\.removeAt\(index\)/,
  "damage state must retain a parallel graded break-scale array",
);
requireText(
  damage,
  /takeFrenzyRandomUnit[\s\S]*?frenzyRandomDraws = events\.frenzyRandomDraws \+ 1[\s\S]*?breakChanceScale > 0\.0 && target\.maxHp > 0[\s\S]*?amount \/[\s\S]*?breakChanceScale \* <float>target\.maxHp[\s\S]*?takeRandomUnit\(events\) >= breakChance/,
  "damage reducer must make the Fear draw independently and only when its aura is present",
);
requireText(
  damage,
  /fearBreakTarget[\s\S]*?basicHit\(50, "shadow"\)[\s\S]*?events\.randomIndex != 1[\s\S]*?basicHit\(100, "shadow"\)[\s\S]*?events\.randomIndex != 2[\s\S]*?ordinaryBreakTarget[\s\S]*?events\.randomIndex != 0/,
  "damage reducer regression coverage must distinguish graded and immediate breaks",
);

process.stdout.write(`WOS67 Fear damage-break static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
