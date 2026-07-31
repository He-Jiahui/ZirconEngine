import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HISTORICAL_COMMIT = '7c10f280eec380e9877e66ce16333089e171fe42';
const CURRENT_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const checkOnly = process.argv.includes('--check');
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const referenceRoot = resolve(scriptDirectory, '..', 'reference');
const historicalRoot = referenceRoot;
const currentRoot = join(referenceRoot, 'current-head');
const outputPath = join(currentRoot, 'delta_from_7c10.json');

main();

function main() {
  const historicalManifest = documentAt(historicalRoot, 'source_manifest.json');
  const currentManifest = documentAt(currentRoot, 'source_manifest.json');
  invariant(historicalManifest.source_commit === HISTORICAL_COMMIT,
    'historical reference catalog does not use the expected source commit');
  invariant(currentManifest.source_commit === CURRENT_COMMIT,
    'current-head reference catalog does not use the expected source commit');

  const historical = catalogsAt(historicalRoot);
  const current = catalogsAt(currentRoot);
  const document = {
    schema_version: 1,
    from_commit: HISTORICAL_COMMIT,
    to_commit: CURRENT_COMMIT,
    generated_by: 'examples/woc/tools/reference_delta.mjs',
    source_manifest_sha256: {
      historical: sha256(render(historicalManifest)),
      current_head: sha256(render(currentManifest)),
    },
    totals: totalsDelta(historicalManifest.audited_totals, currentManifest.audited_totals),
    commands: catalogDelta(historical.commands.entries, current.commands.entries, (entry) => entry.name),
    world_api: {
      members: catalogDelta(historical.world.entries, current.world.entries,
        (entry) => `${entry.facet}:${entry.name}`),
      facets: catalogDelta(historical.world.facets, current.world.facets, (entry) => entry.name),
    },
    parity: catalogDelta(historical.parity.entries, current.parity.entries, (entry) => entry.name),
    assets: assetDelta(historical.assets.entries, current.assets.entries),
  };
  const output = render(document);
  if (checkOnly) {
    invariant(existsSync(outputPath), 'current-head delta is missing; run npm run generate:current-delta');
    invariant(readFileSync(outputPath, 'utf8') === output,
      'current-head delta is stale; run npm run generate:current-delta');
    process.stdout.write(`checked WOC current-head delta at ${outputPath}\n`);
    return;
  }
  writeFileSync(outputPath, output, 'utf8');
  process.stdout.write(`generated WOC current-head delta at ${outputPath}\n`);
}

function catalogsAt(root) {
  return {
    commands: documentAt(root, 'command_catalog.json'),
    world: documentAt(root, 'world_api_catalog.json'),
    parity: documentAt(root, 'parity_scenarios.json'),
    assets: documentAt(root, 'asset_catalog.json'),
  };
}

function documentAt(root, name) {
  const path = join(root, name);
  invariant(existsSync(path), `missing reference catalog: ${path}`);
  return JSON.parse(readFileSync(path, 'utf8'));
}

function totalsDelta(historical, current) {
  const keys = [...new Set([...Object.keys(historical), ...Object.keys(current)])].sort();
  return Object.fromEntries(keys.map((key) => {
    const before = historical[key] ?? null;
    const after = current[key] ?? null;
    return [key, {
      historical: before,
      current_head: after,
      delta: typeof before === 'number' && typeof after === 'number' ? after - before : null,
    }];
  }));
}

function catalogDelta(historical, current, key) {
  const before = indexBy(historical, key, 'historical catalog');
  const after = indexBy(current, key, 'current-head catalog');
  const keys = [...new Set([...before.keys(), ...after.keys()])].sort();
  const added = [];
  const removed = [];
  const changed = [];
  for (const id of keys) {
    const beforeEntry = before.get(id);
    const afterEntry = after.get(id);
    if (!beforeEntry) {
      added.push(afterEntry);
    } else if (!afterEntry) {
      removed.push(beforeEntry);
    } else if (render(beforeEntry) !== render(afterEntry)) {
      changed.push({ id, historical: beforeEntry, current_head: afterEntry });
    }
  }
  return { historical_count: historical.length, current_head_count: current.length, added, removed, changed };
}

function assetDelta(historical, current) {
  const delta = catalogDelta(historical, current, (entry) => entry.path);
  return {
    historical_count: delta.historical_count,
    current_head_count: delta.current_head_count,
    added: delta.added.map((entry) => entry.path),
    removed: delta.removed.map((entry) => entry.path),
    content_changed: delta.changed.map((entry) => entry.id),
  };
}

function indexBy(entries, key, label) {
  const indexed = new Map();
  for (const entry of entries) {
    const id = key(entry);
    invariant(!indexed.has(id), `duplicate ${label} key: ${id}`);
    indexed.set(id, entry);
  }
  return indexed;
}

function render(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
