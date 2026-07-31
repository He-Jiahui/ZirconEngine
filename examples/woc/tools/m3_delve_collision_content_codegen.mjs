import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_IDS = [
  'reliquary_sunken_ossuary', 'reliquary_bell_niche', 'reliquary_saintless_hall',
  'reliquary_finale', 'litany_sluice', 'litany_ledger', 'litany_ring',
  'litany_baptistry', 'litany_choir_loft', 'litany_causeway', 'litany_apse',
];
const EXPECTED_COUNTS = [24, 20, 24, 16, 43, 59, 61, 51, 59, 53, 63];
const EXPECTED_LAYOUTS_SHA256 = '883abb06d568943ce30dbd2760845b86261c2a76354af42f25e53d84043fd1e2';
const EXPECTED_CHAINS = [
  { id: 'collapsed_reliquary', index: 0, modules: [0, 1, 2, 3] },
  { id: 'drowned_litany', index: 1, modules: [4, 5, 6, 10] },
];
const EXPECTED_CHAINS_SHA256 = '45373e93256d5089a84b6c2fdc770a512ed8bc5d2adbc0e8c951b4d73dd919c8';
const EXPECTED_ROUTING = {
  delve_band_x_min: 4773,
  yumi_band_x_min: 8000,
  delve_origin_base_x: 4800,
  delve_origin_index_spacing: 600,
  delve_origin_z0: -1250,
  delve_slot_count: 24,
  delve_slot_spacing: 620,
  delve_module_gap: 16,
  delve_module_z_start: 8,
};
const EXPECTED_SOURCE_VECTORS = [
  {
    id: 'reliquary_side_wall', module_index: 0, world_x: 4774, world_z: -1206,
    origin_x: 4800, origin_z: -1242, resolved: { x: 4773.5, z: -1206 },
  },
  {
    id: 'litany_sluice_entry_slab', module_index: 4, world_x: 5395.333333333333,
    world_z: -1256, origin_x: 5400, origin_z: -1242,
    resolved: { x: 5395.333333333333, z: -1254.5 },
  },
];
const EXPECTED_MOVEMENT_VECTORS = [
  {
    id: 'reliquary_side_wall_sweep', module_index: 0, origin_x: 4800, origin_z: -1242,
    from_x: 4770, from_z: -1206, to_x: 4780, to_z: -1206,
    resolved: { x: 4773.5, z: -1206 },
  },
  {
    id: 'litany_sluice_slab_sweep', module_index: 4, origin_x: 5400, origin_z: -1242,
    from_x: 5395.333333333333, from_z: -1248, to_x: 5395.333333333333, to_z: -1262,
    resolved: { x: 5395.333333333333, z: -1254.5 },
  },
];
const EXPECTED_SIGHT_VECTORS = [
  { id: 'reliquary_wall', from: { x: 4770, z: -1206 }, to: { x: 4780, z: -1206 }, radius: 0.05, clear: false, expected: false },
  { id: 'reliquary_clear', from: { x: 4800, z: -1220 }, to: { x: 4802, z: -1220 }, radius: 0.05, clear: true, expected: true },
  { id: 'litany_slab', from: { x: 5395.333333333333, z: -1248 }, to: { x: 5395.333333333333, z: -1262 }, radius: 0.05, clear: false, expected: false },
  { id: 'litany_clear', from: { x: 5400, z: -1240 }, to: { x: 5400, z: -1230 }, radius: 0.05, clear: true, expected: true },
  { id: 'fallback_wall', from: { x: 5970, z: -1206 }, to: { x: 5980, z: -1206 }, radius: 0.05, clear: false, expected: false },
  { id: 'fallback_clear', from: { x: 6000, z: -1220 }, to: { x: 6002, z: -1220 }, radius: 0.05, clear: true, expected: true },
];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm3_delve_collision_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm3_delve_collision_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'world', 'delve_collision_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');
  const extracted = extract();
  assert(JSON.stringify(extracted.layouts.map((layout) => layout.id)) === JSON.stringify(EXPECTED_IDS),
    'Delve module identity/order drifted');
  assert(JSON.stringify(extracted.layouts.map((layout) => layout.colliders.length)) === JSON.stringify(EXPECTED_COUNTS),
    'Delve module collider counts drifted');
  assert(extracted.layouts.every((layout) => Number.isFinite(layout.span) && layout.span > 0),
    'Delve module span is invalid');
  assert(extracted.layouts.every((layout) => layout.colliders.every(isValidCollider)),
    'Delve module collider is invalid');
  assert(sha256(JSON.stringify(extracted.layouts)) === EXPECTED_LAYOUTS_SHA256,
    'Delve module layout content/order drifted');
  assert(JSON.stringify(extracted.defaultChains) === JSON.stringify(EXPECTED_CHAINS),
    'Delve default chain identity/order drifted');
  assert(sha256(JSON.stringify(extracted.defaultChains)) === EXPECTED_CHAINS_SHA256,
    'Delve default chain content/order drifted');
  assert(JSON.stringify(extracted.routing) === JSON.stringify(EXPECTED_ROUTING),
    'Delve routing constants drifted');
  assert(JSON.stringify(extracted.sourceVectors) === JSON.stringify(EXPECTED_SOURCE_VECTORS),
    'Delve collision source vectors drifted');
  assert(JSON.stringify(extracted.movementVectors) === JSON.stringify(EXPECTED_MOVEMENT_VECTORS),
    'Delve collision movement vectors drifted');
  assert(JSON.stringify(extracted.sightVectors) === JSON.stringify(EXPECTED_SIGHT_VECTORS),
    'Delve line-of-sight vectors drifted');

  const sourceTexts = Object.fromEntries([
    'src/sim/colliders.ts',
    'src/sim/data.ts',
    'src/sim/delve_layout.ts',
    'src/sim/delve_litany_layout.ts',
  ].map((path) => [path, gitShow(path)]));
  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m3_delve_collision_content_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts).map(([path, text]) => [path, sha256(text)])),
    layouts: extracted.layouts,
    default_chains: extracted.defaultChains,
    routing: extracted.routing,
    source_vectors: extracted.sourceVectors,
    movement_vectors: extracted.movementVectors,
    sight_vectors: extracted.sightVectors,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({
    layouts: catalog.layouts,
    default_chains: catalog.default_chains,
    routing: catalog.routing,
    source_vectors: catalog.source_vectors,
    movement_vectors: catalog.movement_vectors,
    sight_vectors: catalog.sight_vectors,
  }));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(catalog));
}

