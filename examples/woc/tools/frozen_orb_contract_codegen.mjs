import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ORB_PATH = 'src/sim/combat/frozen_orb.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'frozen_orb_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'frozen_orb_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blobs = Object.fromEntries([ORB_PATH, TYPES_PATH].map((path) => [path, sourceBlob(path)]));
  const orb = blobs[ORB_PATH].toString('utf8');
  const types = blobs[TYPES_PATH].toString('utf8');
  const speed = Number(capture(orb, /export const FROZEN_ORB_SPEED\s*=\s*(\d+);/, 'Frozen Orb speed')[1]);
  const tickRate = Number(capture(types, /export const TICK_RATE\s*=\s*(\d+);/, 'simulation tick rate')[1]);
  invariant(types.includes('export const DT = 1 / TICK_RATE;'), 'simulation DT definition drifted');
  invariant(orb.includes('pulseTimer: eff.interval,') && orb.includes('firstHitDone: false,') && orb.includes('halted: false,'), 'Frozen Orb spawn state drifted');
  invariant(orb.includes('const latched = hasOrbContact(ctx, orb, source);') && orb.includes('if (latched !== orb.halted)') && orb.includes('if (!orb.halted)'), 'Frozen Orb latch transition drifted');
  invariant(orb.includes('orb.remaining -= DT;') && orb.includes('orb.pulseTimer -= DT;') && orb.includes('if (orb.pulseTimer <= 0) {') && orb.includes('orb.pulseTimer += orb.interval;') && orb.includes('if (orb.remaining <= 0)'), 'Frozen Orb tick ordering drifted');
  const document = { schema_version: 1, source_commit: SOURCE_COMMIT, generated_by: 'examples/woc/tools/frozen_orb_contract_codegen.mjs', source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])), tick: { speed_yards_per_second: speed, dt: 1 / tickRate, pulse: 'at_most_once_then_add_interval', expiration: 'after_pulse' }, latch: { holds_position: true, lifetime_continues: true } };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Frozen Orb JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Frozen Orb Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Frozen Orb contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) { return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` + `pub speed(required: bool): float { return required ? ${document.tick.speed_yards_per_second}.0 : 0.0; }\n` + `pub dt(required: bool): float { return required ? ${document.tick.dt} : 0.0; }\n`; }
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:frozen-orb-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:frozen-orb-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
