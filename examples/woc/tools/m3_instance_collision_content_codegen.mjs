import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_LAYOUT_IDS = ['crypt', 'sanctum', 'temple', 'nythraxis', 'arena', 'yumi'];
const EXPECTED_COLLIDER_COUNTS = [28, 24, 24, 30, 16, 48];
const EXPECTED_LAYOUTS_SHA256 = '7edf49ca9f2f92d9fd3a14a8c0b80e3ab30f8f5360f46be5943fe3e234a7f4a2';
const EXPECTED_DUNGEONS = [
  { id: 'hollow_crypt', index: 0, interior: 'crypt' },
  { id: 'sunken_bastion', index: 1, interior: 'crypt' },
  { id: 'gravewyrm_sanctum', index: 2, interior: 'sanctum' },
  { id: 'drowned_temple', index: 3, interior: 'temple' },
  { id: 'nythraxis_crypt', index: 4, interior: 'crypt' },
  { id: 'nythraxis_boss_arena', index: 5, interior: 'nythraxis' },
];
const EXPECTED_ROUTING = {
  dungeon_x_threshold: 600,
  dungeon_slot_count: 24,
  dungeon_origin_base_x: 900,
  dungeon_origin_index_spacing: 600,
  dungeon_origin_z0: -1250,
  dungeon_slot_spacing: 500,
  arena_x: 4200,
  arena_x_min: 4200,
  arena_slot_count: 4,
  arena_origin_z0: -1250,
  arena_slot_spacing: 120,
  delve_band_x_min: 4773,
  yumi_band_x_min: 8000,
  yumi_band_x_max: 12000,
  yumi_maze_x: 8400,
  yumi_maze_slot_count: 4,
  yumi_maze_origin_z0: -1250,
  yumi_maze_slot_spacing: 200,
  yumi_maze_seed: 212332557,
};
const EXPECTED_SIGHT_VECTORS = [
  { id: 'crypt_wall', from: { x: 870, z: -1203 }, to: { x: 880, z: -1203 }, radius: 0.05, clear: false, expected: false },
  { id: 'crypt_clear', from: { x: 900, z: -1240 }, to: { x: 905, z: -1240 }, radius: 0.05, clear: true, expected: true },
  { id: 'arena_wall', from: { x: 4170, z: -1250 }, to: { x: 4230, z: -1250 }, radius: 0.05, clear: false, expected: false },
  { id: 'arena_clear', from: { x: 4200, z: -1240 }, to: { x: 4200, z: -1230 }, radius: 0.05, clear: true, expected: true },
  { id: 'yumi_shell', from: { x: 8400, z: -1250 }, to: { x: 8400, z: -1305 }, radius: 0.05, clear: false, expected: false },
  { id: 'yumi_clear', from: { x: 8400, z: -1250 }, to: { x: 8400, z: -1240 }, radius: 0.05, clear: true, expected: true },
];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm3_instance_collision_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm3_instance_collision_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'world', 'instance_collision_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');
  const extracted = extract();
  assert(
    JSON.stringify(extracted.layouts.map((layout) => layout.id)) === JSON.stringify(EXPECTED_LAYOUT_IDS),
    'instance collision layout identity/order drifted',
  );
  assert(extracted.layouts.every((layout) => layout.colliders.length > 0), 'instance layout is empty');
  assert(extracted.layouts.every((layout) => layout.colliders.every(isValidCollider)), 'instance collider is invalid');
  assert(
    JSON.stringify(extracted.layouts.map((layout) => layout.colliders.length)) === JSON.stringify(EXPECTED_COLLIDER_COUNTS),
    'instance collider counts drifted',
  );
  assert(
    sha256(JSON.stringify(extracted.layouts)) === EXPECTED_LAYOUTS_SHA256,
    'instance collider content/order drifted',
  );
  assert(
    JSON.stringify(extracted.dungeons) === JSON.stringify(EXPECTED_DUNGEONS),
    'instance dungeon routing table drifted',
  );
  assert(JSON.stringify(extracted.routing) === JSON.stringify(EXPECTED_ROUTING), 'instance routing constants drifted');
  assert(JSON.stringify(extracted.sightVectors) === JSON.stringify(EXPECTED_SIGHT_VECTORS),
    'instance line-of-sight vectors drifted');

  const sourceTexts = Object.fromEntries([
    'src/sim/dungeon_layout.ts',
    'src/sim/colliders.ts',
    'src/sim/data.ts',
    'src/sim/yumi_maze_layout.ts',
  ].map((path) => [path, gitShow(path)]));
  const catalog = {
    schema_version: 3,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m3_instance_collision_content_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts).map(([path, text]) => [path, sha256(text)])),
    layouts: extracted.layouts,
    routing: extracted.routing,
    dungeons: extracted.dungeons,
    sight_vectors: extracted.sightVectors,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({
    layouts: catalog.layouts,
    routing: catalog.routing,
    dungeons: catalog.dungeons,
    sight_vectors: catalog.sight_vectors,
  }));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(catalog));
}

