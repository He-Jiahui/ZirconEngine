import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_WAVE_COUNTS = [6, 6, 3];
const EXPECTED_CONTENT_SHA256 =
  '7e47e5ee102ba924fd64ea149d0aa72befc830310bf81e65480ec20dc2ef377d';
const EXPECTED_SOURCE_SHA256 = {
  'src/sim/delves/drowned_litany_rooms.ts':
    '4e2e8ac7282cd4fc846ff8a04350b82f5424db15dc7c695d23df505c907d3846',
  'src/sim/delves/runs.ts':
    '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
};

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm7_baptistry_content_source_extract.mjs');
const contractPath = join(projectRoot, 'contracts', 'm7_baptistry_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances',
  'delve_baptistry_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');

  const content = extract();
  assert(JSON.stringify(content.waves.map((wave) => wave.length)) === JSON.stringify(EXPECTED_WAVE_COUNTS),
    'Baptistry wave counts drifted');
  assert(content.egg_sac_spots.length === 3 && content.egg_sac_wave_radius === 7 &&
    content.egg_sac_wave_percent === 0.06 && content.egg_sac_burst_despawn === 1.1 &&
    content.hatchling_body_r === 0.8 && content.hatchling_spawn_attempts === 12,
  'Baptistry scalar content drifted');
  const contentHash = sha256(JSON.stringify(content));
  assert(contentHash === EXPECTED_CONTENT_SHA256, 'Baptistry content drifted');

  const sourceTexts = Object.fromEntries(Object.keys(EXPECTED_SOURCE_SHA256)
    .map((sourcePath) => [sourcePath, gitShow(sourcePath)]));
  for (const [sourcePath, expectedHash] of Object.entries(EXPECTED_SOURCE_SHA256)) {
    assert(sha256(sourceTexts[sourcePath]) === expectedHash, `${sourcePath} drifted`);
  }
  const roomSource = sourceTexts['src/sim/delves/drowned_litany_rooms.ts'];
  assert(roomSource.includes('tickLitanyBaptistryWaves') &&
    roomSource.includes('tickLitanyEggSacBursts') && roomSource.includes('onEggSacBurst'),
  'Baptistry source lifecycle hooks are absent');

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m7_baptistry_content_codegen.mjs',
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
  assert(child.status === 0, child.stderr || `Baptistry extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function renderZr(content) {
  return [
    '// Generated Sinkhole Baptistry waves, egg-sac positions and constants.',
    '// Source: src/sim/delves/drowned_litany_rooms.ts at the pinned WOC commit.',
    '',
    renderScalar('constantValue', 'Baptistry constant', [
      content.egg_sac_wave_radius,
      content.egg_sac_wave_percent,
      content.egg_sac_burst_despawn,
      content.hatchling_body_r,
      content.hatchling_spawn_attempts,
    ]),
    '',
    renderCount('waveCount', 'Baptistry wave count', content.waves.length),
    '',
    renderWaveCount(content.waves),
    '',
    renderWaveMobId(content.waves),
    '',
    renderWaveCoordinate(content.waves),
    '',
    renderCount('eggSacSpotCount', 'Baptistry egg-sac spot count', content.egg_sac_spots.length),
    '',
    renderSpotCoordinate(content.egg_sac_spots),
    '',
    renderContractTest(content),
    '',
  ].join('\n');
}

function renderScalar(name, label, values) {
  const lines = [
    `pub ${name}(field: int, required: bool): float {`,
    `    if (!required || field < 1 || field > ${values.length}) {`,
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < values.length; index++) {
    lines.push(`    if (field == ${index + 1}) {`, `        return ${number(values[index])};`, '    }');
  }
  lines.push(`    return ${number(values.at(-1))};`, '}');
  return lines.join('\n');
}

function renderCount(name, label, count) {
  return [
    `pub ${name}(required: bool): int {`,
    '    if (!required) {',
    `        throw "woc ${label} is required";`,
    '    }',
    `    return ${count};`,
    '}',
  ].join('\n');
}

function renderWaveCount(waves) {
  const lines = [
    'pub waveSpawnCount(waveIndex: int, required: bool): int {',
    `    if (!required || waveIndex < 0 || waveIndex >= ${waves.length}) {`,
    '        throw "woc Baptistry wave spawn count is invalid";',
    '    }',
  ];
  for (let index = 0; index + 1 < waves.length; index++) {
    lines.push(`    if (waveIndex == ${index}) {`, `        return ${waves[index].length};`, '    }');
  }
  lines.push(`    return ${waves.at(-1).length};`, '}');
  return lines.join('\n');
}

function renderWaveMobId(waves) {
  const lines = [
    'pub waveMobId(waveIndex: int, spawnIndex: int, required: bool): string {',
    `    if (!required || waveIndex < 0 || waveIndex >= ${waves.length} || spawnIndex < 0) {`,
    '        throw "woc Baptistry wave mob id is invalid";',
    '    }',
  ];
  renderNestedWaves(lines, waves, 'Baptistry wave mob id', (spawn) => `"${spawn.mob_id}"`);
  lines.push('}');
  return lines.join('\n');
}

function renderWaveCoordinate(waves) {
  const lines = [
    'pub waveCoordinate(waveIndex: int, spawnIndex: int, axis: int, required: bool): float {',
    `    if (!required || waveIndex < 0 || waveIndex >= ${waves.length} || spawnIndex < 0 ||`,
    '        (axis != 1 && axis != 2)) {',
    '        throw "woc Baptistry wave coordinate is invalid";',
    '    }',
  ];
  renderNestedWaves(lines, waves, 'Baptistry wave coordinate',
    (spawn) => `axis == 1 ? ${number(spawn.x)} : ${number(spawn.z)}`);
  lines.push('}');
  return lines.join('\n');
}

function renderNestedWaves(lines, waves, label, render) {
  for (let waveIndex = 0; waveIndex < waves.length; waveIndex++) {
    const wave = waves[waveIndex];
    lines.push(`    if (waveIndex == ${waveIndex}) {`,
      `        if (spawnIndex >= ${wave.length}) {`,
      `            throw "woc ${label} is invalid";`,
      '        }');
    for (let spawnIndex = 0; spawnIndex + 1 < wave.length; spawnIndex++) {
      lines.push(`        if (spawnIndex == ${spawnIndex}) {`,
        `            return ${render(wave[spawnIndex])};`, '        }');
    }
    lines.push(`        return ${render(wave.at(-1))};`, '    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`);
}

function renderSpotCoordinate(spots) {
  const lines = [
    'pub eggSacSpotCoordinate(spotIndex: int, axis: int, required: bool): float {',
    `    if (!required || spotIndex < 0 || spotIndex >= ${spots.length} ||`,
    '        (axis != 1 && axis != 2)) {',
    '        throw "woc Baptistry egg-sac spot coordinate is invalid";',
    '    }',
  ];
  for (let index = 0; index + 1 < spots.length; index++) {
    lines.push(`    if (spotIndex == ${index}) {`,
      `        return axis == 1 ? ${number(spots[index].x)} : ${number(spots[index].z)};`, '    }');
  }
  const last = spots.at(-1);
  lines.push(`    return axis == 1 ? ${number(last.x)} : ${number(last.z)};`, '}');
  return lines.join('\n');
}

function renderContractTest(content) {
  return [
    'pub contractTest(): int {',
    `    if (waveCount(true) != ${content.waves.length} ||`,
    `        waveSpawnCount(0, true) != ${content.waves[0].length} ||`,
    `        waveSpawnCount(1, true) != ${content.waves[1].length} ||`,
    `        waveSpawnCount(2, true) != ${content.waves[2].length} ||`,
    `        waveMobId(0, 0, true) != "${content.waves[0][0].mob_id}" ||`,
    `        waveCoordinate(1, 2, 2, true) != ${number(content.waves[1][2].z)}) {`,
    '        return -1;',
    '    }',
    `    if (eggSacSpotCount(true) != ${content.egg_sac_spots.length} ||`,
    `        eggSacSpotCoordinate(0, 1, true) != ${number(content.egg_sac_spots[0].x)} ||`,
    `        eggSacSpotCoordinate(2, 2, true) != ${number(content.egg_sac_spots[2].z)} ||`,
    `        constantValue(1, true) != ${number(content.egg_sac_wave_radius)} ||`,
    `        constantValue(2, true) != ${number(content.egg_sac_wave_percent)}) {`,
    '        return -2;',
    '    }',
    '    return 1;',
    '}',
  ].join('\n');
}

function number(value) {
  assert(Number.isFinite(value), `cannot emit non-finite Baptistry number ${value}`);
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
