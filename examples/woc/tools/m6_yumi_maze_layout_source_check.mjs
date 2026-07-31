import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/yumi_maze_layout.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_maze_layout.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_maze_layout_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_yumi_maze_layout_tests.zrp'), 'utf8'),
);

for (const needle of [
  'export const YUMI_MAZE_SEED = 0xca7f00d;',
  'export const YUMI_MAZE_COLS = 13;',
  'export const YUMI_MAZE_ROWS = 13;',
  'export const YUMI_MAZE_PITCH = 6.75;',
  'const rng = new Rng(seed);',
  '180-degree symmetrize by UNION of openings',
  'Carve the plazas',
  'Braid: every dead end opens one closed interior wall',
  'Run-length merge the surviving closed walls into stub rects',
  'const spawnOffsets = [',
  'return { x, z, facing: Math.atan2(-x, -z) };',
]) {
  invariant(source.includes(needle), `source Yumi maze drifted: ${needle}`);
}

for (const needle of [
  'var rng = %import("kernel/rng");',
  'var math = %import("zr.zircon.math");',
  'pub class YumiMazeLayout',
  'pub buildYumiMaze(seed: uint): YumiMazeLayout',
  'symmetrizeOpenings(result);',
  'carvePlazas(result);',
  'braidDeadEnds(result, localRng);',
  'appendMergedWalls(result);',
  'appendCellsAndSpawns(result);',
  'pub mazeCorridorDistance(',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC Yumi maze projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/yumi_maze_layout")',
  'yumiMaze.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC Yumi maze test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_yumi_maze_layout_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-yumi-maze-layout-tests' &&
    testProject.entry === 'social/yumi_maze_layout_test_main',
  'Yumi maze test project contract drifted',
);

process.stdout.write(`checked M6 Yumi maze source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