function extract() {
  const child = spawnSync(process.execPath, ['--no-warnings', '--experimental-loader', loaderUrl, extractorPath], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `Delve collision extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function isValidCollider(collider) {
  return (collider.kind === 'circle' || collider.kind === 'obb') &&
    [collider.x, collider.z, collider.radius, collider.half_width, collider.half_depth, collider.rotation]
      .every(Number.isFinite) && collider.radius >= 0 && collider.half_width >= 0 && collider.half_depth >= 0;
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function renderZr(catalog) {
  const { layouts, default_chains: chains, routing } = catalog;
  return [
    '// Generated fixed Delve module collision projection from pinned source.',
    '// Module indices are source-order reliquary then Litany layouts. Kinds:',
    '// 1=circle, 2=rotated OBB. Float fields: 1=x, 2=z, 3=radius,',
    '// 4=half-width, 5=half-depth, 6=rotation.',
    '',
    'moduleIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${layouts.length};`,
    '}',
    '',
    'pub moduleCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc Delve module count is required";',
    '    }',
    `    return ${layouts.length};`,
    '}',
    '',
    renderLayoutValue('moduleColliderCount', 'Delve module collider count', 'int', layouts,
      (layout) => String(layout.colliders.length)),
    '',
    renderLayoutValue('moduleSpan', 'Delve module span', 'float', layouts,
      (layout) => formatNumber(layout.span)),
    '',
    renderKind(layouts),
    '',
    renderFloat(layouts),
    '',
    renderChains(chains),
    '',
    renderConstants(routing),
    '',
    renderContractTest(layouts, chains, routing),
    '',
  ].join('\n');
}

function renderLayoutValue(name, label, type, layouts, format) {
  const lines = [
    `pub ${name}(moduleIndex: int, required: bool): ${type} {`,
    '    if (!required || !moduleIndexIsValid(moduleIndex)) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < layouts.length; index++) {
    lines.push(`    if (moduleIndex == ${index}) {`, `        return ${format(layouts[index])};`, '    }');
  }
  lines.push(`    return ${format(layouts.at(-1))};`, '}');
  return lines.join('\n');
}

