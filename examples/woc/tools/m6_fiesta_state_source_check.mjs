import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';

const COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = resolve('..', '..', '..', 'dev', 'world-of-claudecraft');
const source = execFileSync('git', ['-C', root, 'show', `${COMMIT}:src/sim/social/fiesta.ts`], { encoding: 'utf8' });
const wocRoot = resolve('..', 'scripts', 'woc_game', 'src');
const projection = readFileSync(resolve(wocRoot, 'social', 'fiesta_state.zr'), 'utf8');

for (const needle of [
  'export const FIESTA_SCORE_LIMIT = 15;',
  'export const FIESTA_FIRST_WAVE_AT = 8;',
  'export const FIESTA_RESPAWN_BASE = 3;',
  'export const FIESTA_RESPAWN_MAX = 14;',
  'export const FIESTA_RING_DPS_PCT = 0.06;',
  'export const FIESTA_POWERUP_TELEGRAPH = 5;',
  'export const FIESTA_POWERUP_TTL = 18;',
  'rng: new Rng((ctx.tickCount * 2654435761 + ctx.nextArenaMatchId * 40503) >>> 0),',
  'if (ctx.tickCount % 10 !== 0) return;',
  'if (f.powerups.length < FIESTA_POWERUP_MAX) fiestaSpawnPowerup(match);',
]) invariant(source.includes(needle), `missing pinned Fiesta behavior: ${needle}`);

for (const needle of [
  'delete e.queuedOnSwingCostMultiplier;',
  'if (e.leap !== undefined) e.leap = null;',
]) invariant(source.includes(needle), `missing current Fiesta death-reset behavior: ${needle}`);

for (const needle of [
  '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a',
  'pub var queuedSwingCostMultiplierPresent:',
  'pub var leapActive:',
  'clearDeathCarryover(state, player);',
  'fiestaCurrentHeadDeathResetTest(): int',
]) invariant(projection.includes(needle), `Fiesta projection omitted current-head reset: ${needle}`);

const importers = zrFiles(wocRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("social/fiesta_state")'))
  .map((path) => relative(wocRoot, path).replaceAll('\\', '/'))
  .sort();
if (JSON.stringify(importers) !== JSON.stringify(['social/fiesta_state_test_main.zr', 'social/m6_scenario_matrix.zr'])) {
  throw new Error(`fiesta_state escaped the M6 fixture boundary: ${importers.join(', ')}`);
}
process.stdout.write(`checked M6 Fiesta state source: ${COMMIT.slice(0, 15)}\n`);

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function zrFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(directory, entry.name);
    return entry.isDirectory() ? zrFiles(path) : entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
