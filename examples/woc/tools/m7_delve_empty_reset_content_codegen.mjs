import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHashes = {
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
  'src/sim/types.ts': '303321fb109f7bcce51a6871597be5b7b05f54ed1c6b86185387390cc47ccd90',
};
const contentHash = '081003adcfb2b2cd790666d824a0b53b7cd1111594e556e4e69f7eda2a7cab62';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_empty_reset_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_empty_reset_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_empty_reset_content.zr');
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
  throw new Error(`Delve empty reset content drifted: ${actualContentHash}`);
}
for (const [path, expectedHash] of Object.entries(sourceHashes)) {
  const source = execFileSync('git', ['-C', sourceRoot, 'show', `${commit}:${path}`], {
    encoding: 'utf8',
  });
  if (createHash('sha256').update(source).digest('hex') !== expectedHash) {
    throw new Error(`Delve empty reset source drifted: ${path}`);
  }
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_empty_reset_content_codegen.mjs',
  source_sha256: sourceHashes,
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const zr = `// Generated Delve instance empty-reset contract.\npub checkTickInterval(required: bool): int {\n    if (!required) { throw \"woc Delve empty reset is required\"; }\n    return 20;\n}\n\npub occupancyXRadius(required: bool): float {\n    if (!required) { throw \"woc Delve empty reset is required\"; }\n    return 120.0;\n}\n\npub emptyTimeoutSeconds(required: bool): int {\n    if (!required) { throw \"woc Delve empty reset is required\"; }\n    return 300;\n}\n\npub strictBounds(required: bool): bool {\n    if (!required) { throw \"woc Delve empty reset is required\"; }\n    return true;\n}\n\npub contractTest(): int {\n    return checkTickInterval(true) == 20 && occupancyXRadius(true) == 120.0 &&\n        emptyTimeoutSeconds(true) == 300 && strictBounds(true) ? 1 : -1;\n}\n`;

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
