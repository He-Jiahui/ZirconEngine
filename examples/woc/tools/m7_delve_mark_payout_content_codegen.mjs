import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHash = '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff';
const contentHash = '9b022fbddfbf870db2ed25971badb7392c1f4cc219e978cdfa4a1ebc2c2b282b';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_mark_payout_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_mark_payout_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_mark_payout_content.zr');
const check = process.argv.includes('--check');

const output = spawnSync(process.execPath, [extractor], {
  encoding: 'utf8',
  env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: commit },
});
if (output.status !== 0) {
  throw new Error(output.stderr);
}
const content = JSON.parse(output.stdout);
const actualContentHash = createHash('sha256').update(JSON.stringify(content)).digest('hex');
if (actualContentHash !== contentHash) {
  throw new Error(`Delve Mark payout content drifted: ${actualContentHash}`);
}
const source = execFileSync('git', ['-C', sourceRoot, 'show', `${commit}:src/sim/delves/runs.ts`], {
  encoding: 'utf8',
});
if (createHash('sha256').update(source).digest('hex') !== sourceHash) {
  throw new Error('Delve Mark payout source drifted');
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_mark_payout_content_codegen.mjs',
  source_sha256: { 'src/sim/delves/runs.ts': sourceHash },
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const zr = `// Generated Delve Mark payout content.\npub firstFullClearLimit(required: bool): int {\n    if (!required) { throw \"woc Delve Mark payout is required\"; }\n    return 3;\n}\n\npub firstNormalMarks(required: bool): int {\n    if (!required) { throw \"woc Delve Mark payout is required\"; }\n    return 1;\n}\n\npub firstHeroicMarks(required: bool): int {\n    if (!required) { throw \"woc Delve Mark payout is required\"; }\n    return 2;\n}\n\npub repeatHeroicMarks(required: bool): int {\n    if (!required) { throw \"woc Delve Mark payout is required\"; }\n    return 1;\n}\n\npub repeatNormalProbability(required: bool): float {\n    if (!required) { throw \"woc Delve Mark payout is required\"; }\n    return 0.5;\n}\n\npub contractTest(): int {\n    return firstFullClearLimit(true) == 3 && firstNormalMarks(true) == 1 &&\n        firstHeroicMarks(true) == 2 && repeatHeroicMarks(true) == 1 &&\n        repeatNormalProbability(true) == 0.5 ? 1 : -1;\n}\n`;

for (const [path, value] of [
  [contractPath, json],
  [zrPath, zr],
]) {
  if (check) {
    if (!existsSync(path) || readFileSync(path, 'utf8') !== value) {
      throw new Error(`generated drift ${path}`);
    }
  } else {
    writeFileSync(path, value, 'utf8');
  }
}
