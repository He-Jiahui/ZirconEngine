import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/glacial_front.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'glacial_front_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'glacial_front_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  const angle = Number(capture(text, /GLACIAL_FRONT_ANGLE_DEG\s*=\s*(\d+);/, 'Glacial Front angle')[1]);
  const minRange = Number(capture(text, /GLACIAL_FRONT_MIN_RANGE\s*=\s*(\d+);/, 'Glacial Front minimum range')[1]);
  const maxRange = Number(capture(text, /GLACIAL_FRONT_MAX_RANGE\s*=\s*(\d+);/, 'Glacial Front maximum range')[1]);
  const ranges = [...capture(text, /return \[([^\]]+)\]\[empoweredStageForProgress\(progress, 4\) - 1\];/, 'Glacial Front presentation ranges')[1].matchAll(/\d+/g)].map((match) => Number(match[0]));
  invariant(JSON.stringify(ranges) === JSON.stringify([7, 10, 13, 16]), 'Glacial Front presentation ranges drifted');
  invariant(text.includes('const clamped = Math.max(0, Math.min(1, progress));') && text.includes('Math.floor(clamped * stageCount) + 1'), 'Glacial Front stage clamp drifted');
  invariant(text.includes('return Math.max(0, Math.min(1, (total - remaining) / total));'), 'Glacial Front cast progress drifted');
  invariant(text.includes('Math.hypot(dx, dz) > range') && text.includes('Math.atan2(dx, dz) - facing') && text.includes('Math.abs(delta) <= halfAngle'), 'Glacial Front cone geometry drifted');
  const document = { schema_version: 1, source_commit: SOURCE_COMMIT, generated_by: 'examples/woc/tools/glacial_front_contract_codegen.mjs', source_blobs: { [SOURCE_PATH]: sha256(source) }, angle_degrees: angle, min_range: minRange, max_range: maxRange, presentation_ranges: ranges, stage_count: 4 };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Glacial Front JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Glacial Front Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Glacial Front contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) { return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` + `pub angleDegrees(required: bool): float { return required ? ${document.angle_degrees}.0 : 0.0; }\n` + `pub minRange(required: bool): float { return required ? ${document.min_range}.0 : 0.0; }\n` + `pub maxRange(required: bool): float { return required ? ${document.max_range}.0 : 0.0; }\n` + `pub presentationRange(stage: int): float {\n    if (stage == 1) return ${document.presentation_ranges[0]}.0;\n    if (stage == 2) return ${document.presentation_ranges[1]}.0;\n    if (stage == 3) return ${document.presentation_ranges[2]}.0;\n    return stage == 4 ? ${document.presentation_ranges[3]}.0 : 0.0;\n}\n`; }
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:glacial-front-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:glacial-front-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
