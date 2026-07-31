import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const commit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceHash = 'ce46a3f5f8bbb86d2903f54aaa84f4c709a9825e0c5f76fa9f004d1822e43552';
const contentHash = 'ad6b306e6bd38bbb26e18752a9ffee45b00b172e3ec6cd3df072cfb6c1f9312d';
const toolDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(toolDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(toolDirectory, 'm7_delve_shop_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_shop_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_shop_content.zr');
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
  throw new Error(`delve shop content drifted: ${actualContentHash}`);
}
const source = execFileSync(
  'git',
  ['-C', sourceRoot, 'show', `${commit}:src/sim/content/delves/shop.ts`],
  { encoding: 'utf8' },
);
if (createHash('sha256').update(source).digest('hex') !== sourceHash) {
  throw new Error('Delve shop source drifted');
}

const catalog = {
  schema_version: 1,
  source_commit: commit,
  generated_by: 'examples/woc/tools/m7_delve_shop_content_codegen.mjs',
  source_sha256: { 'src/sim/content/delves/shop.ts': sourceHash },
  content,
  catalog_sha256: contentHash,
};
const json = `${JSON.stringify(catalog, null, 2)}\n`;
const offers = content.shops.flatMap((shop, delveIndex) =>
  shop.offers.map((offer, offerIndex) => ({ ...offer, delveIndex, offerIndex })),
);
const offerAccessor = (name, type, valueFor) => {
  const cases = offers
    .map(
      (offer) =>
        `    if (delveIndex == ${offer.delveIndex} && offerIndex == ${offer.offerIndex}) { return ${valueFor(offer)}; }`,
    )
    .join('\n');
  return `pub ${name}(delveIndex: int, offerIndex: int, required: bool): ${type} {\n    if (!required) { throw \"woc Delve shop offer is required\"; }\n${cases}\n    throw \"woc Delve shop offer index is invalid\";\n}\n`;
};
const countCases = content.shops
  .map((shop, index) => `    if (delveIndex == ${index}) { return ${shop.offers.length}; }`)
  .join('\n');
const shopIdCases = content.shops
  .map((shop, index) => `    if (delveIndex == ${index}) { return \"${shop.id}\"; }`)
  .join('\n');
const gateKind = (offer) => content.gate_kinds[offer.gate.startsWith('clears:') ? 'clears' : offer.gate];
const requiredClears = (offer) => (offer.gate.startsWith('clears:') ? Number(offer.gate.slice(7)) : 0);
const zr = `// Generated Delve Marks shop catalog and gate metadata.\npub shopCount(required: bool): int {\n    if (!required) { throw \"woc Delve shop catalog is required\"; }\n    return ${content.shops.length};\n}\n\npub shopId(delveIndex: int, required: bool): string {\n    if (!required) { throw \"woc Delve shop catalog is required\"; }\n${shopIdCases}\n    return \"\";\n}\n\npub offerCount(delveIndex: int, required: bool): int {\n    if (!required) { throw \"woc Delve shop catalog is required\"; }\n${countCases}\n    return 0;\n}\n\n${offerAccessor('offerItemId', 'string', (offer) => `\"${offer.item_id}\"`)}\n${offerAccessor('offerMarks', 'int', (offer) => `${offer.marks}`)}\n${offerAccessor('offerGateKind', 'int', (offer) => `${gateKind(offer)}`)}\n${offerAccessor('offerRequiredClears', 'int', (offer) => `${requiredClears(offer)}`)}\npub contractTest(): int {\n    return shopCount(true) == 2 && shopId(0, true) == \"collapsed_reliquary\" &&\n        shopId(1, true) == \"drowned_litany\" && offerCount(0, true) == 9 &&\n        offerCount(1, true) == 9 && offerItemId(0, 0, true) == \"reliquary_legs\" &&\n        offerMarks(0, 6, true) == 12 && offerGateKind(0, 6, true) == 2 &&\n        offerRequiredClears(0, 6, true) == 3 && offerGateKind(1, 8, true) == 3 &&\n        offerMarks(1, 8, true) == 56 ? 1 : -1;\n}\n`;

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
