import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/projectile_travel.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'projectile_travel_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'projectile_travel_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m4_projectile_travel_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  'export const PROJECTILE_SPEED = 26;',
  'export const PROJECTILE_REACH = 0.7;',
  'export const PROJECTILE_MAX_FLIGHT = 3;',
  'const dist = Math.sqrt(dx * dx + dz * dz);',
  'if (dist <= Math.max(PROJECTILE_REACH, step))',
  'if (!source || source.dead || !target || target.dead) continue;',
  'const next = stepProjectile(proj.x, proj.z, target.pos.x, target.pos.z, step);',
  'if (next.hit) {',
  'proj.ttl -= DT;',
  'if (proj.ttl <= 0) {',
]) {
  invariant(source.includes(needle), `source projectile rule drifted: ${needle}`);
}

assertOrder(source, [
  'if (!source || source.dead || !target || target.dead) continue;',
  'const next = stepProjectile(proj.x, proj.z, target.pos.x, target.pos.z, step);',
  'if (next.hit) {',
  'proj.ttl -= DT;',
  'if (proj.ttl <= 0) {',
]);

for (const needle of [
  'pub projectileSpeed(): float {',
  'return 26.0;',
  'pub projectileReach(): float {',
  'return 0.7;',
  'pub projectileMaxFlight(): float {',
  'return 3.0;',
  'var distance = math.sqrt(deltaX * deltaX + deltaZ * deltaZ);',
  'if (distance <= arrivalDistance) {',
  'if (!sourceLive || !targetLive) {',
  'stepProjectile(state.x, state.z, targetX, targetZ, projectileStepDistance(), next);',
  'if (next.hit) {',
  'var remaining = state.ttl - projectileTickSeconds();',
  'if (remaining <= 0.0) {',
]) {
  invariant(projection.includes(needle), `WOC projectile projection is missing: ${needle}`);
}

assertOrder(projection, [
  'if (!sourceLive || !targetLive) {',
  'stepProjectile(state.x, state.z, targetX, targetZ, projectileStepDistance(), next);',
  'if (next.hit) {',
  'var remaining = state.ttl - projectileTickSeconds();',
  'if (remaining <= 0.0) {',
]);

for (const needle of [
  '%import("world/projectile_travel_state")',
  'fleeingTicks <= 14',
  'closingTicks >= 19',
  'runawayTicks <= 45 || runawayTicks >= 70',
]) {
  invariant(testMain.includes(needle), `WOC projectile contract is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m4_projectile_travel_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-projectile-travel-state-tests' &&
    testProject.entry === 'world/projectile_travel_state_test_main',
  'projectile travel test project contract drifted',
);

process.stdout.write(`checked M4 projectile travel source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

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
