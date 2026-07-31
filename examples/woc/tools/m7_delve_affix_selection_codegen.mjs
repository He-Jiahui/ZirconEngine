import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_CATALOG_SHA256 =
  '707a287d674779984109a9ba0d09058ad331d6350dd48a2b192c39feb4284131';
const EXPECTED_SOURCE_SHA256 = {
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
  'src/sim/rng.ts': 'd516034919a56ce15f3a893cfd07345b851a2eb833704009fbfbace24c446713',
  'src/sim/content/delves/affixes.ts': '9ea53c2bbc4b99e7f162460674757d1a3df01f4dcc4290f1885ca73c243f7b9e',
  'src/sim/content/delves/collapsed_reliquary.ts': 'f12a3538da887f8e7dd2fcf804287df7609f0c706284be51a377e70ea5e1b00d',
  'src/sim/content/delves/drowned_litany.ts': '8f747166e6a63d36b8c20bae0d4feb43ba592376d2df0eb6139f4489aab1acb3',
};
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm7_delve_affix_selection_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm7_delve_affix_selection.json');
const contentPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_affix_selection_content.zr');
const rulesPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_affix_selection.zr');
const checkOnly = process.argv.includes('--check');

main();
function main() {
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(sourceManifest.source_commit === SOURCE_COMMIT, 'Delve affix source manifest drifted');
  const extracted = extract();
  assert(extracted.seed_xor === 1511112926, 'Delve affix seed xor drifted');
  assert(JSON.stringify(extracted.delves.map((delve) => delve.pool)) ===
    JSON.stringify([['restless_graves', 'bad_air', 'candleblind'],
      ['high_water', 'lively_choir', 'belligerent_dead']]), 'Delve affix pools drifted');
  assert(extracted.vectors.length === 32 && extracted.vectors.filter((vector) =>
    vector.tier_id === 'normal').every((vector) => vector.affixes.length === 0) &&
    extracted.vectors.filter((vector) => vector.tier_id === 'heroic').every((vector) =>
      vector.affixes.length === 1), 'Delve affix vector coverage drifted');
  const catalogHash = sha256(JSON.stringify(extracted));
  assert(catalogHash === EXPECTED_CATALOG_SHA256, 'Delve affix catalog drifted');
  const sourceTexts = Object.fromEntries(Object.keys(EXPECTED_SOURCE_SHA256).map((path) => [path, gitShow(path)]));
  for (const [path, expected] of Object.entries(EXPECTED_SOURCE_SHA256)) {
    if (expected !== 'TODO') assert(sha256(sourceTexts[path]) === expected, `${path} drifted`);
  }
  assert(sourceTexts['src/sim/delves/runs.ts'].includes('seed ^ 0x5a11c0de') &&
    sourceTexts['src/sim/delves/runs.ts'].includes('DELVE_IMPLEMENTED_AFFIXES') &&
    sourceTexts['src/sim/delves/runs.ts'].includes("run.affixes.includes('belligerent_dead')") &&
    sourceTexts['src/sim/delves/runs.ts'].includes('mob.maxHp = Math.round(mob.maxHp * 1.1);') &&
    sourceTexts['src/sim/delves/runs.ts'].includes('mob.hp = mob.maxHp;'),
  'source Delve affix behavior is absent');
  const catalog = { schema_version: 1, source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m7_delve_affix_selection_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts).map(([path, text]) => [path, sha256(text)])),
    ...extracted, catalog_sha256: catalogHash };
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(contentPath, renderContent(extracted));
  verifyOrWrite(rulesPath, renderRules(extracted));
}
function extract() { const child = spawnSync(process.execPath, ['--no-warnings','--experimental-loader',loaderUrl,extractorPath], { encoding:'utf8', maxBuffer:32*1024*1024, env:{...process.env,WOC_GIT_ROOT:sourceRoot,WOC_GIT_COMMIT:SOURCE_COMMIT} }); assert(child.status===0, child.stderr || 'affix extractor failed'); return JSON.parse(child.stdout); }
function gitShow(path) { return execFileSync('git',['-C',sourceRoot,'show',`${SOURCE_COMMIT}:${path}`],{encoding:'utf8'}); }
function renderContent(extracted) { const lines=['// Generated deterministic Delve affix selection catalog.','']; lines.push('pub seedXor(required: bool): int { if (!required) { throw "woc affix seed xor is required"; } return '+extracted.seed_xor+'; }',''); lines.push('pub poolCount(delveIndex: int, required: bool): int { if (!required || delveIndex < 0 || delveIndex >= '+extracted.delves.length+') { throw "woc Delve affix pool is invalid"; } return 3; }',''); lines.push('pub poolId(delveIndex: int, poolIndex: int, required: bool): string { if (!required || delveIndex < 0 || delveIndex >= '+extracted.delves.length+' || poolIndex < 0 || poolIndex >= 3) { throw "woc Delve affix pool id is invalid"; }'); for (const delve of extracted.delves) { lines.push('    if (delveIndex == '+delve.index+') { if (poolIndex == 0) { return "'+delve.pool[0]+'"; } if (poolIndex == 1) { return "'+delve.pool[1]+'"; } return "'+delve.pool[2]+'"; }'); } lines.push('    throw "woc Delve affix pool id is invalid";','}',''); lines.push('pub tierAffixCount(delveIndex: int, tierIndex: int, required: bool): int { if (!required || delveIndex < 0 || delveIndex >= '+extracted.delves.length+') { throw "woc Delve affix tier is invalid"; } return tierIndex == 1 ? 1 : 0; }',''); lines.push('pub contractTest(): int { if (seedXor(true) != '+extracted.seed_xor+' || poolId(0,0,true) != "restless_graves" || poolId(1,2,true) != "belligerent_dead" || tierAffixCount(0,0,true) != 0 || tierAffixCount(1,1,true) != 1) { return -1; } return 1; }',''); return lines.join('\n'); }
function renderRules(extracted) { const heroVectors=extracted.vectors.filter((vector)=>vector.tier_id==='heroic'); const lines=['// Exact rollDelveAffixes scalar projection for the pinned two Delve definitions.','var content = %import("instances/delve_affix_selection_content");','var rngModule = %import("kernel/rng");','', 'pub affixCount(delveIndex: int, tierIndex: int, required: bool): int { return content.tierAffixCount(delveIndex,tierIndex,required); }','', 'pub affixId(delveIndex: int, tierIndex: int, seed: int, affixOffset: int, required: bool): string {', '    var count = affixCount(delveIndex,tierIndex,required);', '    if (affixOffset < 0 || affixOffset >= count) { throw "woc Delve affix offset is invalid"; }', '    var entry0 = content.poolId(delveIndex,0,true); var entry1 = content.poolId(delveIndex,1,true); var entry2 = content.poolId(delveIndex,2,true);', '    var rng = new rngModule.Mulberry32(<uint>(seed ^ content.seedXor(true)));', '    var selected = <int>(rng.next() * 3.0); var swap2 = entry2; if (selected == 0) { entry2 = entry0; entry0 = swap2; } else if (selected == 1) { entry2 = entry1; entry1 = swap2; }', '    selected = <int>(rng.next() * 2.0); var swap1 = entry1; if (selected == 0) { entry1 = entry0; entry0 = swap1; }', '    return entry0;', '}', '', 'pub contractTest(): int { if (content.contractTest() != 1) { return -1; }']; let code=-2; for (const vector of heroVectors) { lines.push('    if (affixId('+vector.delve_index+', 1, '+vector.seed+', 0, true) != "'+vector.affixes[0]+'") { return '+code+'; }'); code--; } lines.push('    return 1;','}',''); return lines.join('\n'); }
function sha256(text) { return createHash('sha256').update(text).digest('hex'); }
function verifyOrWrite(path,text) { if (checkOnly) { assert(existsSync(path),'generated file missing: '+path); assert(readFileSync(path,'utf8')===text,'generated file drifted: '+path); } else writeFileSync(path,text,'utf8'); }
function assert(condition,message) { if (!condition) throw new Error(message); }
