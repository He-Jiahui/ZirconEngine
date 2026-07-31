import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/delves/runs.ts';
const EXPECTED_SOURCE_SHA256 = '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff';
const EXPECTED_CONTENT_SHA256 = '85abb00978edb529be9c0aee0a9c997375ecf9730a06680e72bc5ddb00ba6dc1';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm7_delve_candleblind_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_delve_candleblind_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_candleblind_content.zr');
const checkOnly = process.argv.includes('--check');

const child = spawnSync(process.execPath, [extractorPath], {
  encoding: 'utf8',
  env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
});
if (child.status !== 0) throw new Error(child.stderr || 'Candleblind extractor failed');
const content = JSON.parse(child.stdout);
const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
if (sourceManifest.source_commit !== SOURCE_COMMIT) throw new Error('Candleblind source manifest drifted');
const contentHash = sha256(JSON.stringify(content));
if (contentHash !== EXPECTED_CONTENT_SHA256) throw new Error('Candleblind content drifted');
const sourceText = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], {
  encoding: 'utf8',
});
if (sha256(sourceText) !== EXPECTED_SOURCE_SHA256) throw new Error('Candleblind source drifted');

const catalog = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/m7_delve_candleblind_content_codegen.mjs',
  source_sha256: { [SOURCE_PATH]: EXPECTED_SOURCE_SHA256 },
  content,
  catalog_sha256: contentHash,
};
const zr = [
  '// Generated Candleblind source contract.',
  `// Source ${SOURCE_COMMIT}; do not edit by hand.`,
  `pub detectionMultiplier(active: bool): float { return active ? ${number(content.active_multiplier)} : ${number(content.inactive_multiplier)}; }`,
  'pub requiresActiveAffix(required: bool): bool {',
  '    if (!required) { throw "woc Candleblind active-affix requirement is required"; }',
  `    return ${content.requires_active_affix ? 'true' : 'false'};`,
  '}',
  'pub contractTest(): int {',
  `    return detectionMultiplier(false) == ${number(content.inactive_multiplier)} &&`,
  `        detectionMultiplier(true) == ${number(content.active_multiplier)} &&`,
  `        requiresActiveAffix(true) == ${content.requires_active_affix ? 'true' : 'false'} ? 1 : -1;`,
  '}',
  '',
].join('\n');
for (const [path, value] of [[contractPath, `${JSON.stringify(catalog, null, 2)}\n`], [zrPath, zr]]) {
  if (checkOnly) {
    if (!existsSync(path) || readFileSync(path, 'utf8') !== value) throw new Error(`generated drift ${path}`);
  } else {
    writeFileSync(path, value, 'utf8');
  }
}

function sha256(text) {
  return createHash('sha256').update(text).digest('hex');
}

function number(value) {
  if (!Number.isFinite(value)) throw new Error(`invalid Candleblind number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : `${value}`;
}