function renderKind(layouts) {
  const lines = [
    'pub colliderKind(moduleIndex: int, colliderIndex: int, required: bool): int {',
    '    if (!required || !moduleIndexIsValid(moduleIndex)) {',
    '        throw "woc Delve collider kind is invalid";',
    '    }',
  ];
  renderNestedCollider(lines, layouts, 'Delve collider kind', (collider) => String(kindCode(collider)));
  lines.push('}');
  return lines.join('\n');
}

function renderFloat(layouts) {
  const fields = ['x', 'z', 'radius', 'half_width', 'half_depth', 'rotation'];
  const lines = [
    'pub colliderFloat(moduleIndex: int, colliderIndex: int, field: int, required: bool): float {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || field < 1 || field > 6) {',
    '        throw "woc Delve collider float is invalid";',
    '    }',
  ];
  for (let moduleIndex = 0; moduleIndex < layouts.length; moduleIndex++) {
    const colliders = layouts[moduleIndex].colliders;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`);
    lines.push(`        if (colliderIndex < 0 || colliderIndex >= ${colliders.length}) {`);
    lines.push('            throw "woc Delve collider float is invalid";', '        }');
    for (let colliderIndex = 0; colliderIndex + 1 < colliders.length; colliderIndex++) {
      lines.push(`        if (colliderIndex == ${colliderIndex}) {`);
      for (let field = 0; field + 1 < fields.length; field++) {
        lines.push(`            if (field == ${field + 1}) {`,
          `                return ${formatNumber(colliders[colliderIndex][fields[field]])};`, '            }');
      }
      lines.push(`            return ${formatNumber(colliders[colliderIndex][fields.at(-1)])};`, '        }');
    }
    const final = colliders.at(-1);
    for (let field = 0; field + 1 < fields.length; field++) {
      lines.push(`        if (field == ${field + 1}) {`,
        `            return ${formatNumber(final[fields[field]])};`, '        }');
    }
    lines.push(`        return ${formatNumber(final[fields.at(-1)])};`, '    }');
  }
  lines.push('    throw "woc Delve collider float is invalid";', '}');
  return lines.join('\n');
}

function renderNestedCollider(lines, layouts, label, format) {
  for (let moduleIndex = 0; moduleIndex < layouts.length; moduleIndex++) {
    const colliders = layouts[moduleIndex].colliders;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`);
    lines.push(`        if (colliderIndex < 0 || colliderIndex >= ${colliders.length}) {`);
    lines.push(`            throw "woc ${label} is invalid";`, '        }');
    for (let colliderIndex = 0; colliderIndex + 1 < colliders.length; colliderIndex++) {
      lines.push(`        if (colliderIndex == ${colliderIndex}) {`,
        `            return ${format(colliders[colliderIndex])};`, '        }');
    }
    lines.push(`        return ${format(colliders.at(-1))};`, '    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`);
}

function renderChains(chains) {
  const lines = [
    'defaultChainRouteForDelveIndex(delveIndex: int): int {',
  ];
  for (let route = 0; route + 1 < chains.length; route++) {
    lines.push(`    if (delveIndex == ${chains[route].index}) {`, `        return ${route};`, '    }');
  }
  lines.push(`    return delveIndex == ${chains.at(-1).index} ? ${chains.length - 1} : -1;`, '}', '');
  lines.push('pub defaultChainCount(delveIndex: int, required: bool): int {',
    '    if (!required) {', '        throw "woc Delve default chain count is required";', '    }',
    '    var route = defaultChainRouteForDelveIndex(delveIndex);',
    '    if (route < 0) {', '        throw "woc Delve default chain is invalid";', '    }');
  for (let route = 0; route + 1 < chains.length; route++) {
    lines.push(`    if (route == ${route}) {`, `        return ${chains[route].modules.length};`, '    }');
  }
  lines.push(`    return ${chains.at(-1).modules.length};`, '}', '');
  lines.push('pub defaultChainModuleIndex(delveIndex: int, moduleOffset: int, required: bool): int {',
    '    if (!required || moduleOffset < 0) {', '        throw "woc Delve default chain module is invalid";', '    }',
    '    var route = defaultChainRouteForDelveIndex(delveIndex);',
    '    if (route < 0 || moduleOffset >= defaultChainCount(delveIndex, true)) {',
    '        throw "woc Delve default chain module is invalid";', '    }');
  for (let route = 0; route < chains.length; route++) {
    const modules = chains[route].modules;
    lines.push(`    if (route == ${route}) {`);
    for (let offset = 0; offset + 1 < modules.length; offset++) {
      lines.push(`        if (moduleOffset == ${offset}) {`, `            return ${modules[offset]};`, '        }');
    }
    lines.push(`        return ${modules.at(-1)};`, '    }');
  }
  lines.push('    throw "woc Delve default chain module is invalid";', '}');
  return lines.join('\n');
}

function renderConstants(routing) {
  const lines = [];
  for (const [name, value] of Object.entries(routing)) {
    const title = name.split('_').map((part) => part[0].toUpperCase() + part.slice(1)).join('');
    const type = name.endsWith('_count') ? 'int' : 'float';
    lines.push(`pub ${title[0].toLowerCase() + title.slice(1)}(required: bool): ${type} {`,
      '    if (!required) {', `        throw "woc Delve ${name} is required";`, '    }',
      `    return ${type === 'int' ? value : formatNumber(value)};`, '}', '');
  }
  return lines.join('\n');
}

function renderContractTest(layouts, chains, routing) {
  const last = layouts.length - 1;
  return [
    'pub contractTest(): int {',
    `    if (moduleCount(true) != ${layouts.length} || moduleColliderCount(0, true) != ${layouts[0].colliders.length} || moduleSpan(0, true) != ${formatNumber(layouts[0].span)} ||`,
    `        moduleColliderCount(${last}, true) != ${layouts.at(-1).colliders.length} || moduleSpan(${last}, true) != ${formatNumber(layouts.at(-1).span)} ||`,
    `        colliderKind(0, 0, true) != ${kindCode(layouts[0].colliders[0])} || colliderFloat(0, 0, 1, true) != ${formatNumber(layouts[0].colliders[0].x)} ||`,
    `        colliderKind(${last}, 0, true) != ${kindCode(layouts.at(-1).colliders[0])} || colliderFloat(${last}, 0, 2, true) != ${formatNumber(layouts.at(-1).colliders[0].z)}) {`,
    '        return -1;', '    }',
    `    if (defaultChainCount(0, true) != ${chains[0].modules.length} || defaultChainModuleIndex(0, 0, true) != ${chains[0].modules[0]} ||`,
    `        defaultChainModuleIndex(0, ${chains[0].modules.length - 1}, true) != ${chains[0].modules.at(-1)} || defaultChainCount(1, true) != ${chains[1].modules.length} ||`,
    `        defaultChainModuleIndex(1, ${chains[1].modules.length - 1}, true) != ${chains[1].modules.at(-1)}) {`,
    '        return -2;', '    }',
    `    if (delveBandXMin(true) != ${formatNumber(routing.delve_band_x_min)} || yumiBandXMin(true) != ${formatNumber(routing.yumi_band_x_min)} ||`,
    `        delveOriginBaseX(true) != ${formatNumber(routing.delve_origin_base_x)} || delveOriginIndexSpacing(true) != ${formatNumber(routing.delve_origin_index_spacing)} ||`,
    `        delveOriginZ0(true) != ${formatNumber(routing.delve_origin_z0)} || delveSlotCount(true) != ${routing.delve_slot_count} ||`,
    `        delveSlotSpacing(true) != ${formatNumber(routing.delve_slot_spacing)} || delveModuleGap(true) != ${formatNumber(routing.delve_module_gap)} ||`,
    `        delveModuleZStart(true) != ${formatNumber(routing.delve_module_z_start)}) {`,
    '        return -3;', '    }',
    '    return 1;', '}',
  ].join('\n');
}

function kindCode(collider) {
  return collider.kind === 'circle' ? 1 : 2;
}

function formatNumber(value) {
  assert(Number.isFinite(value), `cannot emit non-finite number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : value.toString();
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
