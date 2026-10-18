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

const sourceChannels = source("src/sim/mob/healer_channel.ts");
const sourceDispatch = source("src/sim/combat/effect_dispatch.ts");
const sourceProfile = source("src/sim/mob/combat_profile.ts");
const sourceDungeons = source("src/sim/content/dungeons.ts");
requireText(
  sourceChannels,
  /NYTHRAXIS_SPIRIT_MENDING_CAST_ID = 'nythraxis_spirit_mending'[\s\S]*?SCRIPTED_INTERRUPTIBLE_CHANNELS[\s\S]*?shadow/,
  "source scripted interruptible channel vocabulary drifted",
);
requireText(
  sourceDispatch,
  /SCRIPTED_INTERRUPTIBLE_CHANNELS[\s\S]*?scriptedChannel[\s\S]*?school[\s\S]*?ctx\.cancelCast\(target\)/,
  "source interrupt dispatch no longer resolves scripted channels",
);
requireText(
  sourceProfile,
  /updateHealerHold[\s\S]*?HEALER_STANDOFF = 6[\s\S]*?NYTHRAXIS_SPIRIT_MENDING_CAST_ID[\s\S]*?castRemaining = Math\.max/,
  "source Nythraxis healer hold drifted",
);
requireText(
  sourceDungeons,
  /nythraxis_heroic_priest_add[\s\S]*?channelHeal:[\s\S]*?radius: 45[\s\S]*?every: 4[\s\S]*?baseHeal: 320[\s\S]*?rampAdd: 240[\s\S]*?maxHeal: 1440[\s\S]*?school: 'shadow'/,
  "source Nythraxis channel content drifted",
);

const channels = read("scripts", "woc_game", "src", "combat", "scripted_interruptible_channels.zr");
requireText(
  channels,
  /noScriptedCastCode\(\): uint \{ return <uint>0; \}[\s\S]*?nythraxisSpiritMendingCastCode[\s\S]*?return <uint>1;[\s\S]*?scriptedCastCode[\s\S]*?scriptedCastId[\s\S]*?scriptedInterruptibleChannelSchoolForCode[\s\S]*?isScriptedInterruptibleChannelCode/,
  "WOC scripted cast numeric vocabulary is incomplete",
);
const channelState = read("scripts", "woc_game", "src", "combat", "nythraxis_channel_state.zr");
requireText(
  channelState,
  /protecteeRadius\(\): float \{ return 45\.0; \}[\s\S]*?healerStandoff\(\): float \{ return 6\.0; \}[\s\S]*?channelEvery\(\): float \{ return 4\.0; \}[\s\S]*?channelBaseHeal\(\): float \{ return 320\.0; \}[\s\S]*?channelRampAdd\(\): float \{ return 240\.0; \}[\s\S]*?channelMaximumHeal\(\): float \{ return 1440\.0; \}[\s\S]*?protecteeIsEligible[\s\S]*?canHoldChannel[\s\S]*?resolveTick[\s\S]*?interrupted/,
  "WOC Nythraxis channel behavior foundation is incomplete",
);
const plan = read("..", "..", "docs", "plans", "woc", "222-scripted-interruptible-channel-state.md");
requireText(
  plan,
  /WOS114[\s\S]*?entityBossChannelTimers[\s\S]*?entityBossChannelRamps[\s\S]*?Pummel[\s\S]*?unknown pseudo-cast/,
  "WOS222 state-migration boundary drifted",
);
const fixture = read("scripts", "woc_game", "src", "combat", "scripted_interruptible_channels_test_main.zr");
if (!fixture.includes('channels.contractTest()')) {
  throw new Error("scripted interruptible channel contract entry is missing");
}
const projectFixture = read(
  "scripts", "woc_game", "woc_scripted_interruptible_channel_state_tests.zrp",
);
if (!projectFixture.includes('backend = "zr_vm:project"') ||
    !projectFixture.includes('entry = "src/combat/scripted_interruptible_channels_test_main.zr"')) {
  throw new Error("scripted interruptible channel ZrVM project fixture is missing");
}
const channelFixture = read("scripts", "woc_game", "woc_nythraxis_channel_state_tests.zrp");
if (!channelFixture.includes('backend = "zr_vm:project"') ||
    !channelFixture.includes('entry = "src/combat/nythraxis_channel_state_test_main.zr"')) {
  throw new Error("Nythraxis channel ZrVM project fixture is missing");
}

process.stdout.write(`WOS222 scripted channel static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
