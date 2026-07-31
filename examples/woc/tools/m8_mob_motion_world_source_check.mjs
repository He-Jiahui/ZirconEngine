import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sim = gitShow('src/sim/sim.ts');
const world = gitShow('src/sim/world.ts');
const playerMotion = gitShow('src/sim/player_motion.ts');
const types = gitShow('src/sim/types.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'mob_motion_world.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'mob_motion_world_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m8_mob_motion_world_tests.zrp'), 'utf8'),
);

for (const needle of [
  'if (!ignoreObstacles && MOBS[e.templateId]?.phasesThroughObstacles) ignoreObstacles = true;',
  'if (d < 0.3) return true;',
  'const step = Math.min(speed * DT, d);',
  'for (const off of MOVE_SLIDE_FAN)',
  'groundHeight(nx, nz, this.cfg.seed) < waterLevelAt(nx, nz) - SWIM_DEPTH',
  'if (nearSteepWalls(nx, nz) && terrainSteepnessAt(nx, nz, this.cfg.seed) > MAX_CLIMB_SLOPE)',
  'const r = this.resolveMovePoint(nx, nz, BODY_RADIUS, e);',
  'if (off === 0 && progress >= step - 1e-3) break;',
  'return dist2d(e.pos, dest) < 0.3;',
]) {
  invariant(sim.includes(needle), `source mob movement drifted: ${needle}`);
}

for (const needle of [
  'const RIDGE_SIGMA = 10;',
  'if (Math.abs(z - ridge.z) < RIDGE_SIGMA * 4) return true;',
]) {
  invariant(world.includes(needle), `source steep-wall predicate drifted: ${needle}`);
}

invariant(
  playerMotion.includes('return waterLevelAt(x, z) - 0.75;'),
  'source swim-surface contract drifted',
);
invariant(
  types.includes('return Math.atan2(to.x - from.x, to.z - from.z);'),
  'source bearing contract drifted',
);

for (const needle of [
  'pub class MobMotionState',
  'pub initializeMobMotion(',
  'pub currentSourceMobCanSwim(',
  'pub nearSteepWalls(',
  'pub slideFanOffset(',
  'pub stepMobToward(',
  'worldCollision.resolveSupportedWorldCoordinate(',
  'hasWaterAt(candidateX, candidateZ)',
  'playerMotion.swimSurfaceY(',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC mob movement projection is missing: ${needle}`);
}

for (const needle of [
  '%import("world/mob_motion_world")',
  'mobMotion.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC mob movement test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m8_mob_motion_world_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m8-mob-motion-world-tests' &&
    testProject.entry === 'world/mob_motion_world_test_main',
  'mob movement test project contract drifted',
);

process.stdout.write(`checked M8 mob motion source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
