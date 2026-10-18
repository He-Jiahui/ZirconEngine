import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const knownCatalogPath = join(projectRoot, 'reference', 'current-head', 'known_ability_catalog.json');
const m4CatalogPath = join(projectRoot, 'contracts', 'm4_abilities.json');
const outputPath = join(projectRoot, 'reference', 'current-head', 'm4_ability_projection_coverage.json');
const extractorPath = join(scriptDirectory, 'm4_ability_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const classesModule = 'wocgit:///src/sim/content/classes.ts';
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const knownCatalog = readJson(knownCatalogPath);
  const m4Catalog = readJson(m4CatalogPath);
  validateInputs(knownCatalog, m4Catalog);

  const m4ById = new Map(m4Catalog.entries.map((entry) => [entry.id, entry]));
  const sourceById = loadSourceDefinitions(knownCatalog.abilities.map((ability) => ability.id));
  const m4EffectTypes = new Set(
    m4Catalog.entries.flatMap((entry) => entry.definition.effects.map((effect) => effect.type)),
  );
  const entries = knownCatalog.abilities.map((ability) => {
    const m4 = m4ById.get(ability.id);
    const definition = sourceById.get(ability.id);
    const sourceEffectTypes = definition.effects.map((effect) => effect.type);
    return {
      code: ability.code,
      id: ability.id,
      class_id: ability.class_id,
      learn_level: ability.learn_level,
      school: ability.school,
      m4_projection: m4 ? 'm4_projected' : 'not_m4_projected',
      m4_index: m4?.index ?? null,
      m4_scenarios: m4?.scenarios ?? [],
      m4_source_owner: m4?.source_owner ?? null,
      source_effect_types: sourceEffectTypes,
      m4_effect_shape_status: effectShapeStatus(m4, sourceEffectTypes, m4EffectTypes),
    };
  });
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m4_ability_projection_coverage_codegen.mjs',
    known_ability_catalog_sha256: knownCatalog.catalog_sha256,
    m4_ability_catalog_sha256: m4Catalog.catalog_sha256,
    scope_note: 'Projection and source effect-shape status only; not_m4_projected does not assert that an ability is unimplemented elsewhere in WOC.',
    totals: {
      current_known_abilities: entries.length,
      m4_projected_abilities: entries.filter((entry) => entry.m4_projection === 'm4_projected').length,
      not_m4_projected_abilities: entries.filter((entry) => entry.m4_projection === 'not_m4_projected').length,
    },
    entries,
  };
  validateCoverage(document, m4Catalog.entries);
  const rendered = `${JSON.stringify(document, null, 2)}\n`;
  if (checkOnly) {
    invariant(existsSync(outputPath), 'M4 ability projection coverage is missing; run the generator');
    invariant(readFileSync(outputPath, 'utf8') === rendered, 'M4 ability projection coverage is stale; run the generator');
  } else {
    writeFileSync(outputPath, rendered, 'utf8');
  }
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} ${document.totals.m4_projected_abilities}/` +
    `${document.totals.current_known_abilities} current abilities in the M4 projection\n`,
  );
}

function loadSourceDefinitions(abilityIds) {
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
    classesModule,
    ...abilityIds,
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `ability effect extractor exited ${child.status}`);
  const definitions = JSON.parse(child.stdout);
  invariant(Array.isArray(definitions) && definitions.length === abilityIds.length,
    'source ability effect extraction is incomplete');
  const sourceById = new Map();
  for (const definition of definitions) {
    invariant(typeof definition?.id === 'string' && definition.id.length > 0,
      'source ability definition is invalid');
    invariant(Array.isArray(definition.effects), `source ability effects are invalid: ${definition.id}`);
    invariant(!sourceById.has(definition.id), `duplicate source ability definition: ${definition.id}`);
    sourceById.set(definition.id, definition);
  }
  for (const id of abilityIds) invariant(sourceById.has(id), `source ability definition is missing: ${id}`);
  return sourceById;
}

function effectShapeStatus(m4, sourceEffectTypes, m4EffectTypes) {
  if (m4) return 'm4_projected';
  if (sourceEffectTypes.length === 0) return 'no_active_effect';
  return sourceEffectTypes.every((effectType) => m4EffectTypes.has(effectType))
    ? 'existing_m4_effect_shapes'
    : 'new_m4_effect_shape_required';
}

function validateInputs(knownCatalog, m4Catalog) {
  invariant(knownCatalog.source_commit === SOURCE_COMMIT, 'known ability catalog source commit drifted');
  invariant(m4Catalog.source_commit === SOURCE_COMMIT, 'M4 ability catalog source commit drifted');
  invariant(Array.isArray(knownCatalog.abilities) && knownCatalog.abilities.length === 308,
    'current known ability count drifted');
invariant(Array.isArray(m4Catalog.entries) && m4Catalog.entries.length === 117,
    'M4 ability count drifted');
  const knownIds = new Set();
  for (const ability of knownCatalog.abilities) {
    invariant(typeof ability.id === 'string' && ability.id.length > 0, 'known ability id is invalid');
    invariant(!knownIds.has(ability.id), `duplicate known ability ${ability.id}`);
    knownIds.add(ability.id);
    invariant(Number.isInteger(ability.code) && ability.code > 0, `known ability code is invalid: ${ability.id}`);
  }
  const m4Ids = new Set();
  for (const entry of m4Catalog.entries) {
    invariant(typeof entry.id === 'string' && entry.id.length > 0, 'M4 ability id is invalid');
    invariant(!m4Ids.has(entry.id), `duplicate M4 ability ${entry.id}`);
    m4Ids.add(entry.id);
    invariant(knownIds.has(entry.id), `M4 ability is absent from current known catalog: ${entry.id}`);
    invariant(Number.isInteger(entry.index) && entry.index >= 0, `M4 ability index is invalid: ${entry.id}`);
    invariant(Array.isArray(entry.scenarios), `M4 ability scenarios are invalid: ${entry.id}`);
  }
}

function validateCoverage(document, m4Entries) {
  invariant(document.entries.length === 308, 'coverage known ability count drifted');
invariant(document.totals.m4_projected_abilities === 117, 'coverage M4 projected count drifted');
invariant(document.totals.not_m4_projected_abilities === 191, 'coverage non-M4 projected count drifted');
  const covered = document.entries.filter((entry) => entry.m4_projection === 'm4_projected');
  invariant(covered.length === m4Entries.length, 'M4 coverage cardinality drifted');
  for (const entry of covered) {
    invariant(Number.isInteger(entry.m4_index), `covered ability has no M4 index: ${entry.id}`);
    invariant(entry.m4_source_owner !== null, `covered ability has no M4 source owner: ${entry.id}`);
    invariant(entry.m4_effect_shape_status === 'm4_projected',
      `covered ability has an invalid M4 effect-shape status: ${entry.id}`);
  }
  for (const entry of document.entries.filter((entry) => entry.m4_projection === 'not_m4_projected')) {
    invariant(entry.m4_index === null && entry.m4_scenarios.length === 0 && entry.m4_source_owner === null,
      `non-M4 ability has M4 metadata: ${entry.id}`);
    invariant(entry.m4_effect_shape_status === 'no_active_effect' ||
      entry.m4_effect_shape_status === 'existing_m4_effect_shapes' ||
      entry.m4_effect_shape_status === 'new_m4_effect_shape_required',
    `non-M4 ability has an invalid M4 effect-shape status: ${entry.id}`);
  }
  for (const entry of document.entries) {
    invariant(Array.isArray(entry.source_effect_types), `ability effect types are invalid: ${entry.id}`);
  }
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
