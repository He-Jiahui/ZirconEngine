import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/professions/masterwork.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'masterwork_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'masterwork_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before masterwork contracts');
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  const constants = Object.fromEntries([
    'MASTERWORK_BASE_CHANCE',
    'MASTERWORK_PER_TIER_ABOVE_CHANCE',
    'MASTERWORK_SIGNED_CHANCE',
    'MASTERWORK_SPECIALIZATION_CHANCE',
    'MASTERWORK_CHANCE_CAP',
  ].map((name) => [name, literalNumber(text, name)]));
  invariant(text.includes("'common',") && text.includes("'uncommon',") && text.includes("'rare',") &&
    text.includes("'epic',") && text.includes("'legendary',"),
  'masterwork quality ladder drifted from the fixed five-tier contract');
  invariant(text.includes('const base = quality ?? \'common\';') &&
    text.includes('if (idx < 0) return null;') && text.includes('if (bumpedIdx >= MASTERWORK_QUALITY_LADDER.length) return null;'),
  'masterwork quality bump semantics drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/masterwork_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    chance: {
      base: constants.MASTERWORK_BASE_CHANCE,
      per_tier_above: constants.MASTERWORK_PER_TIER_ABOVE_CHANCE,
      signed_reagent: constants.MASTERWORK_SIGNED_CHANCE,
      specialization: constants.MASTERWORK_SPECIALIZATION_CHANCE,
      cap: constants.MASTERWORK_CHANCE_CAP,
    },
    quality_ladder: ['common', 'uncommon', 'rare', 'epic', 'legendary'],
    source_semantics: {
      draw_ownership: 'crafting.ts owns exactly one successful-craft proc draw; this pure module never draws RNG',
      quality: 'poor, legendary and unknown qualities cannot bump; absent quality normalizes to common',
      stats: 'baked primary-stat tier delta remains owned by masterworkBonusStats and item_budget integration',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'masterwork JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'masterwork Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} masterwork contract for ${SOURCE_COMMIT}\n`);
}

function literalNumber(source, name) {
  const match = source.match(new RegExp(`export const ${name} = ([0-9.]+);`));
  invariant(match, `${name} is missing or no longer a literal`);
  return Number(match[1]);
}

function renderZr(document) {
  const chance = document.chance;
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub baseChance(required: bool): float { return required ? ${chance.base} : 0.0; }\n` +
    `pub perTierAboveChance(required: bool): float { return required ? ${chance.per_tier_above} : 0.0; }\n` +
    `pub signedReagentChance(required: bool): float { return required ? ${chance.signed_reagent} : 0.0; }\n` +
    `pub specializationChance(required: bool): float { return required ? ${chance.specialization} : 0.0; }\n` +
    `pub chanceCap(required: bool): float { return required ? ${chance.cap} : 0.0; }\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:masterwork-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:masterwork-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
