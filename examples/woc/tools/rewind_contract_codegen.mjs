import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CLASSES_PATH = 'src/sim/content/classes.ts';
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
  const blobs = Object.fromEntries([CLASSES_PATH, HISTORY_PATH, REWIND_PATH].map((path) => [path, sourceBlob(path)]));
  const classes = blobs[CLASSES_PATH].toString('utf8');
  const history = blobs[HISTORY_PATH].toString('utf8');
  const rewind = blobs[REWIND_PATH].toString('utf8');
  const abilityBlock = capture(
    classes,
    /temporal_rewind:\s*\{([\s\S]*?)\r?\n\s*\},\r?\n\s*\/\/ Hourglass of Suspension/,
    'Rewind ability definition',
  )[1];
  const abilityId = capture(abilityBlock, /id:\s*'([^']+)'/, 'Rewind ability id')[1];
  const abilityClass = capture(abilityBlock, /class:\s*'([^']+)'/, 'Rewind ability class')[1];
  const learnLevel = Number(capture(abilityBlock, /learnLevel:\s*(\d+)/, 'Rewind learn level')[1]);
  const resourceCost = Number(capture(abilityBlock, /cost:\s*(\d+)/, 'Rewind resource cost')[1]);
  const castTime = Number(capture(abilityBlock, /castTime:\s*([\d.]+)/, 'Rewind cast time')[1]);
  const cooldown = Number(capture(abilityBlock, /cooldown:\s*(\d+)/, 'Rewind cooldown')[1]);
  const range = Number(capture(abilityBlock, /range:\s*([\d.]+)/, 'Rewind range')[1]);
  const requiresTarget = capture(abilityBlock, /requiresTarget:\s*(true|false)/, 'Rewind target requirement')[1] === 'true';
  const effect = capture(
    abilityBlock,
    /effects:\s*\[\{\s*type:\s*'rewind',\s*fraction:\s*([\d.]+),\s*maxHpFraction:\s*([\d.]+),\s*windowSec:\s*(\d+),\s*radius:\s*([\d.]+)\s*}\s*]/,
    'Rewind effect definition',
  );
  const window = capture(history, /export const REWIND_WINDOW_SEC\s*=\s*(\d+);/, 'Rewind window seconds');
  const tickRate = capture(history, /REWIND_WINDOW_TICKS\s*=\s*REWIND_WINDOW_SEC\s*\*\s*(\d+);/, 'Rewind tick rate');
  const fraction = capture(rewind, /fraction\s*=\s*([\d.]+),\s*\r?\n\s*maxHpFraction\s*=\s*([\d.]+),/, 'Rewind fractions');
  invariant(abilityId === 'temporal_rewind' && abilityClass === 'mage', 'Rewind source identity drifted');
  invariant(!requiresTarget && castTime === 0 && range === 0, 'Rewind cast surface drifted');
  invariant(Number(effect[1]) === Number(fraction[1]) && Number(effect[2]) === Number(fraction[2]), 'Rewind effect and combat fractions drifted');
  invariant(Number(effect[3]) === Number(window[1]), 'Rewind effect window drifted');
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
    ability: {
      id: abilityId,
      class: abilityClass,
      learn_level: learnLevel,
      resource_cost: resourceCost,
      cast_time_seconds: castTime,
      cooldown_seconds: cooldown,
      range,
      requires_target: requiresTarget,
      radius: Number(effect[4]),
    },
    damage_history: { window_seconds: Number(window[1]), tick_rate: Number(tickRate[1]), window_ticks: Number(window[1]) * Number(tickRate[1]), prune_excludes_cutoff: true, query_includes_only_after_cutoff: true },
    healing: { fraction: Number(fraction[1]), fraction_percent: Math.round(Number(fraction[1]) * 100), max_hp_fraction: Number(fraction[2]), max_hp_fraction_percent: Math.round(Number(fraction[2]) * 100), rounding: 'Math.round', clamps: ['zero', 'max_hp_cap', 'missing_hp'] },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Rewind JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Rewind Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Rewind contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub learnLevel(required: bool): int { return required ? ${document.ability.learn_level} : 0; }\n` +
    `pub resourceCost(required: bool): int { return required ? ${document.ability.resource_cost} : 0; }\n` +
    `pub castTimeSeconds(required: bool): float { return required ? ${zrFloat(document.ability.cast_time_seconds)} : 0.0; }\n` +
    `pub cooldownSeconds(required: bool): int { return required ? ${document.ability.cooldown_seconds} : 0; }\n` +
    `pub range(required: bool): float { return required ? ${zrFloat(document.ability.range)} : 0.0; }\n` +
    `pub radius(required: bool): float { return required ? ${zrFloat(document.ability.radius)} : 0.0; }\n` +
    `pub windowSeconds(required: bool): int { return required ? ${document.damage_history.window_seconds} : 0; }\n` +
    `pub tickRate(required: bool): int { return required ? ${document.damage_history.tick_rate} : 0; }\n` +
    `pub windowTicks(required: bool): int { return required ? ${document.damage_history.window_ticks} : 0; }\n` +
    `pub fractionPercent(required: bool): int { return required ? ${document.healing.fraction_percent} : 0; }\n` +
    `pub maxHpFractionPercent(required: bool): int { return required ? ${document.healing.max_hp_fraction_percent} : 0; }\n`;
}

function zrFloat(value) { return Number.isInteger(value) ? `${value}.0` : String(value); }

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:rewind-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:rewind-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
