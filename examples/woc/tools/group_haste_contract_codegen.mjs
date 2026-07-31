import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/haste_burst.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'group_haste_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'group_haste_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  const id = capture(text, /export const SATED_ID\s*=\s*'([^']+)'/, 'Sated id')[1];
  const name = capture(text, /export const SATED_NAME\s*=\s*'([^']+)'/, 'Sated name')[1];
  const duration = Number(capture(text, /export const SATED_DURATION\s*=\s*(\d+);/, 'Sated duration')[1]);
  invariant(text.includes("if (p.exhaust && target.auras.some((a) => a.kind === 'sated')) continue;"), 'shared exhaustion eligibility drifted');
  invariant(text.includes("kind: 'buff_haste'") && text.includes('value: p.mult,'), 'attack haste aura shape drifted');
  invariant(text.includes("kind: 'buff_spellhaste'") && text.includes('value: p.mult - 1'), 'spell haste value semantics drifted');
  invariant(text.includes("kind: 'sated'") && text.includes('remaining: SATED_DURATION') && text.includes('duration: SATED_DURATION'), 'shared exhaustion aura shape drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/group_haste_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    sated: { id, name, duration_seconds: duration },
    semantics: { skip_sated_only_when_exhausting: true, attack_haste_value: 'multiplier', spell_haste_value: 'multiplier_minus_one' },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'group haste JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'group haste Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} group haste contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub satedId(required: bool): string { return required ? "${document.sated.id}" : ""; }\n` +
    `pub satedName(required: bool): string { return required ? "${document.sated.name}" : ""; }\n` +
    `pub satedDuration(required: bool): float { return required ? ${document.sated.duration_seconds}.0 : 0.0; }\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function capture(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return match; }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:group-haste-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:group-haste-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
