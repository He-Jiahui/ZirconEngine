import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/pvp/honor.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'pvp_honor_policy_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'pvp_honor_policy_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const rankedOneVOne = literal(source, /'1v1':\s*(\d+),/, '1v1 ranked honor');
  const rankedTwoVTwo = literal(source, /'2v2':\s*(\d+),/, '2v2 ranked honor');
  const fiestaKill = literal(source, /export const FIESTA_KILL_HONOR = (\d+);/, 'Fiesta kill honor');
  const fiestaCompletion = literal(source, /export const FIESTA_COMPLETION_HONOR = (\d+);/, 'Fiesta completion honor');
  const fiestaWinBonus = literal(source, /export const FIESTA_WIN_BONUS_HONOR = (\d+);/, 'Fiesta win bonus honor');
  const arenaTaperStart = literal(source, /export const ARENA_DAILY_TAPER_START = (\d+);/, 'arena taper start');
  const arenaTaperFloorStart = literal(source, /export const ARENA_DAILY_TAPER_FLOOR_START = (\d+);/, 'arena taper floor start');
  for (const needle of [
    'export const ARENA_REPEAT_DR = [1, 0] as const;',
    'export const HONOR_REPEAT_DR = [1, 0.5, 0.25, 0] as const;',
    'export function repeatHonorMultiplier(previousAwards: number): number {',
    'export function arenaRepeatHonorMultiplier(previousAwards: number): number {',
    'if (totalWins < ARENA_DAILY_TAPER_START) return 1;',
    'if (totalWins < ARENA_DAILY_TAPER_FLOOR_START) return 0.5;',
    "return grantHonor(ctx, meta, FIESTA_KILL_HONOR * repeatHonorMultiplier(repeats), 'fiesta_kill');",
  ]) {
    invariant(source.includes(needle), 'PvP honor policy source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/pvp_honor_policy_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    ranked_arena_win_honor: { one_v_one: rankedOneVOne, two_v_two: rankedTwoVTwo },
    fiesta_honor: { kill: fiestaKill, completion: fiestaCompletion, win_bonus: fiestaWinBonus },
    arena_repeat_dr: [1, 0],
    honor_repeat_dr: [1, 0.5, 0.25, 0],
    arena_daily_taper: {
      full_through_wins: arenaTaperStart - 1,
      half_through_wins: arenaTaperFloorStart - 1,
      floor_multiplier: 0.25,
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'PvP honor policy JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'PvP honor policy Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' PvP honor policy contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  return '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n' +
    'pub rankedOneVOneHonor(): int { return ' + document.ranked_arena_win_honor.one_v_one + '; }\n' +
    'pub rankedTwoVTwoHonor(): int { return ' + document.ranked_arena_win_honor.two_v_two + '; }\n' +
    'pub fiestaKillHonor(): int { return ' + document.fiesta_honor.kill + '; }\n' +
    'pub fiestaCompletionHonor(): int { return ' + document.fiesta_honor.completion + '; }\n' +
    'pub fiestaWinBonusHonor(): int { return ' + document.fiesta_honor.win_bonus + '; }\n' +
    'pub arenaDailyTaperStart(): int { return ' + (document.arena_daily_taper.full_through_wins + 1) + '; }\n' +
    'pub arenaDailyTaperFloorStart(): int { return ' + (document.arena_daily_taper.half_through_wins + 1) + '; }\n';
}

function literal(source, expression, label) {
  const match = source.match(expression);
  invariant(match, 'PvP honor policy source no longer exposes ' + label);
  return Number(match[1]);
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), label + ' is missing; run its generate script');
    invariant(readFileSync(path, 'utf8') === output, label + ' is stale; run its generate script');
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
