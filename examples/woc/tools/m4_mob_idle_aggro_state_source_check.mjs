import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const locomotion = gitShow('src/sim/mob/locomotion.ts');
const targeting = gitShow('src/sim/mob/targeting.ts');
const threat = gitShow('src/sim/threat.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'mob_idle_aggro_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'mob_idle_aggro_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m4_mob_idle_aggro_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  'export const MAX_AGGRO_RADIUS = 20;',
  'Math.max(',
  'Math.min(MAX_AGGRO_RADIUS, template.aggroRadius + (mob.level - e.level) * 1.5)',
  'if (e.dead) return;',
  'if (isTrivialTo(mob, e)) return;',
  'radius *= ctx.delveDetectMult(e);',
  'radius = stealthDetectionRadius(mob, e, radius);',
  'if (d < radius && d < detectedD)',
]) {
  invariant(locomotion.includes(needle), `source idle aggro drifted: ${needle}`);
}

for (const needle of [
  'const TRIVIAL_LEVEL_GAP = 10;',
  'return player.level - mob.level >= TRIVIAL_LEVEL_GAP;',
]) {
  invariant(targeting.includes(needle), `source trivial-con drifted: ${needle}`);
}

for (const needle of [
  'export const STEALTH_DETECTION_MULT = 0.25;',
  'export const STEALTH_DETECTION_PER_LEVEL = 0.08;',
  'export const STEALTH_DETECTION_MIN_MULT = 0.1;',
  'export const STEALTH_DETECTION_MAX_MULT = 1;',
]) {
  invariant(threat.includes(needle), `source stealth detection drifted: ${needle}`);
}

for (const needle of [
  'pub class IdleAggroCandidate',
  'pub class IdleAggroSelection',
  'pub idleAggroRadius(',
  'pub stealthDetectionMultiplier(',
  'pub isTrivialToPlayer(',
  'pub selectIdleAggroTarget(',
  'candidate.distanceSquared < radius * radius',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC idle aggro projection is missing: ${needle}`);
}

for (const needle of [
  '%import("world/mob_idle_aggro_state")',
  'aggro.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC idle aggro test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m4_mob_idle_aggro_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-idle-aggro-state-tests' &&
    testProject.entry === 'world/mob_idle_aggro_state_test_main',
  'idle aggro test project contract drifted',
);

process.stdout.write(`checked M4 idle aggro source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
