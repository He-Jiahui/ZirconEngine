import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/deeds_completion.ts');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const projection = readFileSync(
  resolve(wocSourceRoot, 'src', 'progression', 'deed_completion_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocSourceRoot, 'src', 'progression', 'deed_completion_state_test_main.zr'),
  'utf8',
);
const lifecycle = readFileSync(resolve(wocSourceRoot, 'src', 'main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(
  resolve(wocSourceRoot, 'woc_m5_deed_completion_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  'export function countsTowardCompletion(def: DeedDef, earned: boolean): boolean {',
  'if (def.feat === true) return false;',
  'if (def.hidden === true && !earned) return false;',
  'const def = deeds[id];',
  'if (!def) continue;',
  'const has = earnedIds.has(id);',
  'if (!countsTowardCompletion(def, has)) continue;',
  'total++;',
  'if (has) earned++;',
]) {
  invariant(source.includes(needle), `missing pinned deed-completion behavior: ${needle}`);
}

for (const needle of [
  'pub class DeedCompletionState',
  'pub append(',
  'pub setEarned(',
  'pub countsTowardCompletion(',
  '!<bool>state.defined[index]',
  'if (<bool>state.feat[index])',
  'if (<bool>state.hidden[index] && !<bool>state.earned[index])',
  'pub total(',
  'pub earnedTotal(',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `deed-completion projection omitted: ${needle}`);
}

for (const needle of [
  '%import("progression/deed_completion_state")',
  'deeds.contractTest()',
]) {
  invariant(testMain.includes(needle), `missing M5 deed test entry behavior: ${needle}`);
  invariant(lifecycle.includes(needle), `main lifecycle omitted deed self-test: ${needle}`);
}
invariant(
  testProject.name === 'woc_m5_deed_completion_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m5-deed-completion-state-tests' &&
    testProject.entry === 'progression/deed_completion_state_test_main',
  'M5 deed-completion test project contract drifted',
);

process.stdout.write(`checked M5 deed-completion source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
