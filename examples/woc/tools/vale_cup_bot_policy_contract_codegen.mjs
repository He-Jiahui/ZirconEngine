import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/social/vale_cup_bots.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'vale_cup_bot_policy_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'vale_cup_bot_policy_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const constants = Object.fromEntries([
    ['field_cast_period', 'BOT_CAST_PERIOD'],
    ['keeper_cast_period', 'BOT_KEEPER_CAST_PERIOD'],
    ['keeper_shade', 'BOT_KEEPER_SHADE'],
    ['keeper_guard_depth', 'VC_KEEPER_GUARD_DEPTH'],
    ['kick_reach', 'BOT_KICK_REACH'],
    ['sprint_ball_distance', 'BOT_SPRINT_BALL_DIST'],
    ['shoot_range', 'BOT_SHOOT_RANGE'],
    ['pass_min_lead', 'BOT_PASS_MIN_LEAD'],
    ['aim_error_max', 'BOT_AIM_ERROR_MAX'],
    ['aim_error_period', 'BOT_AIM_ERROR_PERIOD'],
    ['shoot_range_encoding', 'SPORT_SHOOT_RANGE'],
    ['intercept_lead', 'BOT_INTERCEPT_LEAD'],
  ].map(([key, name]) => [key, numberLiteral(source, name, name)]));

  for (const needle of [
    'function botRoleForSeat(seat: number, bracket: VcBracket): SportRole {',
    "if (bracket <= 2) return 'allrounder';",
    "if (seat === 0) return 'keeper';",
    "if (seat === bracket - 1) return 'sweeper';",
    "return 'striker';",
    'function botAimError(tick: number, pid: number): number {',
    'const phase = (tick + pid * 37) % BOT_AIM_ERROR_PERIOD;',
    'return (Math.abs(phase - half) / half) * 2 * BOT_AIM_ERROR_MAX - BOT_AIM_ERROR_MAX;',
    'const frac = Math.max(0.42, Math.min(0.6, 0.4 + distToMouth / 55));',
  ]) {
    invariant(source.includes(needle), 'Vale Cup bot policy source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/vale_cup_bot_policy_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    constants,
    role_codes: {
      allrounder: 0,
      keeper: 1,
      sweeper: 2,
      striker: 3,
    },
    source_formulas: {
      aim_phase_pid_multiplier: 37,
      aim_half_period_divisor: 2,
      shot_fraction_min: 0.42,
      shot_fraction_max: 0.6,
      shot_fraction_base: 0.4,
      shot_fraction_distance_divisor: 55,
      shot_goal_post_margin: 1,
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'Vale Cup bot policy JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Vale Cup bot policy Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' Vale Cup bot policy contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  for (const [name, value] of Object.entries(document.constants)) {
    lines.push(floatAccessor(name, value));
  }
  for (const [name, value] of Object.entries(document.role_codes)) {
    lines.push('pub role' + titleCase(name) + '(required: bool): int { return required ? ' + value + ' : 0; }\n');
  }
  for (const [name, value] of Object.entries(document.source_formulas)) {
    lines.push(floatAccessor(name, value));
  }
  return lines.join('');
}

function floatAccessor(name, value) {
  return 'pub ' + name + '(required: bool): float { return required ? ' + zrFloat(value) + ' : 0.0; }\n';
}

function titleCase(value) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}

function zrFloat(value) {
  return Number.isInteger(value) ? String(value) + '.0' : String(value);
}

function numberLiteral(source, name, label) {
  const declaration = source
    .split(/\r?\n/)
    .find((line) => line.includes('const ' + name + ' ='));
  const match = declaration
    ? declaration.match(/=\s*(-?(?:\d+(?:\.\d+)?|\.\d+))\s*;/)
    : null;
  invariant(match, label + ' is no longer a literal contract');
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
