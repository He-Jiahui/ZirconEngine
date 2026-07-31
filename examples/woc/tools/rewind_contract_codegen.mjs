import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const HISTORY_PATH = 'src/sim/combat/damage_history.ts';
const REWIND_PATH = 'src/sim/combat/rewind.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'rewind_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'rewind_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blobs = Object.fromEntries([HISTORY_PATH, REWIND_PATH].map((path) => [path, sourceBlob(path)]));
  const history = blobs[HISTORY_PATH].toString('utf8');
  const rewind = blobs[REWIND_PATH].toString('utf8');
  const window = capture(history, /export const REWIND_WINDOW_SEC\s*=\s*(\d+);/, 'Rewind window seconds');
  const tickRate = capture(history, /REWIND_WINDOW_TICKS\s*=\s*REWIND_WINDOW_SEC\s*\*\s*(\d+);/, 'Rewind tick rate');
  const fraction = capture(rewind, /fraction\s*=\s*([\d.]+),\s*\r?\n\s*maxHpFraction\s*=\s*([\d.]+),/, 'Rewind fractions');
  invariant(history.includes('if (amount <= 0) return;'), 'damage history must reject non-positive real HP loss');
  invariant(history.includes('history[drop].tick <= cutoff'), 'damage history prune boundary drifted');
  invariant(history.includes('entry.tick > cutoff'), 'damage history query boundary drifted');
  invariant(rewind.includes('Math.round(recentDamage * fraction)') && rewind.includes('Math.round(maxHp * maxHpFraction)'), 'Rewind rounding formula drifted');
  invariant(rewind.includes('livingGroupRaidInRadius') && rewind.includes('ctx.applyHeal(caster, ally, heal, abilityName, null, false)'), 'Rewind canonical target/heal route drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/rewind_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    damage_history: { window_seconds: Number(window[1]), tick_rate: Number(tickRate[1]), window_ticks: Number(window[1]) * Number(tickRate[1]), prune_excludes_cutoff: true, query_includes_only_after_cutoff: true },
    healing: { fraction: Number(fraction[1]), fraction_percent: Math.round(Number(fraction[1]) * 100), max_hp_fraction: Number(fraction[2]), max_hp_fraction_percent: Math.round(Number(fraction[2]) * 100), rounding: 'Math.round', clamps: ['zero', 'max_hp_cap', 'missing_hp'] },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Rewind JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Rewind Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Rewind contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub windowSeconds(required: bool): int { return required ? ${document.damage_history.window_seconds} : 0; }\n` +
    `pub tickRate(required: bool): int { return required ? ${document.damage_history.tick_rate} : 0; }\n` +
    `pub windowTicks(required: bool): int { return required ? ${document.damage_history.window_ticks} : 0; }\n` +
    `pub fractionPercent(required: bool): int { return required ? ${document.healing.fraction_percent} : 0; }\n` +
    `pub maxHpFractionPercent(required: bool): int { return required ? ${document.healing.max_hp_fraction_percent} : 0; }\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:rewind-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:rewind-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
