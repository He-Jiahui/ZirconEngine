import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/social/yumi.ts');
const types = gitShow('src/sim/types.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_match_rules.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_match_rules_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_yumi_match_rules_tests.zrp'), 'utf8'),
);

for (const needle of [
  'export const YUMI_HP = 5000;',
  'export const YUMI_COUNTDOWN = 5;',
  'export const YUMI_TELEPORT_EVERY = 60;',
  'export const YUMI_RESPAWN_SECONDS = 10;',
  'export const YUMI_SUDDEN_AT = 600;',
  'export const YUMI_SUDDEN_STEP = 15;',
  'export const YUMI_SUDDEN_RAMP = 0.25;',
  'export const YUMI_SUDDEN_BLEED_PCT = 0.01;',
  'export const YUMI_SUDDEN_BLEED_EVERY = 2;',
  "return fmt === 'yumi3' ? 3 : 5;",
  'return 1 + Math.floor((timer - YUMI_SUDDEN_AT) / YUMI_SUDDEN_STEP);',
  'return n === 0 ? 1 : 1 + YUMI_SUDDEN_RAMP * n;',
  'const dmg = Math.ceil(YUMI_HP * YUMI_SUDDEN_BLEED_PCT * suddenStep(match.timer));',
  'let dmg = Math.round(amount * yumiTakenMult(match.timer));',
  'dmg = Math.min(dmg, cat.hp);',
  "if (hpA !== hpB) return hpA > hpB ? 'A' : 'B';",
  "if (dmgToYumiA !== dmgToYumiB) return dmgToYumiB > dmgToYumiA ? 'A' : 'B';",
  "return rng.next() < 0.5 ? 'A' : 'B';",
]) {
  invariant(source.includes(needle), `source Yumi rule drifted: ${needle}`);
}
invariant(types.includes('export const TICK_RATE = 20;'), 'source fixed tick rate drifted');

for (const needle of [
  'pub yumiTickRate(): int { return 20; }',
  'pub yumiSuddenStepForElapsedTicks(elapsedTicks: int): int {',
  'return 1 + <int>(<float>(elapsedTicks - yumiSuddenAtTicks()) / <float>yumiSuddenStepTicks());',
  'pub yumiTakenMultiplierForElapsedTicks(elapsedTicks: int): float {',
  'return 1.0 + 0.25 * <float>yumiSuddenStepForElapsedTicks(elapsedTicks);',
  'pub yumiSuddenBleedDamageForElapsedTicks(elapsedTicks: int): int {',
  'return 50 * yumiSuddenStepForElapsedTicks(elapsedTicks);',
  'pub yumiScaledDamage(amount: int, elapsedTicks: int): int {',
  'var quarters = 4 + yumiSuddenStepForElapsedTicks(elapsedTicks);',
  'pub yumiTiebreakWinner(',
  'yumiScaledDamage(2, 12000) != 3',
]) {
  invariant(projection.includes(needle), `WOC Yumi projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/yumi_match_rules")',
  'yumi.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC Yumi test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_yumi_match_rules_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-yumi-match-rules-tests' &&
    testProject.entry === 'social/yumi_match_rules_test_main',
  'Yumi test project contract drifted',
);

process.stdout.write(`checked M6 Yumi match rules source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
