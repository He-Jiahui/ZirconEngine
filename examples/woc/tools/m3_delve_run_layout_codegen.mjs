import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_DELVES = [
  {
    id: 'collapsed_reliquary',
    index: 0,
    non_final_module_indices: [0, 1, 2],
    finale_module_index: 3,
    tier_ids: ['normal', 'heroic'],
    module_counts: [3, 3],
  },
  {
    id: 'drowned_litany',
    index: 1,
    non_final_module_indices: [4, 5, 6, 7, 8, 9],
    finale_module_index: 10,
    tier_ids: ['normal', 'heroic'],
    module_counts: [3, 3],
  },
];
const EXPECTED_SELECTION_VECTORS_SHA256 =
  'f214f7a9d2fa2866801d6284e29c33119a7ce03ec0dbd82ecef37b11ba79d1b7';
const EXPECTED_SOURCE_SHA256 = {
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
  'src/sim/rng.ts': 'd516034919a56ce15f3a893cfd07345b851a2eb833704009fbfbace24c446713',
  'src/sim/delve_layout.ts': 'bab9792386316e45b9d04ae920d056755f1e7facaa2bec30d4dc6793dc3fdd1e',
  'src/sim/content/delves/collapsed_reliquary.ts':
    'f12a3538da887f8e7dd2fcf804287df7609f0c706284be51a377e70ea5e1b00d',
  'src/sim/content/delves/drowned_litany.ts':
    '8f747166e6a63d36b8c20bae0d4feb43ba592376d2df0eb6139f4489aab1acb3',
};
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm3_delve_run_layout_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm3_delve_run_layout_content.json');
const contentZrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'world',
  'delve_run_layout_content.zr');
const layoutZrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'world',
  'delve_run_layout.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');

  const extracted = extract();
  assert(JSON.stringify(extracted.delves) === JSON.stringify(EXPECTED_DELVES),
    'active Delve definition/order drifted');
  assert(sha256(JSON.stringify(extracted.selection_vectors)) === EXPECTED_SELECTION_VECTORS_SHA256,
    'active Delve module selection vectors drifted');
  assert(extracted.selection_vectors.length === 42,
    'active Delve selection vector coverage drifted');
  assert(extracted.delves.every((delve) =>
    delve.non_final_module_indices.length > 0 &&
    delve.module_counts.length === delve.tier_ids.length &&
    delve.module_counts.every((count) =>
      Number.isInteger(count) && count > 0 && count <= delve.non_final_module_indices.length) &&
    Number.isInteger(delve.finale_module_index) && delve.finale_module_index >= 0),
  'active Delve definition is invalid');
  assert(extracted.selection_vectors.every((vector) => {
    const delve = extracted.delves.find((candidate) => candidate.index === vector.delve_index);
    return delve !== undefined &&
      vector.module_indices.length >= 2 &&
      vector.module_indices.at(-1) === delve.finale_module_index &&
      vector.module_indices.slice(0, -1)
        .every((moduleIndex) => delve.non_final_module_indices.includes(moduleIndex));
  }), 'active Delve selection result is invalid');

  const sourceTexts = Object.fromEntries(Object.keys(EXPECTED_SOURCE_SHA256)
    .map((path) => [path, gitShow(path)]));
  for (const [path, expectedHash] of Object.entries(EXPECTED_SOURCE_SHA256)) {
    assert(sha256(sourceTexts[path]) === expectedHash, `${path} drifted`);
  }
  assert(sourceTexts['src/sim/delves/runs.ts'].includes('export function pickDelveModules'),
    'source active Delve selector is absent');
  assert(sourceTexts['src/sim/delves/runs.ts'].includes('const j = rng.int(0, i);'),
    'source Fisher-Yates selector drifted');
  assert(sourceTexts['src/sim/rng.ts'].includes('Deterministic seeded RNG (mulberry32)'),
    'source run RNG changed');

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m3_delve_run_layout_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts)
      .map(([path, text]) => [path, sha256(text)])),
    delves: extracted.delves,
    selection_vectors: extracted.selection_vectors,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({
    delves: catalog.delves,
    selection_vectors: catalog.selection_vectors,
  }));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(contentZrPath, renderContentZr(catalog));
  verifyOrWrite(layoutZrPath, renderLayoutZr(catalog));
}

