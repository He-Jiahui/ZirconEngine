import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/ring_of_frost.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'ring_of_frost_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'ring_of_frost_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  const padding = Number(capture(text, /const SWEEP_QUERY_PADDING\s*=\s*(\d+);/, 'Ring of Frost sweep padding')[1]);
  invariant(text.includes('if (maxDistanceSq < innerRadius * innerRadius) return false;'), 'Ring of Frost inner-radius early-out drifted');
  invariant(text.includes('lengthSq > 0 ? Math.max(0, Math.min(1, -(startX * dx + startZ * dz) / lengthSq)) : 0;'), 'Ring of Frost closest-segment projection drifted');
  invariant(text.includes('return closestX * closestX + closestZ * closestZ <= outerRadius * outerRadius;'), 'Ring of Frost outer-radius boundary drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/ring_of_frost_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    annulus: { projection: 'clamped_segment', inner_boundary: 'strictly_outside_inner', outer_boundary: 'inclusive', zero_length_projection_t: 0 },
    sweep_query_padding: padding,
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Ring of Frost JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Ring of Frost Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Ring of Frost contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub sweepQueryPadding(required: bool): float { return required ? ${document.sweep_query_padding}.0 : 0.0; }\n` +
    'pub outerBoundaryInclusive(required: bool): bool { return required; }\n' +
    'pub zeroLengthProjectionT(required: bool): float { return required ? 0.0 : -1.0; }\n';
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:ring-of-frost-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:ring-of-frost-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
