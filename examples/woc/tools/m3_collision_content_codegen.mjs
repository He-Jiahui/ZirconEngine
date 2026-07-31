import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_COLLIDER_COUNT = 170;
const EXPECTED_CIRCLE_COUNT = 134;
const EXPECTED_OBB_COUNT = 36;
const EXPECTED_FENCE_COUNT = 6;
const EXPECTED_COLLIDER_SHA256 = 'acd0173730e7d2b8de3646d816d9b7ea5e6acdd4b9c9f1d0c3988234099514b4';
const EXPECTED_FENCE_SEGMENTS = [
  { x1: 16, z1: 16, x2: 22, z2: 4 },
  { x1: -16, z1: 14, x2: -20, z2: 2 },
  { x1: 16, z1: 311, x2: 21, z2: 299 },
  { x1: -18, z1: 313, x2: -22, z2: 300 },
  { x1: -14, z1: 649, x2: -4, z2: 647 },
  { x1: 4, z1: 647, x2: 14, z2: 649 },
];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm3_collision_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm3_collision_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'world', 'collision_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');
  const extracted = extract();
  assert(extracted.colliders.length === EXPECTED_COLLIDER_COUNT, 'builtin collision count drifted');
  assert(extracted.colliders.every(isValidCollider), 'builtin collision record is invalid');
  assert(
    extracted.colliders.filter((collider) => collider.kind === 'circle').length === EXPECTED_CIRCLE_COUNT,
    'builtin circle count drifted',
  );
  assert(
    extracted.colliders.filter((collider) => collider.kind === 'obb').length === EXPECTED_OBB_COUNT,
    'builtin OBB count drifted',
  );
  assert(
    extracted.colliders.filter((collider) => collider.is_fence).length === EXPECTED_FENCE_COUNT,
    'builtin fence count drifted',
  );
  assert(
    sha256(JSON.stringify(extracted.colliders)) === EXPECTED_COLLIDER_SHA256,
    'builtin collision content/order drifted',
  );
  assert(
    JSON.stringify(extracted.fence_segments) === JSON.stringify(EXPECTED_FENCE_SEGMENTS),
    'builtin fence segment content/order drifted',
  );

  const sourceTexts = Object.fromEntries([
    'src/sim/colliders.ts',
    'src/sim/data.ts',
    'src/sim/content/zone1.ts',
    'src/sim/content/zone2.ts',
    'src/sim/content/zone3.ts',
    'src/sim/vale_cup_layout.ts',
  ].map((sourcePath) => [sourcePath, gitShow(sourcePath)]));
  const catalog = {
    schema_version: 2,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m3_collision_content_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts).map(([path, text]) => [path, sha256(text)])),
    colliders: extracted.colliders,
    fence_segments: extracted.fence_segments,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({
    colliders: catalog.colliders,
    fence_segments: catalog.fence_segments,
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
  assert(child.status === 0, child.stderr || `collision source extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function isValidCollider(collider) {
  return (collider.kind === 'circle' || collider.kind === 'obb') &&
    typeof collider.is_fence === 'boolean' &&
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
  const { colliders, fence_segments: fenceSegments } = catalog;
  const last = colliders.length - 1;
  return [
    '// Generated from pinned staticWorldColliders data before procedural decorations.',
    '// Collision kinds: 1=circle, 2=rotated OBB. Float fields: 1=x, 2=z,',
    '// 3=radius, 4=half-width, 5=half-depth, 6=rotation.',
    '',
    'colliderIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${colliders.length};`,
    '}',
    '',
    'pub colliderCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc collision count is required";',
    '    }',
    `    return ${colliders.length};`,
    '}',
    '',
    renderKind(colliders),
    '',
    renderFence(colliders),
    '',
    renderFloat(colliders),
    '',
    renderFenceSegments(fenceSegments),
    '',
    'pub contractTest(): int {',
    `    if (colliderCount(true) != ${colliders.length} || colliderKind(0, true) != ${kindCode(colliders[0])} ||`,
    `        colliderKind(${last}, true) != ${kindCode(colliders[last])} || colliderIsFence(0, true) != ${colliders[0].is_fence ? 'true' : 'false'} ||`,
    `        colliderIsFence(${last}, true) != ${colliders[last].is_fence ? 'true' : 'false'}) {`,
    '        return -1;',
    '    }',
    `    if (colliderFloat(0, 1, true) != ${formatNumber(colliders[0].x)} || colliderFloat(0, 2, true) != ${formatNumber(colliders[0].z)} ||`,
    `        colliderFloat(${last}, 4, true) != ${formatNumber(colliders[last].half_width)} || colliderFloat(${last}, 5, true) != ${formatNumber(colliders[last].half_depth)}) {`,
    '        return -2;',
    '    }',
    `    if (fenceSegmentCount(true) != ${fenceSegments.length} || fenceSegmentFloat(0, 1, true) != ${formatNumber(fenceSegments[0].x1)} ||`,
    `        fenceSegmentFloat(${fenceSegments.length - 1}, 4, true) != ${formatNumber(fenceSegments.at(-1).z2)}) {`,
    '        return -3;',
    '    }',
    '    return 1;',
    '}',
    '',
  ].join('\n');
}

function renderKind(colliders) {
  return renderPerCollider('colliderKind', 'collision kind', 'int', colliders, (collider) => String(kindCode(collider)));
}

function renderFence(colliders) {
  return renderPerCollider('colliderIsFence', 'collision fence flag', 'bool', colliders, (collider) => collider.is_fence ? 'true' : 'false');
}

function renderPerCollider(name, label, type, colliders, format) {
  const lines = [
    `pub ${name}(index: int, required: bool): ${type} {`,
    '    if (!required || !colliderIndexIsValid(index)) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < colliders.length; index++) {
    lines.push(`    if (index == ${index}) {`);
    lines.push(`        return ${format(colliders[index])};`);
    lines.push('    }');
  }
  lines.push(`    return ${format(colliders.at(-1))};`, '}');
  return lines.join('\n');
}

function renderFloat(colliders) {
  const fields = ['x', 'z', 'radius', 'half_width', 'half_depth', 'rotation'];
  const lines = [
    'pub colliderFloat(index: int, field: int, required: bool): float {',
    '    if (!required || !colliderIndexIsValid(index) || field < 1 || field > 6) {',
    '        throw "woc collision float field is invalid";',
    '    }',
  ];
  for (let index = 0; index + 1 < colliders.length; index++) {
    lines.push(`    if (index == ${index}) {`);
    for (let field = 0; field + 1 < fields.length; field++) {
      lines.push(`        if (field == ${field + 1}) {`);
      lines.push(`            return ${formatNumber(colliders[index][fields[field]])};`);
      lines.push('        }');
    }
    lines.push(`        return ${formatNumber(colliders[index][fields.at(-1)])};`);
    lines.push('    }');
  }
  const final = colliders.at(-1);
  for (let field = 0; field + 1 < fields.length; field++) {
    lines.push(`    if (field == ${field + 1}) {`);
    lines.push(`        return ${formatNumber(final[fields[field]])};`);
    lines.push('    }');
  }
  lines.push(`    return ${formatNumber(final[fields.at(-1)])};`, '}');
  return lines.join('\n');
}

function renderFenceSegments(fenceSegments) {
  const fields = ['x1', 'z1', 'x2', 'z2'];
  const lines = [
    'fenceSegmentIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${fenceSegments.length};`,
    '}',
    '',
    'pub fenceSegmentCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc fence segment count is required";',
    '    }',
    `    return ${fenceSegments.length};`,
    '}',
    '',
    'pub fenceSegmentFloat(index: int, field: int, required: bool): float {',
    '    if (!required || !fenceSegmentIndexIsValid(index) || field < 1 || field > 4) {',
    '        throw "woc fence segment field is invalid";',
    '    }',
  ];
  for (let index = 0; index + 1 < fenceSegments.length; index++) {
    lines.push(`    if (index == ${index}) {`);
    for (let field = 0; field + 1 < fields.length; field++) {
      lines.push(`        if (field == ${field + 1}) {`);
      lines.push(`            return ${formatNumber(fenceSegments[index][fields[field]])};`);
      lines.push('        }');
    }
    lines.push(`        return ${formatNumber(fenceSegments[index][fields.at(-1)])};`);
    lines.push('    }');
  }
  const final = fenceSegments.at(-1);
  for (let field = 0; field + 1 < fields.length; field++) {
    lines.push(`    if (field == ${field + 1}) {`);
    lines.push(`        return ${formatNumber(final[fields[field]])};`);
    lines.push('    }');
  }
  lines.push(`    return ${formatNumber(final[fields.at(-1)])};`, '}');
  return lines.join('\n');
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
