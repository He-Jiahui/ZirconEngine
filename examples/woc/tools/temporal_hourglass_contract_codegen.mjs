import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const HOURGLASS_PATH = 'src/sim/combat/temporal_hourglass.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'temporal_hourglass_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'temporal_hourglass_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blobs = Object.fromEntries([HOURGLASS_PATH, TYPES_PATH].map((path) => [path, sourceBlob(path)]));
  const hourglass = blobs[HOURGLASS_PATH].toString('utf8');
  const types = blobs[TYPES_PATH].toString('utf8');
  const id = capture(hourglass, /export const TEMPORAL_HOURGLASS_ID\s*=\s*'([^']+)'/, 'Temporal Hourglass id')[1];
  const tickRate = Number(capture(types, /export const TICK_RATE\s*=\s*(\d+);/, 'simulation tick rate')[1]);
  invariant(types.includes('export const DT = 1 / TICK_RATE;'), 'simulation DT definition drifted');
  invariant(hourglass.includes("aura.id === TEMPORAL_HOURGLASS_ID && aura.kind === 'stasis'"), 'protective stasis identity drifted');
  invariant(hourglass.includes('if (abilityId === TEMPORAL_HOURGLASS_ID || !isProtectiveTemporalHourglass(entity)) return DT;') && hourglass.includes('return DT * (aura?.value ?? 1);'), 'Temporal Hourglass cooldown rule drifted');
  invariant(hourglass.includes('const planned = Math.ceil(remaining / ticks);') && hourglass.includes('aura.temporalHealRemaining = Math.max(0, remaining - planned);') && hourglass.includes('const healed = Math.min(planned, target.maxHp - target.hp);'), 'Temporal Hourglass healing schedule drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/temporal_hourglass_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    protective_stasis: { id, kind: 'stasis', default_cooldown_delta: 1 / tickRate },
    healing_schedule: { planned: 'ceil(remaining/ticks)', consumes_schedule_before_hp_clamp: true, clamps: ['zero_remaining', 'missing_hp'] },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Temporal Hourglass JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Temporal Hourglass Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Temporal Hourglass contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub id(required: bool): string { return required ? "${document.protective_stasis.id}" : ""; }\n` +
    'pub stasisKind(required: bool): string { return required ? "stasis" : ""; }\n' +
    `pub defaultCooldownDelta(required: bool): float { return required ? ${document.protective_stasis.default_cooldown_delta} : 0.0; }\n` +
    'pub consumesScheduleBeforeHpClamp(required: bool): bool { return required; }\n';
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:temporal-hourglass-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:temporal-hourglass-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