function extract() {
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `instance collision extractor exited ${child.status}`);
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
    maxBuffer: 16 * 1024 * 1024,
  });
}

function renderZr(catalog) {
  const { layouts, routing, dungeons } = catalog;
  const lines = [
    '// Generated instance-local layoutColliders projection from pinned dungeon_layout.ts.',
    '// Layout indices: 0=crypt, 1=sanctum, 2=temple, 3=nythraxis, 4=arena, 5=yumi.',
    '// Kinds: 1=circle, 2=rotated OBB. Float fields: 1=x, 2=z, 3=radius,',
    '// 4=half-width, 5=half-depth, 6=rotation.',
    '',
    'layoutIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${layouts.length};`,
    '}',
    '',
    'pub layoutCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc instance layout count is required";',
    '    }',
    `    return ${layouts.length};`,
    '}',
    '',
    renderLayoutColliderCount(layouts),
    '',
    renderKind(layouts),
    '',
    renderFloat(layouts),
    '',
    renderRouting(routing, dungeons),
    '',
    renderContractTest(layouts, routing, dungeons),
    '',
  ];
  return lines.join('\n');
}

function renderLayoutColliderCount(layouts) {
  return renderLayoutValue('layoutColliderCount', 'instance collider count', 'int', layouts,
    (layout) => String(layout.colliders.length));
}

function renderKind(layouts) {
  const lines = [
    'pub colliderKind(layoutIndex: int, colliderIndex: int, required: bool): int {',
    '    if (!required || !layoutIndexIsValid(layoutIndex)) {',
    '        throw "woc instance collider kind is invalid";',
    '    }',
  ];
  renderNestedCollider(lines, layouts, 'instance collider kind', (collider) => String(kindCode(collider)));
  lines.push('}');
  return lines.join('\n');
}

function renderFloat(layouts) {
  const fields = ['x', 'z', 'radius', 'half_width', 'half_depth', 'rotation'];
  const lines = [
    'pub colliderFloat(layoutIndex: int, colliderIndex: int, field: int, required: bool): float {',
    '    if (!required || !layoutIndexIsValid(layoutIndex) || field < 1 || field > 6) {',
    '        throw "woc instance collider float is invalid";',
    '    }',
  ];
  for (let layoutIndex = 0; layoutIndex < layouts.length; layoutIndex++) {
    const colliders = layouts[layoutIndex].colliders;
    lines.push(`    if (layoutIndex == ${layoutIndex}) {`);
    lines.push(`        if (colliderIndex < 0 || colliderIndex >= ${colliders.length}) {`);
    lines.push('            throw "woc instance collider float is invalid";');
    lines.push('        }');
    for (let colliderIndex = 0; colliderIndex + 1 < colliders.length; colliderIndex++) {
      lines.push(`        if (colliderIndex == ${colliderIndex}) {`);
      for (let field = 0; field + 1 < fields.length; field++) {
        lines.push(`            if (field == ${field + 1}) {`);
        lines.push(`                return ${formatNumber(colliders[colliderIndex][fields[field]])};`);
        lines.push('            }');
      }
      lines.push(`            return ${formatNumber(colliders[colliderIndex][fields.at(-1)])};`);
      lines.push('        }');
    }
    const final = colliders.at(-1);
    for (let field = 0; field + 1 < fields.length; field++) {
      lines.push(`        if (field == ${field + 1}) {`);
      lines.push(`            return ${formatNumber(final[fields[field]])};`);
      lines.push('        }');
    }
    lines.push(`        return ${formatNumber(final[fields.at(-1)])};`);
    lines.push('    }');
  }
  lines.push('    throw "woc instance collider float is invalid";', '}');
  return lines.join('\n');
}

function renderLayoutValue(name, label, type, layouts, format) {
  const lines = [
    `pub ${name}(layoutIndex: int, required: bool): ${type} {`,
    '    if (!required || !layoutIndexIsValid(layoutIndex)) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < layouts.length; index++) {
    lines.push(`    if (layoutIndex == ${index}) {`);
    lines.push(`        return ${format(layouts[index])};`);
    lines.push('    }');
  }
  lines.push(`    return ${format(layouts.at(-1))};`, '}');
  return lines.join('\n');
}

function renderDungeonRouteValue(name, label, type, dungeons, format) {
  const lines = [
    `pub ${name}(routeIndex: int, required: bool): ${type} {`,
    '    if (!required || !dungeonRouteIndexIsValid(routeIndex)) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < dungeons.length; index++) {
    lines.push(`    if (routeIndex == ${index}) {`);
    lines.push(`        return ${format(dungeons[index])};`);
    lines.push('    }');
  }
  lines.push(`    return ${format(dungeons.at(-1))};`, '}');
  return lines.join('\n');
}

