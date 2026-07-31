import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_GOLDENS = 54;
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const catalogPath = join(projectRoot, 'reference', 'current-head', 'parity_scenarios.json');
const outputRoot = join(projectRoot, 'reference', 'current-head', 'parity');
const goldenRoot = join(outputRoot, 'golden');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'));
  invariant(catalog.schema_version === 1, 'current-head parity catalog schema drifted');
  invariant(catalog.source_commit === SOURCE_COMMIT, 'current-head parity catalog commit drifted');
  invariant(Array.isArray(catalog.entries), 'current-head parity entries are missing');
  invariant(catalog.entries.length === EXPECTED_GOLDENS,
    `expected ${EXPECTED_GOLDENS} current-head parity rows, found ${catalog.entries.length}`);

  const entries = catalog.entries.map((entry, index) => normalizeEntry(entry, index));
  const expected = new Map();
  expected.set('scenarios.json', `${JSON.stringify({ source_commit: SOURCE_COMMIT, entries }, null, 2)}\n`);
  for (const entry of entries) {
    const bytes = sourceBlob(entry.golden);
    const digest = sha256(bytes);
    invariant(digest === entry.golden_sha256,
      `current-head golden digest drifted for ${entry.name}: ${digest}`);
    expected.set(join('golden', `${entry.name}.json`), bytes);
  }
  assertExpectedPaths(expected);
  for (const [relativePath, contents] of expected) writeOrCheck(relativePath, contents);
  process.stdout.write(`${checkOnly ? 'checked' : 'materialized'} ${entries.length} current-head parity goldens\n`);
}

function normalizeEntry(entry, index) {
  invariant(entry.index === index, `current-head parity row ${index} is not contiguous`);
  invariant(typeof entry.name === 'string' && entry.name.length > 0, `parity row ${index} has no name`);
  invariant(typeof entry.source_owner === 'string', `parity row ${entry.name} has no source owner`);
  invariant(typeof entry.golden === 'string' && entry.golden === `tests/parity/golden/${entry.name}.json`,
    `parity row ${entry.name} has an unexpected golden path`);
  invariant(typeof entry.golden_sha256 === 'string' && /^[0-9a-f]{64}$/u.test(entry.golden_sha256),
    `parity row ${entry.name} has an invalid golden digest`);
  invariant(Array.isArray(entry.coverage), `parity row ${entry.name} has invalid coverage`);
  return {
    name: entry.name,
    source_owner: entry.source_owner,
    golden: entry.golden,
    golden_sha256: entry.golden_sha256,
    coverage: entry.coverage,
  };
}

function sourceBlob(path) {
  return Buffer.from(execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer',
  }));
}

function assertExpectedPaths(expected) {
  if (!existsSync(outputRoot)) return;
  const actual = collectJsonPaths(outputRoot);
  const unexpected = actual.filter((path) => !expected.has(path));
  invariant(unexpected.length === 0,
    `current-head parity output contains unexpected files: ${unexpected.join(', ')}`);
}

function collectJsonPaths(root) {
  const paths = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) {
      for (const child of collectJsonPaths(path)) paths.push(join(entry.name, child));
    } else if (entry.isFile() && entry.name.endsWith('.json')) {
      paths.push(relative(root, path));
    }
  }
  return paths.sort();
}

function writeOrCheck(relativePath, contents) {
  const path = join(outputRoot, relativePath);
  if (checkOnly) {
    invariant(existsSync(path), `current-head parity output is missing ${relativePath}`);
    const actual = readFileSync(path);
    invariant(Buffer.compare(actual, Buffer.from(contents)) === 0,
      `current-head parity output is stale at ${relativePath}`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
