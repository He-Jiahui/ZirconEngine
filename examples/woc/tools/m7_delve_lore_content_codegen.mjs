import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHash = '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff';
const contentHash = '567823b3054abd3c9ef568906800ad666b0697a8ee6d08cdd036d8cd0bf06506';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_lore_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_lore_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_lore_content.zr');
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
  throw new Error(`Delve lore content drifted: ${actualContentHash}`);
}
const source = execFileSync('git', ['-C', sourceRoot, 'show', `${commit}:src/sim/delves/runs.ts`], {
  encoding: 'utf8',
});
if (createHash('sha256').update(source).digest('hex') !== sourceHash) {
  throw new Error('Delve lore source drifted');
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_lore_content_codegen.mjs',
  source_sha256: { 'src/sim/delves/runs.ts': sourceHash },
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const idCases = content.lore_order
  .map((id, index) => `    if (index == ${index}) { return \"${id}\"; }`)
  .join('\n');
const zr = `// Generated Delve lore unlock order.\npub loreCount(required: bool): int {\n    if (!required) { throw \"woc Delve lore catalog is required\"; }\n    return ${content.lore_order.length};\n}\n\npub loreId(index: int, required: bool): string {\n    if (!required) { throw \"woc Delve lore catalog is required\"; }\n${idCases}\n    return \"\";\n}\n\npub contractTest(): int {\n    return loreCount(true) == 5 && loreId(0, true) == \"eastbrook_ledger\" &&\n        loreId(4, true) == \"tessa_note\" && loreId(5, true) == \"\" ? 1 : -1;\n}\n`;

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