function renderNestedCollider(lines, layouts, label, format) {
  for (let layoutIndex = 0; layoutIndex < layouts.length; layoutIndex++) {
    const colliders = layouts[layoutIndex].colliders;
    lines.push(`    if (layoutIndex == ${layoutIndex}) {`);
    lines.push(`        if (colliderIndex < 0 || colliderIndex >= ${colliders.length}) {`);
    lines.push(`            throw "woc ${label} is invalid";`);
    lines.push('        }');
    for (let colliderIndex = 0; colliderIndex + 1 < colliders.length; colliderIndex++) {
      lines.push(`        if (colliderIndex == ${colliderIndex}) {`);
      lines.push(`            return ${format(colliders[colliderIndex])};`);
      lines.push('        }');
    }
    lines.push(`        return ${format(colliders.at(-1))};`);
    lines.push('    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`);
}

function renderRouting(routing, dungeons) {
  const lines = [
    'dungeonRouteIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${dungeons.length};`,
    '}',
    '',
    'pub dungeonRouteCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc dungeon route count is required";',
    '    }',
    `    return ${dungeons.length};`,
    '}',
    '',
    renderDungeonRouteValue('dungeonIndexAtRoute', 'dungeon route index', 'int', dungeons,
      (dungeon) => String(dungeon.index)),
    '',
    renderDungeonRouteValue('dungeonLayoutIndexAtRoute', 'dungeon route layout', 'int', dungeons,
      (dungeon) => String(layoutIndexForInterior(dungeon.interior))),
    '',
  ];
  for (const [name, value] of Object.entries(routing)) {
    const title = name.split('_').map((part) => part[0].toUpperCase() + part.slice(1)).join('');
    const type = name.endsWith('_count') || name.endsWith('_seed') ? 'int' : 'float';
    lines.push(`pub ${title[0].toLowerCase() + title.slice(1)}(required: bool): ${type} {`);
    lines.push('    if (!required) {');
    lines.push(`        throw "woc instance ${name} is required";`);
    lines.push('    }');
    lines.push(`    return ${type === 'int' ? value : formatNumber(value)};`);
    lines.push('}', '');
  }
  return lines.join('\n');
}

function renderContractTest(layouts, routing, dungeons) {
  const lastLayout = layouts.length - 1;
  return [
    'pub contractTest(): int {',
    `    if (layoutCount(true) != ${layouts.length} || layoutColliderCount(0, true) != ${layouts[0].colliders.length} ||`,
    `        layoutColliderCount(${lastLayout}, true) != ${layouts.at(-1).colliders.length}) {`,
    '        return -1;',
    '    }',
    `    if (colliderKind(0, 0, true) != ${kindCode(layouts[0].colliders[0])} || colliderFloat(0, 1, 1, true) != ${formatNumber(layouts[0].colliders[1].x)} ||`,
    `        colliderKind(${lastLayout}, 0, true) != ${kindCode(layouts.at(-1).colliders[0])} || colliderFloat(${lastLayout}, 0, 2, true) != ${formatNumber(layouts.at(-1).colliders[0].z)}) {`,
    '        return -2;',
    '    }',
    `    if (dungeonRouteCount(true) != ${dungeons.length} || dungeonIndexAtRoute(0, true) != ${dungeons[0].index} ||`,
    `        dungeonIndexAtRoute(${dungeons.length - 1}, true) != ${dungeons.at(-1).index} || dungeonLayoutIndexAtRoute(2, true) != ${layoutIndexForInterior(dungeons[2].interior)} ||`,
    `        dungeonXThreshold(true) != ${formatNumber(routing.dungeon_x_threshold)} || dungeonSlotCount(true) != ${routing.dungeon_slot_count} ||`,
    `        dungeonOriginBaseX(true) != ${formatNumber(routing.dungeon_origin_base_x)} || dungeonOriginIndexSpacing(true) != ${formatNumber(routing.dungeon_origin_index_spacing)} ||`,
    `        dungeonOriginZ0(true) != ${formatNumber(routing.dungeon_origin_z0)} || dungeonSlotSpacing(true) != ${formatNumber(routing.dungeon_slot_spacing)} ||`,
    `        arenaX(true) != ${formatNumber(routing.arena_x)} || arenaSlotCount(true) != ${routing.arena_slot_count} || delveBandXMin(true) != ${formatNumber(routing.delve_band_x_min)} ||`,
    `        yumiBandXMin(true) != ${formatNumber(routing.yumi_band_x_min)} || yumiBandXMax(true) != ${formatNumber(routing.yumi_band_x_max)} || yumiMazeX(true) != ${formatNumber(routing.yumi_maze_x)} ||`,
    `        yumiMazeSlotCount(true) != ${routing.yumi_maze_slot_count} || yumiMazeOriginZ0(true) != ${formatNumber(routing.yumi_maze_origin_z0)} || yumiMazeSlotSpacing(true) != ${formatNumber(routing.yumi_maze_slot_spacing)} || yumiMazeSeed(true) != ${routing.yumi_maze_seed}) {`,
    '        return -3;',
    '    }',
    '    return 1;',
    '}',
  ].join('\n');
}

function layoutIndexForInterior(interior) {
  if (interior === 'crypt') return 0;
  if (interior === 'sanctum') return 1;
  if (interior === 'temple') return 2;
  if (interior === 'nythraxis') return 3;
  throw new Error(`unsupported instance interior ${interior}`);
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
