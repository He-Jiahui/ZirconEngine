import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/dot_mutation.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'dot_mutation_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'dot_mutation_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
invariant(source.includes("aura.kind === 'dot' && aura.id === dotId && aura.sourceId === sourceId"), 'owned DoT lookup drifted');
invariant(source.includes('const alreadyExtended = dot.extendedBy ?? 0;') && source.includes('const extension = Math.min(seconds, maxBonus - alreadyExtended);') && source.includes('if (extension <= 0) return 0;'), 'DoT extension cap drifted');
invariant(source.includes('dot.extendedBy = alreadyExtended + extension;') && source.includes('dot.remaining += extension;') && source.includes('dot.duration += extension;'), 'DoT mutation ordering drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/dot_mutation_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  aura_kind: 'dot',
  selection: 'first_target_aura_matching_kind_id_and_source',
  extension: 'min(seconds,maxBonus-extendedBy_or_zero); only_positive; mutate_extendedBy_remaining_duration',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\npub dotKind(): string { return \"${document.aura_kind}\"; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'DoT mutation JSON contract'], [zrOutput, zr, 'DoT mutation Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:dot-mutation-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:dot-mutation-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} DoT mutation contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
