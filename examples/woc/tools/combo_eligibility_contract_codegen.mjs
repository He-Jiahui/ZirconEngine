import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const COMBO_PATH = 'src/sim/professions/combo_eligibility.ts';
const WHEEL_PATH = 'src/sim/professions/wheel.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'combo_eligibility_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'combo_eligibility_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before combo eligibility contracts');
  const combo = sourceBlob(COMBO_PATH);
  const wheel = sourceBlob(WHEEL_PATH);
  const comboText = combo.toString('utf8');
  const wheelText = wheel.toString('utf8');
  const tierStep = literalNumber(wheelText, 'TIER_SKILL_STEP');
  invariant(comboText.includes('function sameUnorderedPair') &&
    comboText.includes("reason: 'not_attuned'") && comboText.includes("reason: 'wrong_pair'") &&
    comboText.includes("reason: 'tier_unmet'") && comboText.includes('[requirement.craftA, requirement.craftB].filter'),
  'combo eligibility ordering or reason contract drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/combo_eligibility_contract_codegen.mjs',
    source_blobs: { [COMBO_PATH]: sha256(combo), [WHEEL_PATH]: sha256(wheel) },
    tier_skill_step: tierStep,
    reasons: ['not_attuned', 'wrong_pair', 'tier_unmet'],
    source_semantics: {
      no_requirement: 'passes without inspecting identity or skill state',
      pair: 'activeArchetype and pairedMajor match craftA/craftB in either order',
      tiers: 'unmet crafts are emitted in requirement order after craftCeiling evaluates both required crafts',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'combo eligibility JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'combo eligibility Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} combo eligibility contract for ${SOURCE_COMMIT}\n`);
}

function literalNumber(source, name) { const match = source.match(new RegExp(`export const ${name} = (\\d+);`)); invariant(match, `${name} is missing or no longer a literal`); return Number(match[1]); }
function renderZr(document) { return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` + `pub tierSkillStep(required: bool): int { return required ? ${document.tier_skill_step} : 0; }\n`; }
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:combo-eligibility-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:combo-eligibility-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
