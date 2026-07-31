import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_CONTENT_SHA256 =
  'fc4ba3bdc9f01043cd62a679acd208f2e6844f6c212fb5656f84c56b123159fd';
const EXPECTED_SOURCE_SHA256 = {
  'src/sim/delves/runs.ts':
    '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
};

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm7_bad_air_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_bad_air_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances', 'delve_bad_air_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');
  const content = extract();
  assert(content.interval_seconds === 8 && content.aura.id === 'bad_air' &&
    content.aura.name === 'Bad Air' && content.aura.kind === 'dot' &&
    content.aura.school === 'nature' && content.aura.remaining === 4 &&
    content.aura.duration === 4 && content.aura.value === 3 &&
    content.aura.tick_interval === 2 && content.aura.tick_timer === 2 &&
    content.aura.source_id === 'self', 'Bad Air source contract drifted');
  const contentHash = sha256(JSON.stringify(content));
  assert(contentHash === EXPECTED_CONTENT_SHA256, 'Bad Air content drifted');
  const sourceTexts = Object.fromEntries(Object.keys(EXPECTED_SOURCE_SHA256)
    .map((sourcePath) => [sourcePath, gitShow(sourcePath)]));
  for (const [sourcePath, expectedHash] of Object.entries(EXPECTED_SOURCE_SHA256)) {
    assert(sha256(sourceTexts[sourcePath]) === expectedHash, `${sourcePath} drifted`);
  }
  assert(sourceTexts['src/sim/delves/runs.ts'].includes('tickDelveBadAir') &&
    sourceTexts['src/sim/delves/runs.ts'].includes('run.badAirTimer = 0'),
  'Bad Air timer source behavior is absent');

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m7_bad_air_content_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts)
      .map(([sourcePath, text]) => [sourcePath, sha256(text)])),
    content,
  };
  catalog.catalog_sha256 = contentHash;
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(content));
}

function extract() {
  const child = spawnSync(process.execPath, [extractorPath], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `Bad Air extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function renderZr(content) {
  const aura = content.aura;
  return [
    '// Generated Bad Air timer and aura contract from the pinned Delve source.',
    '',
    'pub intervalSeconds(required: bool): float {',
    '    if (!required) { throw "woc Bad Air interval is required"; }',
    `    return ${number(content.interval_seconds)};`,
    '}',
    '',
    'pub auraText(field: int, required: bool): string {',
    '    if (!required || field < 1 || field > 4) {',
    '        throw "woc Bad Air aura text field is invalid";',
    '    }',
    `    if (field == 1) { return "${aura.id}"; }`,
    `    if (field == 2) { return "${aura.name}"; }`,
    `    if (field == 3) { return "${aura.kind}"; }`,
    `    return "${aura.school}";`,
    '}',
    '',
    'pub auraNumber(field: int, required: bool): float {',
    '    if (!required || field < 1 || field > 5) {',
    '        throw "woc Bad Air aura number field is invalid";',
    '    }',
    `    if (field == 1) { return ${number(aura.remaining)}; }`,
    `    if (field == 2) { return ${number(aura.duration)}; }`,
    `    if (field == 3) { return ${number(aura.value)}; }`,
    `    if (field == 4) { return ${number(aura.tick_interval)}; }`,
    `    return ${number(aura.tick_timer)};`,
    '}',
    '',
    'pub contractTest(): int {',
    `    if (intervalSeconds(true) != ${number(content.interval_seconds)} ||`,
    `        auraText(1, true) != "${aura.id}" || auraText(4, true) != "${aura.school}" ||`,
    `        auraNumber(1, true) != ${number(aura.remaining)} ||`,
    `        auraNumber(5, true) != ${number(aura.tick_timer)}) {`,
    '        return -1;',
    '    }',
    '    return 1;',
    '}',
    '',
  ].join('\n');
}

function number(value) {
  assert(Number.isFinite(value), `cannot emit non-finite Bad Air number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : value.toString();
}

function sha256(text) {
  return createHash('sha256').update(text).digest('hex');
}

function verifyOrWrite(path, text) {
  if (checkOnly) {
    assert(existsSync(path), `generated file is missing: ${path}`);
    assert(readFileSync(path, 'utf8') === text, `generated file drifted: ${path}`);
    return;
  }
  writeFileSync(path, text, 'utf8');
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
