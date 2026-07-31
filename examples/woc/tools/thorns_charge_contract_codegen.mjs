import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const THORNS_PATH = 'src/sim/combat/thorns_charge.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'thorns_charge_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'thorns_charge_contract.zr');
const checkOnly = process.argv.includes('--check');

const blobs = Object.fromEntries([THORNS_PATH, TYPES_PATH].map((path) => [path, sourceBlob(path)]));
const thorns = blobs[THORNS_PATH].toString('utf8');
const types = blobs[TYPES_PATH].toString('utf8');
const tickRate = literal(types, /export const TICK_RATE\s*=\s*(\d+);/, 'simulation tick rate');
const castCompleteEps = literal(types, /export const CAST_COMPLETE_EPS\s*=\s*([\d.eE+-]+);/, 'cast completion epsilon');

invariant(types.includes('export const DT = 1 / TICK_RATE;'), 'simulation DT definition drifted');
invariant(thorns.includes('return a.charges === undefined || a.charges > 0;'), 'thorns charge predicate drifted');
invariant(thorns.includes('return a.charges !== undefined && a.charges <= 0;'), 'thorns depletion predicate drifted');
invariant(thorns.includes('if (a.icd && a.icd > 0) a.icd = Math.max(0, a.icd - DT);'), 'thorns cooldown tick drifted');
invariant(thorns.includes('if ((a.icd ?? 0) > CAST_COMPLETE_EPS) return false;') && thorns.includes('if (a.charges !== undefined) a.charges -= 1;') && thorns.includes('if (a.icdMax) a.icd = a.icdMax;'), 'thorns consume ordering drifted');
invariant(thorns.includes("a.kind === 'thorns' && consumeThornsCharge(a)") && thorns.includes("a.kind === 'thorns' && thornsDepleted(a)"), 'thorns reaction filtering drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/thorns_charge_contract_codegen.mjs',
  source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
  thorns_kind: 'thorns',
  tick_rate: tickRate,
  dt: 1 / tickRate,
  cast_complete_eps: castCompleteEps,
  charge_model: 'undefined_is_unlimited; consume_before_reflect; depleted_removed_in_reverse_index_order',
};

writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'thorns charge JSON contract');
writeOrCheck(zrOutput, renderZr(document), 'thorns charge Zr contract');
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} thorns charge contract for ${SOURCE_COMMIT}\n`);

function renderZr(contract) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub thornsKind(): string { return \"${contract.thorns_kind}\"; }\n` +
    `pub dt(): float { return ${contract.dt}; }\n` +
    `pub castCompleteEps(): float { return ${Number(contract.cast_complete_eps)}; }\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function literal(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return Number(match[1]); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:thorns-charge-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:thorns-charge-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
