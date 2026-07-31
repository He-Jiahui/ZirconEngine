import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const loot = gitShow('src/sim/loot/loot_roll.ts');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src');
const projection = readFileSync(
  resolve(wocSourceRoot, 'progression', 'loot_distribution_state.zr'),
  'utf8',
);

for (const needle of [
  'export function pickRollGroupWinner(',
  'awardedItemIds: Set<string>,',
  'for (let offset = 0; offset < group.length; offset++)',
  'const candidate = group[(winnerIndex + offset) % group.length];',
  'if (!candidate.itemId || !awardedItemIds.has(candidate.itemId)) return candidate;',
  'const awardedItemIds = new Set<string>();',
  'const winner = pickRollGroupWinner(roll, group, awardedItemIds);',
  'awardedItemIds.add(winner.itemId);',
]) {
  invariant(loot.includes(needle), `missing current loot roll-group behavior: ${needle}`);
}

for (const needle of [
  '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a',
  'pub var awardedRollGroupItemIds:',
  'pub pickDistinctRollGroupWinner(',
  'rollGroupItemAlreadyAwarded(',
  'currentHeadRollGroupDedupTest(): int',
  'state.randomDraws != 0',
]) {
  invariant(projection.includes(needle), `loot projection omitted current roll-group behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("progression/loot_distribution_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['progression/loot_distribution_state_test_main.zr', 'progression/m5_scenario_matrix.zr']),
  `loot_distribution_state escaped the M5 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M5 loot roll-group dedup: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
