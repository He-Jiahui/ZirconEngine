import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/pathfind.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'pathfind_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'pathfind_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(
    resolve(wocRoot, 'scripts', 'woc_game', 'woc_m3_pathfind_state_tests.zrp'),
    'utf8',
  ),
);

for (const needle of [
  'const CELL = 1;',
  'const MARGIN = 8;',
  'const MAX_SPAN = 64;',
  'const SMOOTH_SAMPLE_STEP = 0.25;',
  'export const PLAYER_BODY_RADIUS = 0.5;',
  'export const PLAYER_MAX_CLIMB_SLOPE = 1.5;',
  'export const PLAYER_SWIM_DEPTH = 0.8;',
  'if (!o.ignoreFences && pathCrossesFence(from.x, from.z, to.x, to.z, o.bodyRadius)) return false;',
  'const steps = Math.max(1, Math.ceil(d / SMOOTH_SAMPLE_STEP));',
  'if (stepLen > 1e-6 && rise > 0 && rise / stepLen > o.maxClimbSlope) return false;',
  'if (W > maxSpan || H > maxSpan) return [{ x: to.x, z: to.z }];',
  '// diagonals only when both orthogonal cells are clear (no corner clipping)',
  'return smoothPath(points, o);',
  'minGround: swim ? -Infinity : (x: number, z: number) => waterLevelAt(x, z) - PLAYER_SWIM_DEPTH,',
  'const rings = 24;',
  'const samples = Math.max(12, Math.ceil(radius * 10));',
]) {
  invariant(source.includes(needle), `source pathfind rule drifted: ${needle}`);
}

assertOrder(source, [
  'function segmentWalkable(',
  'function smoothPath(',
  'export function findPath(',
  'export function findPlayerPath(',
  'export function resolvePlayerDestination(',
]);

for (const needle of [
  'pub var PATH_CELL_SIZE: float = 1.0;',
  'pub var PATH_MARGIN: float = 8.0;',
  'pub var PATH_DEFAULT_MAX_SPAN: int = 64;',
  'pub var PATH_SMOOTH_SAMPLE_STEP: float = 0.25;',
  'pub var PLAYER_BODY_RADIUS: float = 0.5;',
  'pub var PLAYER_MAX_CLIMB_SLOPE: float = 1.5;',
  'pub var PLAYER_SWIM_DEPTH: float = 0.8;',
  'collisionSweep.crossesBuiltinFence(',
  'var steps = <int>math.ceil(distance / PATH_SMOOTH_SAMPLE_STEP);',
  'if (stepLength > 0.000001 && rise > 0.0 && rise / stepLength > options.maxClimbSlope)',
  'if (width > options.maxSpan || height > options.maxSpan)',
  'var canUseDiagonal = deltaX == 0 || deltaZ == 0 ||',
  'smoothRoute(raw, options, output);',
  'pub findBuiltinPlayerPath(',
  'pub resolveBuiltinPlayerDestination(',
  'while (ring <= 24) {',
  'var samples = maximumInt(12, <int>math.ceil(radius * 10.0));',
]) {
  invariant(projection.includes(needle), `WOC pathfind projection is missing: ${needle}`);
}

assertOrder(projection, [
  'segmentWalkable(',
  'smoothRoute(',
  'pub findBuiltinPath(',
  'pub findBuiltinPlayerPath(',
  'pub resolveBuiltinPlayerDestination(',
]);

for (const needle of [
  '%import("world/pathfind_state")',
  'pathfind.findBuiltinPath(1000.0, 1000.0, 1008.0, 1005.0, openOptions, route);',
  'pathfind.findBuiltinPlayerPath(seed, -4.0, 2.0, 4.0, 2.0, 128, false, false, route);',
  'pathfind.resolveBuiltinPlayerDestination(seed, -108.0, 84.0, false, walked);',
  'pathfind.resolveBuiltinPlayerDestination(seed, -108.0, 84.0, true, swum);',
  'pathfind.findBuiltinPlayerPath(seed, 13.0, 7.0, 25.0, 13.0, 128, false, false, route);',
  'pathfind.findBuiltinPlayerPath(seed, 0.0, 0.0, 200.0, 0.0, 64, false, false, route);',
]) {
  invariant(testMain.includes(needle), `WOC pathfind contract is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m3_pathfind_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m3-pathfind-state-tests' &&
    testProject.entry === 'world/pathfind_state_test_main',
  'pathfind test project contract drifted',
);

process.stdout.write(`checked M3 pathfind source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function assertOrder(text, needles) {
  let prior = -1;
  for (const needle of needles) {
    const position = text.indexOf(needle);
    invariant(position >= 0, `missing ordered rule: ${needle}`);
    invariant(position > prior, `source order drifted at: ${needle}`);
    prior = position;
  }
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