function extract() {
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `Delve run layout extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function renderContentZr(catalog) {
  const { delves } = catalog;
  return [
    '// Generated active Delve run-layout catalog from pinned source.',
    '// Tier indices are the source tier-array order. An out-of-range index maps',
    '// to source pickDelveModules\' unknown-tier fallback (tier zero).',
    '',
    renderRoute(delves),
    '',
    'pub delveIndexIsKnown(delveIndex: int): bool {',
    '    return routeForDelveIndex(delveIndex) >= 0;',
    '}',
    '',
    renderDelimitedValue('nonFinalModuleCount', 'Delve non-finale module count', delves,
      (delve) => String(delve.non_final_module_indices.length)),
    '',
    renderNestedIndexList('nonFinalModuleIndex', 'Delve non-finale module index', delves,
      (delve) => delve.non_final_module_indices),
    '',
    renderDelimitedValue('finaleModuleIndex', 'Delve finale module index', delves,
      (delve) => String(delve.finale_module_index)),
    '',
    renderDelimitedValue('tierCount', 'Delve tier count', delves,
      (delve) => String(delve.tier_ids.length)),
    '',
    'pub effectiveTierIndex(delveIndex: int, tierIndex: int, required: bool): int {',
    '    if (!required || routeForDelveIndex(delveIndex) < 0) {',
    '        throw "woc Delve tier is invalid";',
    '    }',
    '    return tierIndex >= 0 && tierIndex < tierCount(delveIndex, true) ? tierIndex : 0;',
    '}',
    '',
    renderTierCounts(delves),
    '',
    renderContentContractTest(delves),
    '',
  ].join('\n');
}

function renderRoute(delves) {
  const lines = ['routeForDelveIndex(delveIndex: int): int {'];
  for (let route = 0; route + 1 < delves.length; route++) {
    lines.push(`    if (delveIndex == ${delves[route].index}) {`,
      `        return ${route};`, '    }');
  }
  const last = delves.at(-1);
  lines.push(`    return delveIndex == ${last.index} ? ${delves.length - 1} : -1;`, '}');
  return lines.join('\n');
}

function renderDelimitedValue(name, label, delves, valueForDelve) {
  const lines = [
    `pub ${name}(delveIndex: int, required: bool): int {`,
    '    if (!required) {',
    `        throw "woc ${label} is required";`,
    '    }',
    '    var route = routeForDelveIndex(delveIndex);',
    '    if (route < 0) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let route = 0; route + 1 < delves.length; route++) {
    lines.push(`    if (route == ${route}) {`,
      `        return ${valueForDelve(delves[route])};`, '    }');
  }
  lines.push(`    return ${valueForDelve(delves.at(-1))};`, '}');
  return lines.join('\n');
}

function renderNestedIndexList(name, label, delves, valuesForDelve) {
  const lines = [
    `pub ${name}(delveIndex: int, moduleOffset: int, required: bool): int {`,
    '    if (!required || moduleOffset < 0) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
    '    var route = routeForDelveIndex(delveIndex);',
    '    if (route < 0) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let route = 0; route < delves.length; route++) {
    const values = valuesForDelve(delves[route]);
    lines.push(`    if (route == ${route}) {`,
      `        if (moduleOffset >= ${values.length}) {`,
      `            throw "woc ${label} is invalid";`,
      '        }');
    for (let index = 0; index + 1 < values.length; index++) {
      lines.push(`        if (moduleOffset == ${index}) {`,
        `            return ${values[index]};`, '        }');
    }
    lines.push(`        return ${values.at(-1)};`, '    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`, '}');
  return lines.join('\n');
}

function renderTierCounts(delves) {
  const lines = [
    'pub moduleCountForTier(delveIndex: int, tierIndex: int, required: bool): int {',
    '    if (!required) {',
    '        throw "woc Delve run module count is required";',
    '    }',
    '    var route = routeForDelveIndex(delveIndex);',
    '    if (route < 0) {',
    '        throw "woc Delve run module count is invalid";',
    '    }',
    '    var effective = effectiveTierIndex(delveIndex, tierIndex, true);',
  ];
  for (let route = 0; route < delves.length; route++) {
    const counts = delves[route].module_counts;
    lines.push(`    if (route == ${route}) {`);
    for (let tier = 0; tier + 1 < counts.length; tier++) {
      lines.push(`        if (effective == ${tier}) {`,
        `            return ${counts[tier]};`, '        }');
    }
    lines.push(`        return ${counts.at(-1)};`, '    }');
  }
  lines.push('    throw "woc Delve run module count is invalid";', '}');
  return lines.join('\n');
}

function renderContentContractTest(delves) {
  const lines = [
    'pub contractTest(): int {',
    `    if (${delves.map((delve) => `!delveIndexIsKnown(${delve.index})`).join(' || ')} || delveIndexIsKnown(99)) {`,
    '        return -1;',
    '    }',
  ];
  for (let index = 0; index < delves.length; index++) {
    const delve = delves[index];
    lines.push(`    if (nonFinalModuleCount(${delve.index}, true) != ${delve.non_final_module_indices.length} ||`,
      `        finaleModuleIndex(${delve.index}, true) != ${delve.finale_module_index} ||`,
      `        tierCount(${delve.index}, true) != ${delve.tier_ids.length} ||`,
      `        moduleCountForTier(${delve.index}, -1, true) != ${delve.module_counts[0]} ||`,
      `        moduleCountForTier(${delve.index}, ${delve.tier_ids.length - 1}, true) != ${delve.module_counts.at(-1)}) {`,
      `        return -${index + 2};`,
      '    }');
    for (let offset = 0; offset < delve.non_final_module_indices.length; offset++) {
      lines.push(`    if (nonFinalModuleIndex(${delve.index}, ${offset}, true) != ${delve.non_final_module_indices[offset]}) {`,
        `        return -${20 + index * 10 + offset};`,
        '    }');
    }
  }
  lines.push('    return 1;', '}');
  return lines.join('\n');
}

function renderLayoutZr(catalog) {
  const { delves, selection_vectors: vectors } = catalog;
  const maxPool = Math.max(...delves.map((delve) => delve.non_final_module_indices.length));
  return [
    '// Active Delve run-layout projection. Callers supply scalar fields from the',
    '// authoritative run: delve index, slot, seed, tier route and module offset.',
    '// It must not be used as the no-active-run default collision fallback.',
    '',
    'var content = %import("world/delve_run_layout_content");',
    'var collisionContent = %import("world/delve_collision_content");',
    'var rngModule = %import("kernel/rng");',
    '',
    'pub runModuleCount(delveIndex: int, tierIndex: int, required: bool): int {',
    '    if (!required) {',
    '        throw "woc active Delve run module count is required";',
    '    }',
    '    return content.moduleCountForTier(delveIndex, tierIndex, true) + 1;',
    '}',
    '',
    renderActiveModuleIndex(maxPool),
    '',
    renderActiveModuleOrigin(),
    '',
    renderActiveRunOccupancyRadius(),
    '',
    renderLayoutContractTest(vectors),
    '',
  ].join('\n');
}

function renderActiveModuleIndex(maxPool) {
  const lines = [
    'pub activeRunModuleIndex(',
    '    delveIndex: int,',
    '    tierIndex: int,',
    '    seed: int,',
    '    moduleOffset: int,',
    '    required: bool',
    '): int {',
    '    if (!required) {',
    '        throw "woc active Delve module query is required";',
    '    }',
    '    var runCount = runModuleCount(delveIndex, tierIndex, true);',
    '    if (moduleOffset < 0 || moduleOffset >= runCount) {',
    '        throw "woc active Delve module offset is invalid";',
    '    }',
    '    if (moduleOffset + 1 == runCount) {',
    '        return content.finaleModuleIndex(delveIndex, true);',
    '    }',
    '    var poolCount = content.nonFinalModuleCount(delveIndex, true);',
    '    var entry0 = content.nonFinalModuleIndex(delveIndex, 0, true);',
  ];
  for (let index = 1; index < maxPool; index++) {
    lines.push(`    var entry${index} = 0;`,
      `    if (poolCount > ${index}) {`,
      `        entry${index} = content.nonFinalModuleIndex(delveIndex, ${index}, true);`,
      '    }');
  }
  lines.push('    var rng = new rngModule.Mulberry32(<uint>seed);',
    '    var remaining = poolCount - 1;',
    '    while (remaining > 0) {',
    '        var selected = <int>(rng.next() * <float>(remaining + 1));');
  for (let position = maxPool - 1; position >= 1; position--) {
    lines.push(`        if (remaining == ${position}) {`,
      `            var swap${position} = entry${position};`);
    for (let selected = 0; selected < position; selected++) {
      lines.push(`            if (selected == ${selected}) {`,
        `                entry${position} = entry${selected};`,
        `                entry${selected} = swap${position};`,
        '            }');
    }
    lines.push('        }');
  }
  lines.push('        remaining = remaining - 1;',
    '    }');
  for (let index = 0; index + 1 < maxPool; index++) {
    lines.push(`    if (moduleOffset == ${index}) {`,
      `        return entry${index};`,
      '    }');
  }
  lines.push(`    return entry${maxPool - 1};`, '}');
  return lines.join('\n');
}

function renderActiveModuleOrigin() {
  return [
    'pub activeRunModuleOriginCoordinate(',
    '    delveIndex: int,',
    '    slot: int,',
    '    tierIndex: int,',
    '    seed: int,',
    '    moduleOffset: int,',
    '    axis: int,',
    '    required: bool',
    '): float {',
    '    if (!required || slot < 0 || slot >= collisionContent.delveSlotCount(true) ||',
    '        (axis != 1 && axis != 2)) {',
    '        throw "woc active Delve module origin is invalid";',
    '    }',
    '    activeRunModuleIndex(delveIndex, tierIndex, seed, moduleOffset, true);',
    '    var cursorZ = collisionContent.delveModuleZStart(true);',
    '    var previous = 0;',
    '    while (previous < moduleOffset) {',
    '        var previousModule = activeRunModuleIndex(',
    '            delveIndex, tierIndex, seed, previous, true',
    '        );',
    '        cursorZ = cursorZ + collisionContent.moduleSpan(previousModule, true) +',
    '            collisionContent.delveModuleGap(true);',
    '        previous = previous + 1;',
    '    }',
    '    return axis == 1 ?',
    '        collisionContent.delveOriginBaseX(true) +',
    '            <float>delveIndex * collisionContent.delveOriginIndexSpacing(true) :',
    '        collisionContent.delveOriginZ0(true) +',
    '            <float>slot * collisionContent.delveSlotSpacing(true) + cursorZ;',
    '}',
  ].join('\n');
}

function renderActiveRunOccupancyRadius() {
  return [
    '// Matches delveOccupancyRadius for the scalar active-run layout.',
    'pub activeRunOccupancyRadius(',
    '    delveIndex: int,',
    '    tierIndex: int,',
    '    seed: int,',
    '    required: bool',
    '): float {',
    '    if (!required) {',
    '        throw "woc active Delve occupancy radius is required";',
    '    }',
    '    var finalOffset = runModuleCount(delveIndex, tierIndex, true) - 1;',
    '    var cursorZ = collisionContent.delveModuleZStart(true);',
    '    var previous = 0;',
    '    while (previous < finalOffset) {',
    '        var previousModule = activeRunModuleIndex(',
    '            delveIndex, tierIndex, seed, previous, true',
    '        );',
    '        cursorZ = cursorZ + collisionContent.moduleSpan(previousModule, true) +',
    '            collisionContent.delveModuleGap(true);',
    '        previous = previous + 1;',
    '    }',
    '    var finalModule = activeRunModuleIndex(',
    '        delveIndex, tierIndex, seed, finalOffset, true',
    '    );',
    '    return cursorZ + collisionContent.moduleSpan(finalModule, true) + 40.0;',
    '}',
  ].join('\n');
}

function renderLayoutContractTest(vectors) {
  const lines = [
    'pub contractTest(): int {',
    '    if (runModuleCount(0, 0, true) != 4 || runModuleCount(1, 1, true) != 4 ||',
    '        runModuleCount(0, -1, true) != 4) {',
    '        return -1;',
    '    }',
  ];
  let failure = 2;
  for (const vector of vectors) {
    const tierIndex = vector.tier_id === 'normal' ? 0 :
      vector.tier_id === 'heroic' ? 1 : -1;
    for (let offset = 0; offset < vector.module_indices.length; offset++) {
      lines.push(`    if (activeRunModuleIndex(${vector.delve_index}, ${tierIndex}, ${vector.seed}, ${offset}, true) != ${vector.module_indices[offset]}) {`,
        `        return -${failure};`,
        '    }');
      failure += 1;
    }
  }
  lines.push(
    '    if (content.nonFinalModuleIndex(0, 0, true) ==',
    '        activeRunModuleIndex(0, 0, 1, 0, true)) {',
    `        return -${failure};`,
    '    }',
  );
  failure += 1;
  lines.push(
    '    if (activeRunOccupancyRadius(0, 0, 0, true) != 536.0 ||',
    '        activeRunOccupancyRadius(0, 1, 42, true) != 536.0) {',
    `        return -${failure};`,
    '    }',
  );
  failure += 1;
  lines.push(
    '    if (activeRunModuleOriginCoordinate(0, 0, 0, 1, 0, 1, true) != 4800.0 ||',
    '        activeRunModuleOriginCoordinate(0, 0, 0, 1, 0, 2, true) != -1242.0 ||',
    '        activeRunModuleOriginCoordinate(1, 3, 0, 42, 0, 1, true) != 5400.0 ||',
    '        activeRunModuleOriginCoordinate(1, 3, 0, 42, 0, 2, true) != 618.0) {',
    `        return -${failure};`,
    '    }',
    '    return 1;',
    '}',
  );
  return lines.join('\n');
}

function verifyOrWrite(path, text) {
  if (checkOnly) {
    assert(existsSync(path), `generated output is missing: ${path}`);
    assert(readFileSync(path, 'utf8') === text, `generated output drifted: ${path}`);
    return;
  }
  writeFileSync(path, text);
}

function sha256(text) {
  return createHash('sha256').update(text).digest('hex');
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
