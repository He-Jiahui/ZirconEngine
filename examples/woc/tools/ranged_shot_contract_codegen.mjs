import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/ranged_shot.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'ranged_shot_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'ranged_shot_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
invariant(source.includes('if (ranged.wand) return { min: ranged.min, max: ranged.max, speed: ranged.speed };'), 'wand ranged-profile selection drifted');
invariant(source.includes('return { min: weapon.min, max: weapon.max, speed: weapon.speed };'), 'weapon ranged-profile selection drifted');
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/ranged_shot_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  selection: 'wand_uses_class_profile; otherwise_uses_carried_weapon_profile',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\npub wandUsesClassProfile(): bool { return true; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'ranged shot JSON contract'], [zrOutput, zr, 'ranged shot Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:ranged-shot-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:ranged-shot-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} ranged shot contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
