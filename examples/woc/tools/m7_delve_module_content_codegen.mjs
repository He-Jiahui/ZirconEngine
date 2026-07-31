import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_MODULE_IDS = [
  'reliquary_sunken_ossuary', 'reliquary_bell_niche', 'reliquary_saintless_hall',
  'reliquary_finale', 'litany_sluice', 'litany_ledger', 'litany_ring',
  'litany_baptistry', 'litany_choir_loft', 'litany_causeway', 'litany_apse',
];
const EXPECTED_SPAWN_COUNTS = [4, 4, 3, 1, 6, 6, 5, 6, 6, 6, 1];
const EXPECTED_INTERACTABLE_COUNTS = [3, 0, 0, 3, 2, 4, 2, 0, 2, 0, 0];
const EXPECTED_PUZZLE_COUNTS = [2, 0, 0, 1, 2, 4, 2, 0, 2, 0, 0];
const EXPECTED_HAZARD_COUNTS = [0, 0, 0, 0, 6, 7, 4, 4, 9, 11, 4];
const EXPECTED_MODULES_SHA256 =
  '99219908159d0a895c8d64ed31b4a47cacb3a147e3f292ab9b08053703377574';
const EXPECTED_SOURCE_SHA256 = {
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
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
const extractorPath = join(scriptDirectory, 'm7_delve_module_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm7_delve_module_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances',
  'delve_module_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');
  const extracted = extract();
  const modules = extracted.modules;
  assert(JSON.stringify(modules.map((module) => module.id)) === JSON.stringify(EXPECTED_MODULE_IDS),
    'Delve module content identity/order drifted');
  assert(JSON.stringify(modules.map((module) => module.spawn_sets[0]?.spawns.length)) ===
    JSON.stringify(EXPECTED_SPAWN_COUNTS), 'Delve mob spawn counts drifted');
  assert(JSON.stringify(modules.map((module) => module.interactables.length)) ===
    JSON.stringify(EXPECTED_INTERACTABLE_COUNTS), 'Delve interactable counts drifted');
  assert(JSON.stringify(modules.map((module) => module.puzzle_interactable_count)) ===
    JSON.stringify(EXPECTED_PUZZLE_COUNTS), 'Delve puzzle counts drifted');
  assert(JSON.stringify(modules.map((module) => module.hazards.length)) ===
    JSON.stringify(EXPECTED_HAZARD_COUNTS), 'Delve hazard counts drifted');
  assert(modules.every((module) => module.spawn_sets.length === 1 &&
    module.spawn_sets[0].weight === 1 && module.spawn_sets[0].spawns.length > 0),
  'pinned source now requires weighted Delve spawn-set routing');
  assert(modules.every((module) =>
    module.puzzle_interactable_indices.length === module.puzzle_interactable_count &&
    module.puzzle_interactable_indices.every((index) =>
      Number.isInteger(index) && index >= 0 && index < module.interactables.length) &&
    module.puzzle_interactable_indices.length <= 4),
  'pinned source Delve puzzle index mapping is invalid');
  assert(sha256(JSON.stringify(modules)) === EXPECTED_MODULES_SHA256,
    'Delve module content drifted');

  const sourceTexts = Object.fromEntries(Object.keys(EXPECTED_SOURCE_SHA256)
    .map((path) => [path, gitShow(path)]));
  for (const [path, expectedHash] of Object.entries(EXPECTED_SOURCE_SHA256)) {
    assert(sha256(sourceTexts[path]) === expectedHash, `${path} drifted`);
  }
  assert(sourceTexts['src/sim/delves/runs.ts'].includes('pickDelveSpawnSet'),
    'source Delve spawn-set selection is absent');
  assert(sourceTexts['src/sim/delves/runs.ts'].includes('isDelvePuzzleKind'),
    'source Delve puzzle classification is absent');

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m7_delve_module_content_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts)
      .map(([path, text]) => [path, sha256(text)])),
    modules,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify(catalog.modules));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(catalog.modules));
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
  assert(child.status === 0, child.stderr || `Delve module extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function renderZr(modules) {
  return [
    '// Generated fixed Delve spawn, interactable and hazard catalog.',
    '// Module indices match world/delve_collision_content.zr source order.',
    '',
    'moduleIndexIsValid(moduleIndex: int): bool {',
    `    return moduleIndex >= 0 && moduleIndex < ${modules.length};`,
    '}',
    '',
    'pub moduleCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc Delve module content count is required";',
    '    }',
    `    return ${modules.length};`,
    '}',
    '',
    renderModuleValue('spawnSetCount', 'Delve spawn-set count', modules,
      (module) => String(module.spawn_sets.length)),
    '',
    renderSpawnSetValue('spawnSetWeight', 'Delve spawn-set weight', modules,
      (set) => String(set.weight)),
    '',
    renderSpawnSetValue('spawnCount', 'Delve spawn count', modules,
      (set) => String(set.spawns.length)),
    '',
    renderSpawnString(modules),
    '',
    renderSpawnCoordinate(modules),
    '',
    renderModuleValue('interactableCount', 'Delve interactable count', modules,
      (module) => String(module.interactables.length)),
    '',
    renderInteractableString(modules),
    '',
    renderInteractableCoordinate(modules),
    '',
    renderModuleValue('puzzleInteractableCount', 'Delve puzzle count', modules,
      (module) => String(module.puzzle_interactable_count)),
    '',
    renderPuzzleInteractableIndex(modules),
    '',
    renderModuleValue('hazardCount', 'Delve hazard count', modules,
      (module) => String(module.hazards.length)),
    '',
    renderHazardFloat(modules),
    '',
    renderHazardTier(modules),
    '',
    renderContractTest(modules),
    '',
  ].join('\n');
}

function renderModuleValue(name, label, modules, valueForModule) {
  const lines = [
    `pub ${name}(moduleIndex: int, required: bool): int {`,
    '    if (!required || !moduleIndexIsValid(moduleIndex)) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < modules.length; index++) {
    lines.push(`    if (moduleIndex == ${index}) {`,
      `        return ${valueForModule(modules[index])};`, '    }');
  }
  lines.push(`    return ${valueForModule(modules.at(-1))};`, '}');
  return lines.join('\n');
}

function renderSpawnSetValue(name, label, modules, valueForSet) {
  const lines = [
    `pub ${name}(moduleIndex: int, spawnSetIndex: int, required: bool): int {`,
    '    if (!required || !moduleIndexIsValid(moduleIndex) || spawnSetIndex < 0) {',
    `        throw "woc ${label} is invalid";`,
    '    }',
  ];
  for (let moduleIndex = 0; moduleIndex < modules.length; moduleIndex++) {
    const sets = modules[moduleIndex].spawn_sets;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`,
      `        if (spawnSetIndex >= ${sets.length}) {`,
      `            throw "woc ${label} is invalid";`,
      '        }');
    for (let index = 0; index + 1 < sets.length; index++) {
      lines.push(`        if (spawnSetIndex == ${index}) {`,
        `            return ${valueForSet(sets[index])};`, '        }');
    }
    lines.push(`        return ${valueForSet(sets.at(-1))};`, '    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`, '}');
  return lines.join('\n');
}

function renderSpawnString(modules) {
  const lines = [
    'pub spawnMobId(moduleIndex: int, spawnSetIndex: int, spawnIndex: int, required: bool): string {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || spawnSetIndex < 0 || spawnIndex < 0) {',
    '        throw "woc Delve spawn mob id is invalid";',
    '    }',
  ];
  renderNestedSpawns(lines, modules, 'Delve spawn mob id',
    (spawn) => `"${spawn.mob_id}"`);
  lines.push('}');
  return lines.join('\n');
}

function renderSpawnCoordinate(modules) {
  const lines = [
    'pub spawnCoordinate(',
    '    moduleIndex: int, spawnSetIndex: int, spawnIndex: int, axis: int, required: bool',
    '): float {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || spawnSetIndex < 0 ||',
    '        spawnIndex < 0 || (axis != 1 && axis != 2)) {',
    '        throw "woc Delve spawn coordinate is invalid";',
    '    }',
  ];
  renderNestedSpawns(lines, modules, 'Delve spawn coordinate',
    (spawn) => `axis == 1 ? ${formatNumber(spawn.x)} : ${formatNumber(spawn.z)}`);
  lines.push('}');
  return lines.join('\n');
}

function renderNestedSpawns(lines, modules, label, format) {
  for (let moduleIndex = 0; moduleIndex < modules.length; moduleIndex++) {
    const sets = modules[moduleIndex].spawn_sets;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`,
      `        if (spawnSetIndex >= ${sets.length}) {`,
      `            throw "woc ${label} is invalid";`,
      '        }');
    for (let setIndex = 0; setIndex < sets.length; setIndex++) {
      const spawns = sets[setIndex].spawns;
      lines.push(`        if (spawnSetIndex == ${setIndex}) {`,
        `            if (spawnIndex >= ${spawns.length}) {`,
        `                throw "woc ${label} is invalid";`,
        '            }');
      for (let spawnIndex = 0; spawnIndex + 1 < spawns.length; spawnIndex++) {
        lines.push(`            if (spawnIndex == ${spawnIndex}) {`,
          `                return ${format(spawns[spawnIndex])};`, '            }');
      }
      lines.push(`            return ${format(spawns.at(-1))};`, '        }');
    }
    lines.push(`        throw "woc ${label} is invalid";`, '    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`);
}

function renderInteractableString(modules) {
  const lines = [
    'pub interactableKind(moduleIndex: int, interactableIndex: int, required: bool): string {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || interactableIndex < 0) {',
    '        throw "woc Delve interactable kind is invalid";',
    '    }',
  ];
  renderNestedInteractables(lines, modules, 'Delve interactable kind',
    (entry) => `"${entry.kind}"`);
  lines.push('}');
  return lines.join('\n');
}

function renderInteractableCoordinate(modules) {
  const lines = [
    'pub interactableCoordinate(',
    '    moduleIndex: int, interactableIndex: int, axis: int, required: bool',
    '): float {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || interactableIndex < 0 ||',
    '        (axis != 1 && axis != 2)) {',
    '        throw "woc Delve interactable coordinate is invalid";',
    '    }',
  ];
  renderNestedInteractables(lines, modules, 'Delve interactable coordinate',
    (entry) => `axis == 1 ? ${formatNumber(entry.x)} : ${formatNumber(entry.z)}`);
  lines.push('}');
  return lines.join('\n');
}

function renderNestedInteractables(lines, modules, label, format) {
  for (let moduleIndex = 0; moduleIndex < modules.length; moduleIndex++) {
    const entries = modules[moduleIndex].interactables;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`,
      `        if (interactableIndex >= ${entries.length}) {`,
      `            throw "woc ${label} is invalid";`,
      '        }');
    if (entries.length > 0) {
      for (let entryIndex = 0; entryIndex + 1 < entries.length; entryIndex++) {
        lines.push(`        if (interactableIndex == ${entryIndex}) {`,
          `            return ${format(entries[entryIndex])};`, '        }');
      }
      lines.push(`        return ${format(entries.at(-1))};`);
    }
    lines.push('    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`);
}

function renderPuzzleInteractableIndex(modules) {
  const lines = [
    'pub puzzleInteractableIndex(moduleIndex: int, puzzleOffset: int, required: bool): int {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || puzzleOffset < 0) {',
    '        throw "woc Delve puzzle interactable index is invalid";',
    '    }',
  ];
  for (let moduleIndex = 0; moduleIndex < modules.length; moduleIndex++) {
    const indices = modules[moduleIndex].puzzle_interactable_indices;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`,
      `        if (puzzleOffset >= ${indices.length}) {`,
      '            throw "woc Delve puzzle interactable index is invalid";',
      '        }');
    if (indices.length > 0) {
      for (let index = 0; index + 1 < indices.length; index++) {
        lines.push(`        if (puzzleOffset == ${index}) {`,
          `            return ${indices[index]};`,
          '        }');
      }
      lines.push(`        return ${indices.at(-1)};`);
    }
    lines.push('    }');
  }
  lines.push('    throw "woc Delve puzzle interactable index is invalid";', '}');
  return lines.join('\n');
}

function renderHazardFloat(modules) {
  const lines = [
    'pub hazardFloat(moduleIndex: int, hazardIndex: int, field: int, required: bool): float {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || hazardIndex < 0 ||',
    '        field < 1 || field > 5) {',
    '        throw "woc Delve hazard float is invalid";',
    '    }',
  ];
  const fields = ['x', 'z', 'r', 'rx', 'rz'];
  renderNestedHazards(lines, modules, 'Delve hazard float', (hazard) => {
    const parts = fields.slice(0, -1).map((field, index) =>
      `field == ${index + 1} ? ${formatNumber(hazard[field])} : `).join('');
    return `${parts}${formatNumber(hazard.rz)}`;
  });
  lines.push('}');
  return lines.join('\n');
}

function renderHazardTier(modules) {
  const lines = [
    'pub hazardTierCode(moduleIndex: int, hazardIndex: int, required: bool): int {',
    '    if (!required || !moduleIndexIsValid(moduleIndex) || hazardIndex < 0) {',
    '        throw "woc Delve hazard tier is invalid";',
    '    }',
  ];
  renderNestedHazards(lines, modules, 'Delve hazard tier',
    (hazard) => hazard.tier === 'shallow' ? '1' : '2');
  lines.push('}');
  return lines.join('\n');
}

function renderNestedHazards(lines, modules, label, format) {
  for (let moduleIndex = 0; moduleIndex < modules.length; moduleIndex++) {
    const hazards = modules[moduleIndex].hazards;
    lines.push(`    if (moduleIndex == ${moduleIndex}) {`,
      `        if (hazardIndex >= ${hazards.length}) {`,
      `            throw "woc ${label} is invalid";`,
      '        }');
    if (hazards.length > 0) {
      for (let hazardIndex = 0; hazardIndex + 1 < hazards.length; hazardIndex++) {
        lines.push(`        if (hazardIndex == ${hazardIndex}) {`,
          `            return ${format(hazards[hazardIndex])};`, '        }');
      }
      lines.push(`        return ${format(hazards.at(-1))};`);
    }
    lines.push('    }');
  }
  lines.push(`    throw "woc ${label} is invalid";`);
}

function renderContractTest(modules) {
  const lines = [
    'pub contractTest(): int {',
    `    if (moduleCount(true) != ${modules.length} ||`,
    `        spawnCount(0, 0, true) != ${modules[0].spawn_sets[0].spawns.length} ||`,
    `        spawnMobId(0, 0, 0, true) != "${modules[0].spawn_sets[0].spawns[0].mob_id}" ||`,
    `        spawnCoordinate(0, 0, 0, 1, true) != ${formatNumber(modules[0].spawn_sets[0].spawns[0].x)} ||`,
    `        spawnCoordinate(0, 0, 0, 2, true) != ${formatNumber(modules[0].spawn_sets[0].spawns[0].z)} ||`,
    `        interactableCount(0, true) != ${modules[0].interactables.length} ||`,
    `        interactableKind(0, 0, true) != "${modules[0].interactables[0].kind}" ||`,
    `        puzzleInteractableCount(0, true) != ${modules[0].puzzle_interactable_count} ||`,
    `        puzzleInteractableIndex(0, 1, true) != ${modules[0].puzzle_interactable_indices[1]}) {`,
    '        return -1;',
    '    }',
    `    if (spawnCount(4, 0, true) != ${modules[4].spawn_sets[0].spawns.length} ||`,
    `        interactableCount(5, true) != ${modules[5].interactables.length} ||`,
    `        puzzleInteractableCount(5, true) != ${modules[5].puzzle_interactable_count} ||`,
    `        puzzleInteractableIndex(5, 3, true) != ${modules[5].puzzle_interactable_indices[3]} ||`,
    `        hazardCount(4, true) != ${modules[4].hazards.length} ||`,
    `        hazardFloat(4, 0, 1, true) != ${formatNumber(modules[4].hazards[0].x)} ||`,
    `        hazardTierCode(4, 0, true) != ${modules[4].hazards[0].tier === 'shallow' ? 1 : 2}) {`,
    '        return -2;',
    '    }',
    `    if (spawnCount(10, 0, true) != ${modules[10].spawn_sets[0].spawns.length} ||`,
    `        hazardCount(10, true) != ${modules[10].hazards.length}) {`,
    '        return -3;',
    '    }',
    '    return 1;',
    '}',
  ];
  return lines.join('\n');
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
