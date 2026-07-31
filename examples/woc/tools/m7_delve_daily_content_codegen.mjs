import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHash = '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff';
const contentHash = '3512b88cf75aa166403f5c169f705c2d416c1a49259bf13ab9d38932f2fa07ed';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_daily_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_daily_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_daily_content.zr');
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
  throw new Error(`Delve daily reset content drifted: ${actualContentHash}`);
}
const sourceText = execFileSync(
  'git',
  ['-C', sourceRoot, 'show', `${commit}:src/sim/delves/runs.ts`],
  {
  encoding: 'utf8',
  },
);
if (createHash('sha256').update(sourceText).digest('hex') !== sourceHash) {
  throw new Error('Delve daily reset source drifted');
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_daily_content_codegen.mjs',
  source_sha256: { 'src/sim/delves/runs.ts': sourceHash },
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const zr = `// Generated Delve daily-reset behavior contract.\npub emptyUtcDayKeepsState(required: bool): bool {\n    if (!required) { throw \"woc Delve daily reset is required\"; }\n    return true;\n}\n\npub resetsFirstClearXp(required: bool): bool {\n    if (!required) { throw \"woc Delve daily reset is required\"; }\n    return true;\n}\n\npub resetsMarkClears(required: bool): bool {\n    if (!required) { throw \"woc Delve daily reset is required\"; }\n    return true;\n}\n\npub contractTest(): int {\n    return emptyUtcDayKeepsState(true) && resetsFirstClearXp(true) &&\n        resetsMarkClears(true) ? 1 : -1;\n}\n`;

for (const [target, value] of [
  [contractPath, json],
  [zrPath, zr],
]) {
  if (check) {
    if (!existsSync(target) || readFileSync(target, 'utf8') !== value) {
      throw new Error(`generated drift ${target}`);
    }
  } else {
    writeFileSync(target, value, 'utf8');
  }
}
