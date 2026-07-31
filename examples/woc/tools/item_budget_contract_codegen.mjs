import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/item_budget.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'item_budget_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'item_budget_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before item budget contracts');
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  const quality = objectLiterals(text, 'QUALITY_STAT_MULT');
  const slots = objectLiterals(text, 'SLOT_STAT_MULT');
  const statPerIlvl = literalNumber(text, 'STAT_PER_ILVL');
  invariant(JSON.stringify(Object.keys(quality)) === JSON.stringify(['poor', 'common', 'uncommon', 'rare', 'epic', 'legendary']) &&
    JSON.stringify(Object.keys(slots)) === JSON.stringify(['mainhand', 'offhand', 'chest', 'legs', 'helmet', 'shoulder', 'waist', 'gloves', 'feet', 'neck', 'ring', 'ring1', 'ring2']),
  'item budget quality/slot ladder drifted');
  invariant(text.includes("export const PRIMARY_STATS = ['str', 'agi', 'sta', 'int', 'spi']") &&
    text.includes('Math.max(0, Math.round(level * q * s * STAT_PER_ILVL))') &&
    text.includes('const order = [...parts].sort((a, b) => b.frac - a.frac);') &&
    text.includes('order[i % order.length].base += 1;'),
  'item budget primary-stat or largest-remainder semantics drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/item_budget_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    primary_stats: ['str', 'agi', 'sta', 'int', 'spi'],
    quality_stat_mult: quality,
    slot_stat_mult: slots,
    unknown_slot_stat_mult: 0.7,
    stat_per_ilvl: statPerIlvl,
    source_semantics: {
      budget: 'absent slot returns zero; unknown present slot uses 0.7; unknown quality uses zero; positive budget rounds to nearest integer',
      normalize: 'armor is excluded from masterwork primary-profile redistribution; equal fractional ties preserve primary stat order',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'item budget JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'item budget Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} item budget contract for ${SOURCE_COMMIT}\n`);
}

function objectLiterals(source, name) {
  const match = source.match(new RegExp(`export const ${name}: [\\s\\S]*?= \\{([\\s\\S]*?)\\n\\};`));
  invariant(match, `${name} is missing or no longer a literal object`);
  const result = {};
  for (const line of match[1].split('\n')) {
    const entry = line.match(/^\s*([a-z0-9]+):\s*([0-9.]+),/i);
    if (entry) result[entry[1]] = Number(entry[2]);
  }
  return result;
}
function literalNumber(source, name) { const match = source.match(new RegExp(`export const ${name} = ([0-9.]+);`)); invariant(match, `${name} is missing or no longer a literal`); return Number(match[1]); }
function renderZr(document) {
  const lines = [`// Generated from ${SOURCE_COMMIT}; do not edit by hand.`, `pub statPerIlvl(required: bool): float { return required ? ${document.stat_per_ilvl} : 0.0; }`, `pub unknownSlotMultiplier(required: bool): float { return required ? ${document.unknown_slot_stat_mult} : 0.0; }`, 'pub qualityMultiplier(quality: string): float {'];
  for (const [name, value] of Object.entries(document.quality_stat_mult)) lines.push(`    if (quality == "${name}") return ${value};`);
  lines.push('    return 0.0;', '}', 'pub slotMultiplier(slot: string): float {');
  for (const [name, value] of Object.entries(document.slot_stat_mult)) lines.push(`    if (slot == "${name}") return ${value};`);
  lines.push('    return unknownSlotMultiplier(true);', '}');
  return `${lines.join('\n')}\n`;
}
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:item-budget-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:item-budget-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
