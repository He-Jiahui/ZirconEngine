import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/holder_tier.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'holder_tier_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'holder_tier_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_holder_tier_state_tests.zrp'), 'utf8'),
);

for (const key of [
  'ember', 'coinbearer', 'coppercrest', 'silverbound', 'gilded', 'vaultwarden',
  'whale', 'leviathan', 'tidelord', 'stormcaller', 'krakencrown', 'titanforged',
  'starhoard', 'voidwarden', 'realmshaper', 'worldforger', 'worldbearer', 'sovereign',
]) {
  invariant(source.includes(`key: '${key}'`), `source holder tier key drifted: ${key}`);
  invariant(projection.includes(`"${key}"`), `WOC holder tier key is missing: ${key}`);
}

for (const needle of [
  'export const WOC_MAX_SUPPLY = 1_000_000_000;',
  'if (balance === null || !Number.isFinite(balance) || balance < HOLDER_TIER_DEFS[0].threshold)',
  'if (balance >= t.threshold) tier = t;',
  'holderTierForBalance(balance)?.index ?? 0',
  'return tier.threshold / WOC_MAX_SUPPLY;',
]) {
  invariant(source.includes(needle), `source holder tier rule drifted: ${needle}`);
}

for (const needle of [
  'pub wocMaxSupply(): int { return 1000000000; }',
  'pub holderTierCount(): int { return 18; }',
  'if (!hasBalance || balance < holderTierThreshold(1)) { return 0; }',
  'if (balance >= holderTierThreshold(index)) {',
  'return threshold > 0 ? <float>threshold / <float>wocMaxSupply() : 0.0;',
]) {
  invariant(projection.includes(needle), `WOC holder tier projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/holder_tier_state")',
  'holderTier.holderTierIndexForBalance(false, 1000000000) != 0',
  'holderTier.holderTierIndexForBalance(true, 1000000000) != 18',
  'holderTier.holderTierKey(18) != "sovereign"',
]) {
  invariant(testMain.includes(needle), `WOC holder tier contract is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_holder_tier_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-holder-tier-state-tests' &&
    testProject.entry === 'social/holder_tier_state_test_main',
  'holder tier test project contract drifted',
);

process.stdout.write(`checked M6 holder tier source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
