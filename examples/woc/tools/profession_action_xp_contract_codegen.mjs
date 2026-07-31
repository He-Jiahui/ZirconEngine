import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const PROFESSION_XP_SOURCE_PATH = 'src/sim/professions/profession_xp.ts';
const TYPES_SOURCE_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'profession_action_xp_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'profession_action_xp_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const professionXp = sourceBlob(PROFESSION_XP_SOURCE_PATH);
  const types = sourceBlob(TYPES_SOURCE_PATH);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/profession_action_xp_contract_codegen.mjs',
    source_blobs: {
      [PROFESSION_XP_SOURCE_PATH]: sha256(professionXp),
      [TYPES_SOURCE_PATH]: sha256(types),
    },
    gather_xp_base: integerConstant(professionXp, 'GATHER_XP_BASE'),
    gather_xp_per_level: integerConstant(professionXp, 'GATHER_XP_PER_LEVEL'),
    craft_xp_base: integerConstant(professionXp, 'CRAFT_XP_BASE'),
    craft_xp_per_level: integerConstant(professionXp, 'CRAFT_XP_PER_LEVEL'),
    zero_diff_bands: zeroDiffBands(types),
    higher_level_bonus_per_level: 0.05,
    higher_level_bonus_cap: 4,
    semantics: {
      higher_or_equal: 'round base XP after a 5 percent per-level bonus capped at four levels',
      lower_level: 'linearly reduce base XP until zeroDiff(playerLevel) levels below; gray content yields zero',
      scope: 'pure character XP calculation for one gather or successful craft; it does not mutate craft skill or progression state',
    },
  };
  for (const needle of [
    'if (diff >= 0) {',
    'return Math.round(base * (1 + 0.05 * Math.min(diff, 4)));',
    'if (-diff >= zd) return 0;',
    'return Math.round(base * (1 - -diff / zd));',
  ]) {
    invariant(professionXp.includes(needle), 'profession action XP source drifted: ' + needle);
  }
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'profession action XP JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'profession action XP Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' profession action XP contract for ' + SOURCE_COMMIT + '\n');
}

function integerConstant(source, name) {
  const match = source.match(new RegExp('const ' + name + ' = (\\d[\\d_]*);'));
  invariant(match, 'profession action XP source no longer exposes ' + name);
  return Number(match[1].replaceAll('_', ''));
}

function zeroDiffBands(source) {
  const start = source.indexOf('export function zeroDiff(playerLevel: number): number {');
  invariant(start >= 0, 'types source no longer exposes zeroDiff');
  const end = source.indexOf('\n  }', start);
  invariant(end >= 0, 'zeroDiff source is unterminated');
  const body = source.slice(start, end);
  const bands = [...body.matchAll(/if \(playerLevel <= (\d+)\) return (\d+);/g)]
    .map((match) => ({ max_player_level: Number(match[1]), zero_diff: Number(match[2]) }));
  const returns = [...body.matchAll(/return (\d+);/g)];
  const fallback = returns.length > 0 ? returns[returns.length - 1] : null;
  invariant(bands.length === 3 && fallback, 'zeroDiff source shape drifted');
  return { bands, fallback_zero_diff: Number(fallback[1]) };
}

function renderZr(document) {
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub gatherXpBase(): int { return ' + document.gather_xp_base + '; }',
    'pub gatherXpPerLevel(): int { return ' + document.gather_xp_per_level + '; }',
    'pub craftXpBase(): int { return ' + document.craft_xp_base + '; }',
    'pub craftXpPerLevel(): int { return ' + document.craft_xp_per_level + '; }',
    'pub higherLevelBonusPerLevel(): float { return ' + document.higher_level_bonus_per_level + '; }',
    'pub higherLevelBonusCap(): int { return ' + document.higher_level_bonus_cap + '; }',
    'pub zeroDiff(playerLevel: int): int {',
  ];
  document.zero_diff_bands.bands.forEach((band) => {
    lines.push('    if (playerLevel <= ' + band.max_player_level + ') return ' + band.zero_diff + ';');
  });
  lines.push('    return ' + document.zero_diff_bands.fallback_zero_diff + ';');
  lines.push('}');
  return lines.join('\n') + '\n';
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
