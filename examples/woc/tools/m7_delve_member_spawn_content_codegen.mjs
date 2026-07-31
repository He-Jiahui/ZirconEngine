import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHashes = {
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
  'src/sim/content/delves/collapsed_reliquary.ts': 'f12a3538da887f8e7dd2fcf804287df7609f0c706284be51a377e70ea5e1b00d',
  'src/sim/content/delves/drowned_litany.ts': '8f747166e6a63d36b8c20bae0d4feb43ba592376d2df0eb6139f4489aab1acb3',
};
const contentHash = '12a661ce59289675541d0350bf7c49c54e3766067b5f5f9a4efdd2c4374a5be9';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_member_spawn_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_member_spawn_content.json');
const zrPath = join(
  projectRoot,
  'scripts',
  'woc_game',
  'src',
  'instances',
  'delve_member_spawn_content.zr',
);
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
  throw new Error(`delve member spawn content drifted: ${actualContentHash}`);
}

for (const [path, expectedHash] of Object.entries(sourceHashes)) {
  const source = execFileSync('git', ['-C', sourceRoot, 'show', `${commit}:${path}`], {
    encoding: 'utf8',
  });
  const actualHash = createHash('sha256').update(source).digest('hex');
  if (actualHash !== expectedHash) {
    throw new Error(`source drifted: ${path}`);
  }
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_member_spawn_content_codegen.mjs',
  source_sha256: sourceHashes,
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const zr = `// Generated Delve member-entry horizontal placement contract.\n// Terrain height remains a host/world query after this source-derived offset.\npub partyMaxMembers(required: bool): int {\n    if (!required) { throw \"woc Delve party limit is required\"; }\n    return 2;\n}\n\npub spawnSpread(required: bool): float {\n    if (!required) { throw \"woc Delve spawn spread is required\"; }\n    return 2.2;\n}\n\npub slotOffsetCoordinate(slotIndex: int, axis: int, required: bool): float {\n    if (!required) { throw \"woc Delve member spawn is required\"; }\n    if (slotIndex < 0 || slotIndex >= partyMaxMembers(true)) {\n        throw \"woc Delve member slot is outside the source solo/duo limit\";\n    }\n    if (axis != 1 && axis != 2) { throw \"woc Delve member spawn axis is invalid\"; }\n    if (slotIndex == 0) { return 0.0; }\n    return axis == 1 ? 1.1000000000000003 : 1.905255888325765;\n}\n\npub contractTest(): int {\n    return partyMaxMembers(true) == 2 && spawnSpread(true) == 2.2 &&\n        slotOffsetCoordinate(0, 1, true) == 0.0 &&\n        slotOffsetCoordinate(0, 2, true) == 0.0 &&\n        slotOffsetCoordinate(1, 1, true) == 1.1000000000000003 &&\n        slotOffsetCoordinate(1, 2, true) == 1.905255888325765 ? 1 : -1;\n}\n`;

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
