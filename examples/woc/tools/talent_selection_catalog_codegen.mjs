import { spawnSync, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATHS = [
  'src/sim/content/talents.ts',
  'src/sim/content/talent_rows.ts',
  'src/sim/content/talents_warrior.ts',
  'src/sim/content/talents_classic.ts',
  'src/sim/content/warrior_rows.ts',
  'src/sim/content/choice_rows_classic.ts',
];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractorPath = join(scriptDirectory, 'talent_selection_catalog_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const abilityCatalogPath = join(projectRoot, 'reference', 'current-head', 'known_ability_catalog.json');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'talent_selection_catalog.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'current_talent_selection_catalog.zr');
const rustOutput = join(
  projectRoot,
  'native',
  'crates',
  'woc_protocol',
  'src',
  'generated_talent_selection_catalog.rs',
);
const checkOnly = process.argv.includes('--check');
const ROW_LEVELS = [5, 8, 11, 14, 17, 20];

main();

function main() {
  const sourceBlobs = Object.fromEntries(SOURCE_PATHS.map((path) => [path, sourceBlob(path)]));
  const extracted = extract();
  const abilityCatalog = readJson(abilityCatalogPath);
  invariant(abilityCatalog.source_commit === SOURCE_COMMIT,
    'known-ability catalog is not pinned to the current target');
  const abilityCodeById = new Map(abilityCatalog.abilities.map((ability) => [ability.id, ability.code]));
  const knownClassIds = new Set(abilityCatalog.classes.map((playerClass) => playerClass.id));

  invariant(Array.isArray(extracted.classes) && extracted.classes.length === 9,
    'current target must expose nine talent row trees');
  const seenClasses = new Set();
  const seenOptionIds = new Set();
  for (const playerClass of extracted.classes) {
    invariant(typeof playerClass.id === 'string' && knownClassIds.has(playerClass.id) &&
      !seenClasses.has(playerClass.id), `invalid talent class ${playerClass.id}`);
    seenClasses.add(playerClass.id);
    invariant(Array.isArray(playerClass.specs) && playerClass.specs.length === 3,
      `${playerClass.id} does not expose three specs`);
    invariant(Array.isArray(playerClass.rows) && playerClass.rows.length === ROW_LEVELS.length,
      `${playerClass.id} does not expose six talent rows`);
    const seenSpecs = new Set();
    for (const spec of playerClass.specs) {
      invariant(typeof spec.id === 'string' && spec.id.length > 0 && !seenSpecs.has(spec.id),
        `invalid spec for ${playerClass.id}`);
      invariant(abilityCodeById.has(spec.signature),
        `${playerClass.id} spec ${spec.id} references missing signature ${spec.signature}`);
      seenSpecs.add(spec.id);
    }
    const seenOptions = new Set();
    playerClass.rows.forEach((row, index) => {
      invariant(row.level === ROW_LEVELS[index] && Array.isArray(row.options) && row.options.length === 3,
        `${playerClass.id} talent row ${index} drifted`);
      for (const option of row.options) {
        invariant(typeof option.id === 'string' && option.id.length > 0 && !seenOptions.has(option.id),
          `invalid option in ${playerClass.id}`);
        invariant(!seenOptionIds.has(option.id),
          `talent option id ${option.id} is not globally unique`);
        invariant(option.grant_ability === null || abilityCodeById.has(option.grant_ability),
          `${playerClass.id} option ${option.id} references missing grant ${option.grant_ability}`);
        seenOptions.add(option.id);
        seenOptionIds.add(option.id);
      }
    });
  }

  let nextSpecCode = 1;
  let nextOptionCode = 1;
  const classes = extracted.classes.map((playerClass) => ({
    id: playerClass.id,
    specs: playerClass.specs.map((spec) => ({
      code: nextSpecCode++,
      id: spec.id,
      signature_ability_code: abilityCodeById.get(spec.signature),
    })),
    rows: playerClass.rows.map((row) => ({
      level: row.level,
      options: row.options.map((option) => ({
        code: nextOptionCode++,
        id: option.id,
        grant_ability_code: option.grant_ability === null ? 0 : abilityCodeById.get(option.grant_ability),
      })),
    })),
  }));
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/talent_selection_catalog_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(sourceBlobs)
      .map(([path, blob]) => [path, createHash('sha256').update(blob).digest('hex')])),
    known_ability_catalog_sha256: abilityCatalog.catalog_sha256,
    catalog_sha256: hashText(JSON.stringify(classes)),
    classes,
  };
  const json = `${JSON.stringify(document, null, 2)}\n`;
  const zr = renderZr(document);
  const rust = renderRust(document);
  for (const [path, output, label] of [
    [jsonOutput, json, 'talent selection catalog JSON'],
    [zrOutput, zr, 'talent selection catalog Zr'],
    [rustOutput, rust, 'talent selection catalog Rust'],
  ]) {
    if (checkOnly) {
      invariant(existsSync(path), `${label} is missing; run npm run generate:talent-selection-catalog`);
      invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:talent-selection-catalog`);
    } else {
      writeFileSync(path, output, 'utf8');
    }
  }
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} ${nextSpecCode - 1} specs and ` +
    `${nextOptionCode - 1} current talent options (${document.catalog_sha256.slice(0, 15)})\n`);
}

