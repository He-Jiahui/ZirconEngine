import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/social/yumi.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_match_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'yumi_match_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_yumi_match_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  'timer: YUMI_COUNTDOWN,',
  'nextTeleportAt: YUMI_TELEPORT_EVERY,',
  'rng: new Rng((ctx.tickCount * 2654435761 + matchId * 40503) >>> 0),',
  'const a = Math.floor(rng.next() * points.length);',
  'const roll = rng.next();',
  'if (!y.suddenDeath && match.timer >= YUMI_SUDDEN_AT)',
  'y.nextTeleportAt += YUMI_TELEPORT_EVERY;',
  'ctx.tickCount % (YUMI_SUDDEN_BLEED_EVERY * TICK_RATE) === 0',
  'if (dieA && dieB && catA && catB)',
  'if (y.respawn.has(victim.id)) return;',
  'y.respawn.set(victim.id, YUMI_RESPAWN_SECONDS);',
]) {
  invariant(source.includes(needle), `source Yumi match state drifted: ${needle}`);
}

for (const needle of [
  'pub class YumiMatchState',
  'pub yumiMatchSeed(tick: uint, matchId: uint): uint',
  'pub initializeYumiMatch(',
  'pub pickYumiCells(',
  'pub advanceYumiMatch(state: YumiMatchState): int',
  'pub damageYumiCat(',
  'pub benchYumiPlayer(',
  'pulseSuddenDeathBleed(state);',
  'state.nextTeleportTick = state.nextTeleportTick + 1200;',
  'return nextYumiUnit(state) < 0.5 ? 1 : 2;',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC Yumi match projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/yumi_match_state")',
  'yumiMatch.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC Yumi match test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_yumi_match_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-yumi-match-state-tests' &&
    testProject.entry === 'social/yumi_match_state_test_main',
  'Yumi match test project contract drifted',
);

process.stdout.write(`checked M6 Yumi match state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
