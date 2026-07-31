import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/dev_tier.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'dev_tier_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'dev_tier_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_dev_tier_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  "{ index: 1, key: 'tinkerer', threshold: 1 }",
  "{ index: 2, key: 'artificer', threshold: 5 }",
  "{ index: 3, key: 'runesmith', threshold: 15 }",
  "{ index: 4, key: 'architect', threshold: 30 }",
  "{ index: 5, key: 'worldwright', threshold: 70 }",
  'export const DEV_TIER_SIGNIFICANT_INDEX = 4;',
  'if (mergedPrs === null || !Number.isFinite(mergedPrs) || mergedPrs < DEV_TIER_DEFS[0].threshold)',
  'return devTierForMergedPrs(mergedPrs)?.index ?? 0;',
  'return Number.isInteger(index) && index >= 1 && index <= DEV_TIER_DEFS.length',
  'index >= DEV_TIER_SIGNIFICANT_INDEX && index <= DEV_TIER_DEFS.length',
]) {
  invariant(source.includes(needle), `source developer-tier rule drifted: ${needle}`);
}

for (const needle of [
  'pub devTierCount(): int { return 5; }',
  'pub devTierSignificantIndex(): int { return 4; }',
  'if (index == 1) return "tinkerer";',
  'if (index == 5) return "worldwright";',
  'if (index == 5) return 70;',
  'pub hasDevTierIndex(index: int): bool {',
  'pub devTierIndexForMergedPrs(hasMergedPrs: bool, mergedPrs: int): int {',
  'if (!hasMergedPrs || mergedPrs < devTierThreshold(1)) { return 0; }',
  'pub isSignificantDevTier(index: int): bool {',
  'devTierIndexForMergedPrs(true, 500) != 5',
]) {
  invariant(projection.includes(needle), `WOC developer-tier projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/dev_tier_state")',
  'devTier.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC developer-tier test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_dev_tier_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-dev-tier-state-tests' &&
    testProject.entry === 'social/dev_tier_state_test_main',
  'developer-tier test project contract drifted',
);

process.stdout.write(`checked M6 developer-tier source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
