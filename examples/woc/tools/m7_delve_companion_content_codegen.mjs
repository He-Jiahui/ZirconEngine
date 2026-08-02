import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHashes = {
  'src/sim/content/delves/companions.ts': 'af55fba0fdd9e5fc5e4c0d820cb1d6ae8312c8058ca08c15bae0bd7cca4f727f',
  'src/sim/types.ts': '303321fb109f7bcce51a6871597be5b7b05f54ed1c6b86185387390cc47ccd90',
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
};
const contentHash = '8f59e3ff64676eef83ae8550650c3613d01efc9a08fabaf2b2a28d08351e4f21';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_companion_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_companion_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_companion_content.zr');
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
  throw new Error(`Delve companion content drifted: ${actualContentHash}`);
}
for (const [path, expectedHash] of Object.entries(sourceHashes)) {
  const source = execFileSync('git', ['-C', sourceRoot, 'show', `${commit}:${path}`], {
    encoding: 'utf8',
  });
  if (createHash('sha256').update(source).digest('hex') !== expectedHash) {
    throw new Error(`Delve companion source drifted: ${path}`);
  }
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_companion_content_codegen.mjs',
  source_sha256: sourceHashes,
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const companionCases = content.companions
  .map((companion, index) => `    if (index == ${index}) { return \"${companion.id}\"; }`)
  .join('\n');
const companionUtf8LengthCases = content.companions
  .map((companion, index) =>
    `    if (index == ${index}) { return ${Buffer.byteLength(companion.id, 'utf8')}; }`)
  .join('\n');
const companionUtf8ByteCases = content.companions
  .flatMap((companion, index) => Array.from(Buffer.from(companion.id, 'utf8'))
    .map((byte, byteIndex) =>
      `    if (index == ${index} && byteIndex == ${byteIndex}) { return <uint>${byte}; }`))
  .join('\n');
const marksCases = content.upgrade_costs
  .map((cost) => `    if (rank == ${cost.rank}) { return ${cost.marks}; }`)
  .join('\n');
const copperCases = content.upgrade_costs
  .map((cost) => `    if (rank == ${cost.rank}) { return ${cost.copper}; }`)
  .join('\n');
const zr = `// Generated Delve companion identities and rank-up costs.\npub maxRank(required: bool): int {\n    if (!required) { throw \"woc Delve companion catalog is required\"; }\n    return ${content.max_rank};\n}\n\npub companionCount(required: bool): int {\n    if (!required) { throw \"woc Delve companion catalog is required\"; }\n    return ${content.companions.length};\n}\n\npub companionId(index: int, required: bool): string {\n    if (!required) { throw \"woc Delve companion catalog is required\"; }\n${companionCases}\n    return \"\";\n}\n\npub companionIdUtf8Length(index: int, required: bool): int {\n    if (!required) { throw \"woc Delve companion catalog is required\"; }\n${companionUtf8LengthCases}\n    return 0;\n}\n\npub companionIdUtf8Byte(index: int, byteIndex: int, required: bool): uint {\n    if (!required) { throw \"woc Delve companion catalog is required\"; }\n${companionUtf8ByteCases}\n    return <uint>0;\n}\n\npub upgradeMarks(rank: int, required: bool): int {\n    if (!required) { throw \"woc Delve companion upgrade is required\"; }\n${marksCases}\n    return 0;\n}\n\npub upgradeCopper(rank: int, required: bool): int {\n    if (!required) { throw \"woc Delve companion upgrade is required\"; }\n${copperCases}\n    return 0;\n}\n\npub contractTest(): int {\n    return maxRank(true) == 3 && companionCount(true) == 2 &&\n        companionId(0, true) == \"companion_tessa\" &&\n        companionId(1, true) == \"companion_edda\" &&\n        companionIdUtf8Length(0, true) == 15 &&\n        companionIdUtf8Length(1, true) == 14 &&\n        companionIdUtf8Byte(0, 0, true) == <uint>99 &&\n        companionIdUtf8Byte(1, 13, true) == <uint>97 &&\n        upgradeMarks(2, true) == 3 &&\n        upgradeMarks(3, true) == 5 && upgradeCopper(2, true) == 0 &&\n        upgradeCopper(3, true) == 0 ? 1 : -1;\n}\n`;

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
