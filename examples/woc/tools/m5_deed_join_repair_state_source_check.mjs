import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src');
const deeds = gitShow('src/sim/deeds.ts');
const sim = gitShow('src/sim/sim.ts');
const projection = readFileSync(
  resolve(wocSourceRoot, 'progression', 'deed_join_repair_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocSourceRoot, 'progression', 'deed_join_repair_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'woc_m5_deed_join_repair_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  'export const MAX_CREDITABLE_MOB_LEVEL = 22;',
  "if (player.level + 5 > MAX_CREDITABLE_MOB_LEVEL) {",
  "grantDeed(ctx, meta, 'cmb_giantslayer', { retro: true });",
  'if (player.level >= MAX_LEVEL && meta.restedXp <= 0) {',
  "grantDeed(ctx, meta, 'prog_well_rested', { retro: true });",
  "if (GROUND_PICKUP_PROVING_QUESTS.some((q) => meta.questsDone.has(q))) {",
  "grantDeed(ctx, meta, 'exp_something_shiny', { retro: true });",
  "craftId !== 'enchanting' && v > 0",
  "grantDeed(ctx, meta, 'prog_first_craft', { retro: true });",
  "'q_glimmermere_light',",
]) {
  invariant(deeds.includes(needle), `missing pinned join-time deed repair behavior: ${needle}`);
}
invariant(
  sim.includes('deedsMod.retroFallbackGrants(this.ctx, meta, player);'),
  'world join no longer invokes the deed repair pass with the live player',
);

for (const needle of [
  'MAX_LEVEL: int = 20;', 'MAX_CREDITABLE_MOB_LEVEL: int = 22;',
  'pub class DeedJoinRepairState', 'pub setCraftSkill(', 'pub markQuestDone(',
  'pub hasEarnedDeed(', 'groundPickupProvingQuest(', 'pub repairDeedsOnJoin(',
  'state.playerLevel + 5 > MAX_CREDITABLE_MOB_LEVEL',
  'state.playerLevel >= MAX_LEVEL && state.restedXp <= 0',
  '"q_glimmermere_light"', 'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `join-time deed repair projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("progression/deed_join_repair_state")') &&
    testMain.includes('repairs.contractTest()'),
  'missing join-time deed repair test entry behavior',
);
invariant(
  testProject.name === 'woc_m5_deed_join_repair_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m5-deed-join-repair-state-tests' &&
    testProject.entry === 'progression/deed_join_repair_state_test_main',
  'join-time deed repair test project contract drifted',
);

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("progression/deed_join_repair_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) === JSON.stringify(['progression/deed_join_repair_state_test_main.zr']),
  `deed_join_repair_state escaped the focused fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M5 join-time deed repair source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

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
