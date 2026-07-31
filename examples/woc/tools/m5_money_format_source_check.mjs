import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/format_money.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'progression', 'money_format_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'progression', 'money_format_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(
    resolve(wocRoot, 'scripts', 'woc_game', 'woc_m5_money_format_state_tests.zrp'),
    'utf8',
  ),
);

for (const needle of [
  'const g = Math.floor(copper / 10000);',
  'const s = Math.floor((copper % 10000) / 100);',
  'const c = copper % 100;',
  'if (g > 0) parts.push(`${g}g`);',
  'if (s > 0) parts.push(`${s}s`);',
  'if (c > 0 || parts.length === 0) parts.push(`${c}c`);',
  "return parts.join(' ');",
]) {
  invariant(source.includes(needle), `source money-format rule drifted: ${needle}`);
}

for (const needle of [
  'var gold = <int>math.floor(<float>copper / 10000.0);',
  'var silver = <int>math.floor(<float>(copper % 10000) / 100.0);',
  'var copperPart = copper % 100;',
  'if (gold > 0) {',
  'if (silver > 0) {',
  'if (copperPart > 0 || !hasPart) {',
  'return hasPart ? current + " " + fragment : fragment;',
]) {
  invariant(projection.includes(needle), `WOC money-format projection is missing: ${needle}`);
}

assertOrder(projection, [
  'var gold = <int>math.floor(<float>copper / 10000.0);',
  'var silver = <int>math.floor(<float>(copper % 10000) / 100.0);',
  'var copperPart = copper % 100;',
  'if (gold > 0) {',
  'if (silver > 0) {',
  'if (copperPart > 0 || !hasPart) {',
]);

for (const needle of [
  '%import("progression/money_format_state")',
  'money.formatMoney(123405) != "12g 34s 5c"',
  'money.formatMoney(-100) != "0c"',
  'money.formatMoney(-10001) != "-1c"',
]) {
  invariant(testMain.includes(needle), `WOC money-format contract is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m5_money_format_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m5-money-format-state-tests' &&
    testProject.entry === 'progression/money_format_state_test_main',
  'money-format test project contract drifted',
);

process.stdout.write(`checked M5 money-format source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

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
