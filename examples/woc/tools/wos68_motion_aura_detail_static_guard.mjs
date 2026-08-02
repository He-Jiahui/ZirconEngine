import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /entityMotionAuraValues: container\.Array<float>[\s\S]*?entityMotionAuraBreakChanceScales: container\.Array<float>[\s\S]*?entityFearDrStages: container\.Array<int>[\s\S]*?entityFearDrResetAt: container\.Array<float>/,
  "WOS58 must own motion-aura detail and target Fear DR columns",
);
requireText(
  world,
  /writer\.u16\(<uint>78, 1, 1\)[\s\S]*?writer\.u32\(<uint>state\.entityMotionAuraValues\.length[\s\S]*?entityMotionAuraBreakChanceScales[\s\S]*?entityFearDrStages[\s\S]*?entityFearDrResetAt/,
  "WOS58 must write detail rows after historical state",
);
requireText(
  world,
  /schemaVersion != <uint>57 && schemaVersion != <uint>58 &&[\s\S]*?schemaVersion != <uint>59 && schemaVersion != <uint>60 &&[\s\S]*?schemaVersion != <uint>61[\s\S]*?schemaVersion >= <uint>58[\s\S]*?motionAuraDetailCount[\s\S]*?historicalMotionAuraIndex[\s\S]*?entityFearDrStages\.add\(0\)/,
  "WOS58 decoder must preserve WOS2-WOS57 default migration",
);
requireText(
  world,
  /removeMotionAuraAt[\s\S]*?entityMotionAuraValues\.removeAt\(auraIndex\)[\s\S]*?entityMotionAuraBreakChanceScales\.removeAt\(auraIndex\)/,
  "motion aura removal must preserve detail-row alignment",
);
requireText(
  world,
  /applyOfflineMotionAuraWithDetails[\s\S]*?entityMotionAuraValues\[auraIndex\] = value[\s\S]*?entityMotionAuraBreakChanceScales\[auraIndex\] = breakChanceScale[\s\S]*?entityMotionAuraValues\.add\(0\.0\)[\s\S]*?entityMotionAuraBreakChanceScales\.add\(0\.0\)/,
  "motion aura insert and refresh must preserve detailed source fields",
);
requireText(
  world,
  /motionAuraDetailsStateIsValid[\s\S]*?scale < 0\.0 \|\| scale > 1\.0[\s\S]*?entityFearDrStages\[entityIndex\] < 0/,
  "motion-aura detail invariants are missing",
);

const main = read("scripts", "woc_game", "src", "main.zr");
requireText(main, /\\"world_state\\":\\"WOS78\\"/, "WOC capability output must publish WOS78");

const contract = read("contracts", "world-state.md");
requireText(
  contract,
  /world state \(`WOS78`\)[\s\S]*?schema\s+58 then appends the source aura details[\s\S]*?WOS2-WOS57 decode with zero DR state/,
  "world-state contract must document WOS58 migration and ownership",
);

process.stdout.write("WOS68 motion-aura detail static guards passed\n");
