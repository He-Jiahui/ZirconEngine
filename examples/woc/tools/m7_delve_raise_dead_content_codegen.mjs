import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_CONTENT_SHA256 = '0626ff615d47142580a753894c9b69c5d7cccb507d178852552d7ce817c51770';
const EXPECTED_SOURCE_SHA256 = { 'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff' };
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm7_delve_raise_dead_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_raise_dead_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_raise_dead_content.zr');
const checkOnly = process.argv.includes('--check');
main();
function main() {
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(sourceManifest.source_commit === SOURCE_COMMIT, 'Raise Dead source manifest drifted');
  const content = extract();
  assert(content.channel_seconds === 5 && content.interrupt_object_kind === 'cracked_grave' && content.completion_requires_living_boss && content.completion_spawns_boss_adds && content.start_requires_cracked_grave, 'Raise Dead source contract drifted');
  const hash = sha256(JSON.stringify(content)); assert(hash === EXPECTED_CONTENT_SHA256, 'Raise Dead content drifted');
  const source = gitShow('src/sim/delves/runs.ts'); assert(sha256(source) === EXPECTED_SOURCE_SHA256['src/sim/delves/runs.ts'], 'runs.ts drifted');
  const catalog = { schema_version: 1, source_commit: SOURCE_COMMIT, generated_by: 'examples/woc/tools/m7_delve_raise_dead_content_codegen.mjs', source_sha256: EXPECTED_SOURCE_SHA256, content, catalog_sha256: hash };
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, render(content));
}
function extract() { const child = spawnSync(process.execPath, [extractorPath], { encoding: 'utf8', env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT } }); assert(child.status === 0, child.stderr || 'Raise Dead extractor failed'); return JSON.parse(child.stdout); }
function gitShow(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' }); }
function render(content) {
  return [
    '// Generated Raise Dead channel contract from pinned source.',
    `// Source ${SOURCE_COMMIT}; do not edit by hand.`,
    `pub channelSeconds(required: bool): float { if (!required) { throw "woc Raise Dead channel is required"; } return ${number(content.channel_seconds)}; }`,
    `pub interruptObjectKind(required: bool): string { if (!required) { throw "woc Raise Dead interrupt object is required"; } return "${content.interrupt_object_kind}"; }`,
    `pub completionRequiresLivingBoss(required: bool): bool { if (!required) { throw "woc Raise Dead completion rule is required"; } return ${boolean(content.completion_requires_living_boss)}; }`,
    `pub completionSpawnsBossAdds(required: bool): bool { if (!required) { throw "woc Raise Dead boss-add completion rule is required"; } return ${boolean(content.completion_spawns_boss_adds)}; }`,
    `pub startRequiresCrackedGrave(required: bool): bool { if (!required) { throw "woc Raise Dead start requirement is required"; } return ${boolean(content.start_requires_cracked_grave)}; }`,
    'pub contractTest(): int {',
    `    return channelSeconds(true) == ${number(content.channel_seconds)} &&`,
    `        interruptObjectKind(true) == "${content.interrupt_object_kind}" &&`,
    `        completionRequiresLivingBoss(true) == ${boolean(content.completion_requires_living_boss)} &&`,
    `        completionSpawnsBossAdds(true) == ${boolean(content.completion_spawns_boss_adds)} &&`,
    `        startRequiresCrackedGrave(true) == ${boolean(content.start_requires_cracked_grave)} ? 1 : -1;`,
    '}',
    '',
  ].join('\n');
}
function sha256(text) { return createHash('sha256').update(text).digest('hex'); }
function number(value) { assert(Number.isFinite(value), `invalid Raise Dead number ${value}`); return Number.isInteger(value) ? `${value}.0` : `${value}`; }
function boolean(value) { return value ? 'true' : 'false'; }
function verifyOrWrite(path, text) { if (checkOnly) { assert(existsSync(path), `missing ${path}`); assert(readFileSync(path, 'utf8') === text, `drifted ${path}`); } else writeFileSync(path, text, 'utf8'); }
function assert(condition, message) { if (!condition) throw new Error(message); }