function extract() {
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
    'wocgit:///src/sim/content/talents.ts',
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `talent selection extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function renderZr(document) {
  const specs = document.classes.flatMap((playerClass) =>
    playerClass.specs.map((spec) => ({ ...spec, class_id: playerClass.id })));
  const options = document.classes.flatMap((playerClass) => playerClass.rows.flatMap((row) =>
    row.options.map((option) => ({ ...option, class_id: playerClass.id, level: row.level }))));
  return '// Generated by examples/woc/tools/talent_selection_catalog_codegen.mjs. Do not edit.\n' +
    `pub catalogSha(): string { return ${JSON.stringify(document.catalog_sha256)}; }\n` +
    `pub specCount(): int { return ${specs.length}; }\n` +
    `pub optionCount(): int { return ${options.length}; }\n\n` +
    'pub specCode(classId: string, id: string): uint {\n' +
    specs.map((spec) => `    if (classId == ${JSON.stringify(spec.class_id)} && id == ${JSON.stringify(spec.id)}) return <uint>${spec.code};`).join('\n') +
    '\n    return <uint>0;\n}\n\n' +
    'pub specId(code: uint): string {\n' +
    specs.map((spec) => `    if (code == <uint>${spec.code}) return ${JSON.stringify(spec.id)};`).join('\n') +
    '\n    return "";\n}\n\n' +
    'pub specMatchesClass(classId: string, code: uint): bool {\n' +
    specs.map((spec) => `    if (classId == ${JSON.stringify(spec.class_id)} && code == <uint>${spec.code}) return true;`).join('\n') +
    '\n    return false;\n}\n\n' +
    'pub specSignatureAbilityCode(code: uint): uint {\n' +
    specs.map((spec) => `    if (code == <uint>${spec.code}) return <uint>${spec.signature_ability_code};`).join('\n') +
    '\n    return <uint>0;\n}\n\n' +
    'pub optionCode(classId: string, level: int, id: string): uint {\n' +
    options.map((option) => `    if (classId == ${JSON.stringify(option.class_id)} && level == ${option.level} && id == ${JSON.stringify(option.id)}) return <uint>${option.code};`).join('\n') +
    '\n    return <uint>0;\n}\n\n' +
    'pub optionId(code: uint): string {\n' +
    options.map((option) => `    if (code == <uint>${option.code}) return ${JSON.stringify(option.id)};`).join('\n') +
    '\n    return "";\n}\n\n' +
    'pub optionMatchesClassRow(classId: string, level: int, code: uint): bool {\n' +
    options.map((option) => `    if (classId == ${JSON.stringify(option.class_id)} && level == ${option.level} && code == <uint>${option.code}) return true;`).join('\n') +
    '\n    return false;\n}\n\n' +
    'pub optionGrantAbilityCode(code: uint): uint {\n' +
    options.map((option) => `    if (code == <uint>${option.code}) return <uint>${option.grant_ability_code};`).join('\n') +
    '\n    return <uint>0;\n}\n';
}

function renderRust(document) {
  const specs = document.classes.flatMap((playerClass) =>
    playerClass.specs.map((spec) => ({ ...spec, class_id: playerClass.id })));
  const options = document.classes.flatMap((playerClass) => playerClass.rows.flatMap((row) =>
    row.options.map((option) => ({ ...option, class_id: playerClass.id, level: row.level }))));
  return '// Generated by examples/woc/tools/talent_selection_catalog_codegen.mjs. Do not edit.\n' +
    'pub const TALENT_SELECTION_CATALOG_SHA256: &str =\n' +
    `    ${JSON.stringify(document.catalog_sha256)};\n` +
    `pub const TALENT_SPEC_COUNT: u16 = ${specs.length};\n` +
    `pub const TALENT_OPTION_COUNT: u16 = ${options.length};\n\n` +
    'pub fn talent_spec_code(class_id: &str, id: &str) -> Option<u16> {\n' +
    '    match (class_id, id) {\n' +
    specs.map((spec) =>
      `        (${JSON.stringify(spec.class_id)}, ${JSON.stringify(spec.id)}) => Some(${spec.code}),`).join('\n') +
    '\n        _ => None,\n    }\n}\n\n' +
    'pub fn talent_spec_id(code: u16) -> Option<&\'static str> {\n' +
    '    match code {\n' +
    specs.map((spec) => `        ${spec.code} => Some(${JSON.stringify(spec.id)}),`).join('\n') +
    '\n        _ => None,\n    }\n}\n\n' +
    'pub fn talent_spec_matches_class(class_id: &str, code: u16) -> bool {\n' +
    '    match (class_id, code) {\n' +
    specs.map((spec) => `        (${JSON.stringify(spec.class_id)}, ${spec.code}) => true,`).join('\n') +
    '\n        _ => false,\n    }\n}\n\n' +
    'pub fn talent_option_code(id: &str) -> Option<u16> {\n' +
    '    match id {\n' +
    options.map((option) => `        ${JSON.stringify(option.id)} => Some(${option.code}),`).join('\n') +
    '\n        _ => None,\n    }\n}\n\n' +
    'pub fn talent_option_id(code: u16) -> Option<&\'static str> {\n' +
    '    match code {\n' +
    options.map((option) => `        ${option.code} => Some(${JSON.stringify(option.id)}),`).join('\n') +
    '\n        _ => None,\n    }\n}\n\n' +
    'pub fn talent_option_matches_class_row(class_id: &str, level: i32, code: u16) -> bool {\n' +
    '    match (class_id, level, code) {\n' +
    options.map((option) =>
      `        (${JSON.stringify(option.class_id)}, ${option.level}, ${option.code}) => true,`).join('\n') +
    '\n        _ => false,\n    }\n}\n';
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer', maxBuffer: 16 * 1024 * 1024,
  });
}

function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function hashText(value) { return createHash('sha256').update(value, 'utf8').digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
