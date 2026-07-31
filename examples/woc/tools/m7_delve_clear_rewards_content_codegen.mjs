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
const contentHash = '9d5a939810d7c8328e57b48eab17879a8295e1083aaf7c8e0d90bca9e0e6eead';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_clear_rewards_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_clear_rewards_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_clear_rewards_content.zr');
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
  throw new Error(`Delve clear reward content drifted: ${actualContentHash}`);
}
for (const [path, expectedHash] of Object.entries(sourceHashes)) {
  const source = execFileSync('git', ['-C', sourceRoot, 'show', `${commit}:${path}`], {
    encoding: 'utf8',
  });
  if (createHash('sha256').update(source).digest('hex') !== expectedHash) {
    throw new Error(`Delve clear reward source drifted: ${path}`);
  }
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_clear_rewards_content_codegen.mjs',
  source_sha256: sourceHashes,
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const tiers = content.delves.flatMap((delve, delveIndex) =>
  delve.tiers.map((tier, tierIndex) => ({ ...tier, delveIndex, tierIndex })),
);
const tierAccessor = (name, field) => {
  const cases = tiers
    .map(
      (tier) =>
        `    if (delveIndex == ${tier.delveIndex} && tierIndex == ${tier.tierIndex}) { return ${tier[field]}; }`,
    )
    .join('\n');
  return `pub ${name}(delveIndex: int, tierIndex: int, required: bool): int {\n    if (!required) { throw \"woc Delve clear reward is required\"; }\n${cases}\n    throw \"woc Delve reward tier is invalid\";\n}\n`;
};
const delveIdCases = content.delves
  .map((delve, index) => `    if (delveIndex == ${index}) { return \"${delve.id}\"; }`)
  .join('\n');
const tierIdCases = tiers
  .map(
    (tier) =>
      `    if (delveIndex == ${tier.delveIndex} && tierIndex == ${tier.tierIndex}) { return \"${tier.id}\"; }`,
  )
  .join('\n');
const tierCountCases = content.delves
  .map((delve, index) => `    if (delveIndex == ${index}) { return ${delve.tiers.length}; }`)
  .join('\n');
const zr = `// Generated Delve completion XP and copper-range content.\npub delveCount(required: bool): int {\n    if (!required) { throw \"woc Delve reward catalog is required\"; }\n    return ${content.delves.length};\n}\n\npub delveId(delveIndex: int, required: bool): string {\n    if (!required) { throw \"woc Delve reward catalog is required\"; }\n${delveIdCases}\n    return \"\";\n}\n\npub tierCount(delveIndex: int, required: bool): int {\n    if (!required) { throw \"woc Delve reward catalog is required\"; }\n${tierCountCases}\n    return 0;\n}\n\npub tierId(delveIndex: int, tierIndex: int, required: bool): string {\n    if (!required) { throw \"woc Delve reward catalog is required\"; }\n${tierIdCases}\n    return \"\";\n}\n\npub markMultiplier(delveIndex: int, required: bool): int {\n    if (!required) { throw \"woc Delve reward catalog is required\"; }\n    return delveIndex == 1 ? ${content.drowned_litany_mark_multiplier} : 1;\n}\n\n${tierAccessor('firstClearXp', 'first_clear_xp')}\n${tierAccessor('repeatClearXp', 'repeat_clear_xp')}\n${tierAccessor('copperMinimum', 'copper_min')}\n${tierAccessor('copperMaximum', 'copper_max')}\npub contractTest(): int {\n    return delveCount(true) == 2 && delveId(0, true) == \"collapsed_reliquary\" &&\n        tierId(1, 1, true) == \"heroic\" && firstClearXp(0, 0, true) == 700 &&\n        repeatClearXp(0, 1, true) == 650 && copperMinimum(1, 0, true) == 18 &&\n        copperMaximum(1, 1, true) == 48 && markMultiplier(0, true) == 1 &&\n        markMultiplier(1, true) == 2 ? 1 : -1;\n}\n`;

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
