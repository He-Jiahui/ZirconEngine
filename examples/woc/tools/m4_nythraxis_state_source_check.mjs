import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const encounter = gitShow('src/sim/encounters/nythraxis.ts');
const projectionRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const projection = readFileSync(resolve(projectionRoot, 'src', 'combat', 'nythraxis_state.zr'), 'utf8');
const testMain = readFileSync(
  resolve(projectionRoot, 'src', 'combat', 'nythraxis_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(readFileSync(
  resolve(projectionRoot, 'woc_m4_nythraxis_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  'const back = boss.spawnPos.z + 16;',
  'ctx.groundPos(boss.spawnPos.x - 12, back)',
  'ctx.groundPos(boss.spawnPos.x + 12, back)',
  'ctx.groundPos(boss.pos.x - 8, boss.pos.z + 8)',
  'ctx.groundPos(boss.pos.x, boss.pos.z + 10)',
  'ctx.groundPos(boss.pos.x + 8, boss.pos.z + 8)',
  'add.spawnPos = { ...boss.spawnPos };',
]) {
  invariant(encounter.includes(needle), `missing pinned Nythraxis add position behavior: ${needle}`);
}

for (const needle of [
  'pub var bossPositionX: float;', 'pub var bossSpawnX: float;',
  'pub var normalAddPositions: container.Array<float>;',
  'pub var normalAddAnchors: container.Array<float>;',
  'pub var heroicAddPositions: container.Array<float>;',
  'pub var heroicAddAnchors: container.Array<float>;',
  'recordNormalRaisedAddPositions(state);',
  'pub recordHeroicCourtAddPositions(',
  'state.bossSpawnX - 12.0, back, state.bossSpawnX, state.bossSpawnZ',
  'state.bossPositionX - 8.0, state.bossPositionZ + 8.0,',
  'state.bossPositionX, state.bossPositionZ + 10.0,',
  'state.bossPositionX + 8.0, state.bossPositionZ + 8.0,',
]) {
  invariant(projection.includes(needle), `Nythraxis add position projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("combat/nythraxis_state")') &&
    testMain.includes('nythraxis.contractTest()'),
  'missing Nythraxis state test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_nythraxis_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-nythraxis-state-tests' &&
    testProject.entry === 'combat/nythraxis_state_test_main',
  'Nythraxis state test project contract drifted',
);

process.stdout.write(`checked M4 Nythraxis add-anchor source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
