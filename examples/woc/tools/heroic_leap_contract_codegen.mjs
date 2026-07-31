import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const LEAP_PATH = 'src/sim/combat/heroic_leap.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'heroic_leap_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'heroic_leap_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blobs = Object.fromEntries([LEAP_PATH, TYPES_PATH].map((path) => [path, sourceBlob(path)]));
  const leap = blobs[LEAP_PATH].toString('utf8');
  const types = blobs[TYPES_PATH].toString('utf8');
  const duration = Number(capture(leap, /const FLIGHT_DURATION\s*=\s*([\d.]+);/, 'Heroic Leap duration')[1]);
  const apex = Number(capture(leap, /const FLIGHT_APEX\s*=\s*([\d.]+);/, 'Heroic Leap apex')[1]);
  const epsilon = Number(capture(leap, /const EXTERNAL_RELOCATION_EPSILON\s*=\s*([\d.]+);/, 'Heroic Leap relocation epsilon')[1]);
  const rate = Number(capture(types, /export const TICK_RATE\s*=\s*(\d+);/, 'simulation tick rate')[1]);
  invariant(types.includes('export const DT = 1 / TICK_RATE;'), 'simulation DT definition drifted');
  invariant(leap.includes('const progress = Math.min(1, elapsed / flight.duration);') && leap.includes('flight.apex * 4 * progress * (1 - progress)'), 'Heroic Leap parabolic path drifted');
  invariant(leap.includes('Math.hypot(entity.pos.x - expected.x, entity.pos.y - expected.y, entity.pos.z - expected.z) >') && leap.includes('if (entity.dead || wasExternallyRelocated(entity))'), 'Heroic Leap interruption gate drifted');
  invariant(leap.includes('flight.elapsed += DT;') && leap.includes('if (flight.elapsed < flight.duration) return true;') && leap.includes('entity.jumping = false;'), 'Heroic Leap tick/landing order drifted');
  const document = { schema_version: 1, source_commit: SOURCE_COMMIT, generated_by: 'examples/woc/tools/heroic_leap_contract_codegen.mjs', source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])), flight: { duration_seconds: duration, apex: apex, external_relocation_epsilon: epsilon, dt: 1 / rate, landing_after_elapsed_reaches_duration: true } };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Heroic Leap JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Heroic Leap Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Heroic Leap contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) { return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` + `pub duration(required: bool): float { return required ? ${document.flight.duration_seconds} : 0.0; }\n` + `pub apex(required: bool): float { return required ? ${document.flight.apex} : 0.0; }\n` + `pub relocationEpsilon(required: bool): float { return required ? ${document.flight.external_relocation_epsilon} : 0.0; }\n` + `pub dt(required: bool): float { return required ? ${document.flight.dt} : 0.0; }\n`; }
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:heroic-leap-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:heroic-leap-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
