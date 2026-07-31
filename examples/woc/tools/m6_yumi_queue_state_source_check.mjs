import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/social/yumi.ts');
const sourceTest = gitShow('tests/yumi_match.test.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_queue_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_queue_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_yumi_queue_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  'export function packYumiTeams(',
  'const a: ArenaQueueUnit[] = [];',
  'const b: ArenaQueueUnit[] = [];',
  'if (na + u.pids.length <= size) {',
  'a.push(u);',
  'else if (nb + u.pids.length <= size) {',
  'b.push(u);',
  'if (na === size && nb === size) return { a, b };',
  'return null;',
]) {
  invariant(source.includes(needle), `source Yumi queue rule drifted: ${needle}`);
}
for (const needle of [
  'premade of 3 fills team A; three solos fill team B',
  'a premade of 2 + solo per side',
  'not enough players: no match',
]) {
  invariant(sourceTest.includes(needle), `source Yumi queue test drifted: ${needle}`);
}

for (const needle of [
  'pub class YumiQueueUnit {',
  'pub class YumiPackedTeams {',
  'pub packYumiTeams(',
  'if (countA + unitSize <= teamSize) {',
  'else if (countB + unitSize <= teamSize) {',
  'if (countA == teamSize && countB == teamSize) { return true; }',
  'clearPacked(result);',
  'packedTeamBUnitIndex(packed, 2) != 3',
  'packedTeamAPid(packed, 2) != 5',
]) {
  invariant(projection.includes(needle), `WOC Yumi queue projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/yumi_queue_state")',
  'yumiQueue.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC Yumi queue test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_yumi_queue_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-yumi-queue-state-tests' &&
    testProject.entry === 'social/yumi_queue_state_test_main',
  'Yumi queue test project contract drifted',
);

process.stdout.write(`checked M6 Yumi queue source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
