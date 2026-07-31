import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/discord_tier.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'discord_tier_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'discord_tier_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_discord_tier_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  "{ index: 1, key: 'initiate', threshold: 0 }",
  "{ index: 4, key: 'knight', threshold: 2_000 }",
  "{ index: 8, key: 'mythic', threshold: 150_000 }",
  'const pts = Number.isFinite(lifetimePoints) ? Math.max(0, lifetimePoints) : 0;',
  'return discordStatusForPoints(lifetimePoints).index;',
  'Points still needed to reach the next rung, or null when already at the top.',
  "link: { reason: 'link', points: 250 }",
  "guildMember: { reason: 'guild_member', points: 250 }",
  "booster: { reason: 'booster', points: 1_000 }",
  "dailyActive: { reason: 'daily_active', points: 50 }",
  "id: 'title_discordian'",
  "id: 'title_squire'",
  "id: 'chroma_blurple'",
  "id: 'title_champion'",
  "if (claimedIds.includes(swag.id)) return { ok: false, reason: 'claimed' };",
  "if (statusTier < swag.minTier) return { ok: false, reason: 'tier' };",
  "if (spendablePoints < swag.cost) return { ok: false, reason: 'points' };",
]) {
  invariant(source.includes(needle), `source Discord-tier rule drifted: ${needle}`);
}

for (const needle of [
  'pub discordStatusTierCount(): int { return 8; }',
  'pub normalizedDiscordLifetimePoints(lifetimePoints: int): int {',
  'pub discordStatusIndexForPoints(lifetimePoints: int): int {',
  'pub hasNextDiscordStatus(lifetimePoints: int): bool {',
  'pub discordPointsToNextStatus(lifetimePoints: int): int {',
  'pub discordRewardReason(index: int): string {',
  'pub discordSwagIndexById(id: string): int {',
  'pub discordSwagClaimReason(',
  'if (alreadyClaimed) return "claimed";',
  'if (statusTier < discordSwagMinTier(swagIndex)) return "tier";',
  'if (spendablePoints < discordSwagCost(swagIndex)) return "points";',
  'discordSwagClaimReason(3, 1000, 3, false) != "ok"',
]) {
  invariant(projection.includes(needle), `WOC Discord-tier projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/discord_tier_state")',
  'discordTier.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC Discord-tier test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_discord_tier_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-discord-tier-state-tests' &&
    testProject.entry === 'social/discord_tier_state_test_main',
  'Discord-tier test project contract drifted',
);

process.stdout.write(`checked M6 Discord-tier source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
