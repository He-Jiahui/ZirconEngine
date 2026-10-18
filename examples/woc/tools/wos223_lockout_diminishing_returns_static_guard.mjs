import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_COMMIT = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = path.resolve(root, "..", "..");
const sourceRoot = path.resolve(workspaceRoot, "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const readWorkspace = (...parts) => fs.readFileSync(path.join(workspaceRoot, ...parts), "utf8");
const source = (file) => execFileSync(
  "git", ["-C", sourceRoot, "show", `${SOURCE_COMMIT}:${file}`], { encoding: "utf8" },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const sim = source("src/sim/sim.ts");
const dispatch = source("src/sim/combat/effect_dispatch.ts");
requireText(
  sim,
  /PVP_STUN_DR_RESET = 18[\s\S]*?PVP_CC_DR_MULTIPLIERS = \[1, 0\.5, 0\.25\][\s\S]*?category === 'lockout'[\s\S]*?stage >= PVP_CC_DR_MULTIPLIERS\.length[\s\S]*?return null/,
  "source Lockout DR ladder drifted",
);
requireText(
  dispatch,
  /case 'interrupt':[\s\S]*?diminishedCrowdControlDuration\(p, target, 'lockout', eff\.lockout\)[\s\S]*?ctx\.cancelCast\(target\)[\s\S]*?rageOnInterrupt[\s\S]*?if \(remaining === null\) break/,
  "source Pummel DR, cancellation, and Rage ordering drifted",
);

const state = read("scripts", "woc_game", "src", "combat", "lockout_dr_state.zr");
requireText(
  state,
  /lockoutResetSeconds\(\): float \{ return 18\.0; \}[\s\S]*?appliesToPair[\s\S]*?stage >= 3[\s\S]*?false, 0\.0, true[\s\S]*?duration \* multiplier/,
  "WOC Lockout DR pure resolution is incomplete",
);
requireText(
  state,
  /first\.duration != 4\.0[\s\S]*?second\.duration != 2\.0[\s\S]*?third\.duration != 1\.0[\s\S]*?immune\.shouldStore[\s\S]*?expired\.shouldStore/,
  "WOC Lockout DR ladder coverage is incomplete",
);

const plan = readWorkspace("docs", "plans", "woc", "223-lockout-diminishing-returns-state.md");
requireText(
  plan,
  /WOS114[\s\S]*?entityLockoutDrStages[\s\S]*?entityLockoutDrResetAt[\s\S]*?WOS115/,
  "WOS223 persistence handoff is incomplete",
);
const fixture = read("scripts", "woc_game", "woc_lockout_dr_state_tests.zrp");
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/combat/lockout_dr_state_test_main.zr"')) {
  throw new Error("Lockout DR ZrVM project fixture is missing");
}

process.stdout.write(`WOS223 Lockout DR static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
