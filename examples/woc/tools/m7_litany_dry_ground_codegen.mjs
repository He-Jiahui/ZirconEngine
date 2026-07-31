import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_MODULE_IDS = [
  'litany_sluice', 'litany_ledger', 'litany_ring', 'litany_baptistry',
  'litany_choir_loft', 'litany_causeway', 'litany_apse',
];
const EXPECTED_MODULE_INDICES = [4, 5, 6, 7, 8, 9, 10];
const EXPECTED_ISLAND_COUNTS = [6, 7, 13, 5, 8, 8, 8];
const EXPECTED_MODULES_SHA256 =
  'f3ebc971950f8dadb1c75e3a2ec6ffd8e5767c691d06f19f59ee6a6b6d734060';
const EXPECTED_SOURCE_SHA256 = {
  'src/sim/delves/runs.ts': '374bf2a609668a5a0db62e7835bb76749ff85072509de2f96eb874fc05af65ff',
  'src/sim/delve_layout.ts': 'bab9792386316e45b9d04ae920d056755f1e7facaa2bec30d4dc6793dc3fdd1e',
  'src/sim/delve_litany_layout.ts': '47df22aecfe89de51f6dddf6725687ea70516ada159c89d20771ca7793915d2c',
};

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm7_litany_dry_ground_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm7_litany_dry_ground.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'instances',
  'delve_litany_dry_ground.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');

  const modules = extract().modules;
  assert(JSON.stringify(modules.map((module) => module.id)) === JSON.stringify(EXPECTED_MODULE_IDS),
    'Litany safe-ground module identity/order drifted');
  assert(JSON.stringify(modules.map((module) => module.module_index)) ===
    JSON.stringify(EXPECTED_MODULE_INDICES), 'Litany safe-ground module index drifted');
  assert(JSON.stringify(modules.map((module) => module.islands.length)) ===
    JSON.stringify(EXPECTED_ISLAND_COUNTS), 'Litany island counts drifted');
  assert(modules.every((module) => module.dais.r > 0 &&
    module.islands.every((island) => island.hw > 0 && island.hd > 0)),
  'Litany safe-ground geometry is invalid');
  assert(sha256(JSON.stringify(modules)) === EXPECTED_MODULES_SHA256,
    'Litany safe-ground geometry drifted');

  const sourceTexts = Object.fromEntries(Object.keys(EXPECTED_SOURCE_SHA256)
    .map((sourcePath) => [sourcePath, gitShow(sourcePath)]));
  for (const [sourcePath, expectedHash] of Object.entries(EXPECTED_SOURCE_SHA256)) {
    assert(sha256(sourceTexts[sourcePath]) === expectedHash, `${sourcePath} drifted`);
  }
  const runSource = sourceTexts['src/sim/delves/runs.ts'];
  assert(runSource.includes('function standingOnLitanyDryGround') &&
    runSource.includes('DELVE_BLACKWATER_PCT_NORMAL') &&
    runSource.includes('DELVE_BLACKWATER_PCT_HEROIC'),
  'source blackwater safe-ground or damage contract is absent');

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m7_litany_dry_ground_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(sourceTexts)
      .map(([sourcePath, text]) => [sourcePath, sha256(text)])),
    modules,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify(catalog.modules));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(modules));
}

function extract() {
  const child = spawnSync(process.execPath, [
    '--no-warnings', '--experimental-loader', loaderUrl, extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `Litany safe-ground extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function renderZr(modules) {
  const lines = [
    '// Generated authored Drowned Litany dais and island safe-ground geometry.',
    '// Module indices match instances/delve_module_content.zr source order.',
    '',
    'absolute(value: float): float {',
    '    return value < 0.0 ? -value : value;',
    '}',
    '',
    'pub isLitanyModuleIndex(moduleIndex: int, required: bool): bool {',
    '    if (!required || moduleIndex < 0 || moduleIndex >= 11) {',
    '        throw "woc Litany safe-ground module index is invalid";',
    '    }',
    `    return moduleIndex >= ${modules[0].module_index} && ` +
      `moduleIndex <= ${modules.at(-1).module_index};`,
    '}',
    '',
    'pub isDryGround(moduleIndex: int, localX: float, localZ: float, required: bool): bool {',
    '    if (!isLitanyModuleIndex(moduleIndex, required)) {',
    '        return false;',
    '    }',
  ];
  for (const module of modules) {
    lines.push(`    if (moduleIndex == ${module.module_index}) {`,
      `        var daisDx${module.module_index} = ${offset('localX', module.dais.x)};`,
      `        var daisDz${module.module_index} = ${offset('localZ', module.dais.z)};`,
      `        if (daisDx${module.module_index} * daisDx${module.module_index} + ` +
        `daisDz${module.module_index} * daisDz${module.module_index} <= ` +
        `${number(module.dais.r * module.dais.r)}) {`,
      '            return true;',
      '        }');
    for (const island of module.islands) {
      lines.push(`        if (absolute(${offset('localX', island.x)}) <= ${number(island.hw)} &&`,
        `            absolute(${offset('localZ', island.z)}) <= ${number(island.hd)}) {`,
        '            return true;',
        '        }');
    }
    lines.push('        return false;', '    }');
  }
  lines.push('    throw "woc Litany safe-ground module index is invalid";', '}', '',
    'pub contractTest(): int {',
    '    if (!isLitanyModuleIndex(4, true) || !isLitanyModuleIndex(10, true) ||',
    '        isLitanyModuleIndex(3, true)) {',
    '        return -1;',
    '    }',
    '    if (!isDryGround(4, 0.0, 59.0, true) || !isDryGround(4, 9.0, 5.0, true) ||',
    '        isDryGround(4, -8.0, 18.0, true)) {',
    '        return -2;',
    '    }',
    '    if (!isDryGround(6, -21.0, 26.0, true) || !isDryGround(7, 16.0, 34.0, true) ||',
    '        !isDryGround(8, 20.0, 28.0, true) || !isDryGround(9, 0.0, 82.0, true) ||',
    '        !isDryGround(10, 0.0, 72.0, true)) {',
    '        return -3;',
    '    }',
    '    if (isDryGround(0, 0.0, 0.0, true) || isDryGround(10, 12.1, 8.0, true)) {',
    '        return -4;',
    '    }',
    '    return 1;',
    '}',
    '',
  );
  return lines.join('\n');
}

function number(value) {
  assert(Number.isFinite(value), `cannot emit non-finite Litany safe-ground number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : value.toString();
}

function offset(variable, origin) {
  return origin < 0 ? `${variable} + ${number(-origin)}` : `${variable} - ${number(origin)}`;
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
