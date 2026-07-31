import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, extname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

const HISTORICAL_REFERENCE_COMMIT = '7c10f280eec380e9877e66ce16333089e171fe42';
const CURRENT_REFERENCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const REFERENCE_COMMIT = readOption('--commit') ?? CURRENT_REFERENCE_COMMIT;
const rebaseline = process.argv.includes('--rebaseline');
const historical = process.argv.includes('--historical');
const EXPECTED = Object.freeze({
  source_files: 3163,
  source_characters: 56451702,
  commands: 165,
  dispatch_only_commands: 9,
  world_members: 248,
  world_methods: 181,
  world_data_members: 67,
  world_facets: 28,
  test_cases: 14716,
  test_files: 1331,
  parity_scenarios: 54,
  glbs: 949,
});
const SOURCE_EXTENSIONS = new Set(['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs', '.svelte']);
const WORLD_FACET_OWNERSHIP = Object.freeze({
  IWorldEntityRoster: 'simulation',
  IWorldCombat: 'simulation',
  IWorldTargeting: 'simulation',
  IWorldInteraction: 'simulation',
  IWorldLoot: 'simulation',
  IWorldInventory: 'simulation',
  IWorldCosmetics: 'presentation',
  IWorldQuests: 'simulation',
  IWorldProgressionXp: 'simulation',
  IWorldTalents: 'simulation',
  IWorldPet: 'simulation',
  IWorldParty: 'simulation',
  IWorldTrade: 'simulation',
  IWorldChat: 'service',
  IWorldDuelArena: 'simulation',
  IWorldCardMinigame: 'simulation',
  IWorldSocialGraph: 'service',
  IWorldMarket: 'service',
  IWorldMail: 'service',
  IWorldDungeons: 'simulation',
  IWorldDelves: 'simulation',
  IWorldDailyRewards: 'service',
  IWorldTelemetry: 'service',
  IWorldProfessions: 'simulation',
  IWorldBank: 'service',
  IWorldValeCup: 'simulation',
  IWorldDungeonFinder: 'service',
  IWorldDeeds: 'simulation',
});
const UNFACETED_CLIENT_COMMAND_OWNERSHIP = Object.freeze({
  interact: 'simulation',
  loot: 'simulation',
  harvestCorpse: 'simulation',
  pickup: 'simulation',
  accept: 'simulation',
  turnin: 'simulation',
  abandon: 'simulation',
  qlinkaccept: 'simulation',
  equip: 'simulation',
  inv_move: 'simulation',
  unequip_item: 'simulation',
  use: 'simulation',
  discard: 'simulation',
  buy: 'simulation',
  sell: 'simulation',
  buyback: 'simulation',
  sell_all_junk: 'simulation',
  harvest_node: 'simulation',
  craft_item: 'simulation',
  challengeResponse: 'service',
  chat: 'service',
  emote: 'service',
  equip_bag: 'simulation',
  unequip_bag: 'simulation',
  autoloot: 'simulation',
  set_town_focus: 'simulation',
});

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const defaultSourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceRoot = resolve(readOption('--source') ?? defaultSourceRoot);
const outputRoot = resolve(readOption('--output') ?? join(projectRoot, 'reference', 'current-head'));
const checkOnly = process.argv.includes('--check');
let trackedFileSet = new Set();
const referenceTextCache = new Map();

main();

function main() {
  if (REFERENCE_COMMIT !== CURRENT_REFERENCE_COMMIT && !historical) {
    throw new Error('a non-current reference commit requires --historical');
  }
  git('rev-parse', '--verify', `${REFERENCE_COMMIT}^{commit}`);
  const trackedFiles = git('ls-tree', '-r', '-z', '--name-only', REFERENCE_COMMIT)
    .split('\0')
    .filter(Boolean)
    .map(toPosix)
    .sort();
  trackedFileSet = new Set(trackedFiles);
  primeReferenceText(trackedFiles.filter((path) => SOURCE_EXTENSIONS.has(extname(path))));
  const world = extractWorldApi();
  const commands = extractCommands(world.facets);
  const tests = extractTests(trackedFiles);
  const parity = extractParity(trackedFiles);
  const assets = extractAssets(trackedFiles);
  const uiFlows = extractUiFlows(trackedFiles);
  const sourceManifest = buildSourceManifest(trackedFiles, commands, world, tests, parity, assets, uiFlows);

  if (!historical) {
    assertExact('source_files', sourceManifest.audited_totals.source_files);
    assertExact('source_characters', sourceManifest.audited_totals.source_characters);
    assertExact('commands', commands.length);
    assertExact('dispatch_only_commands', commands.filter((entry) => entry.kind === 'dispatch_only').length);
    assertExact('world_members', world.entries.length);
    assertExact('world_methods', world.entries.filter((entry) => entry.kind === 'method').length);
    assertExact('world_data_members', world.entries.filter((entry) => entry.kind === 'data').length);
    assertExact('world_facets', world.facets.length);
    assertExact('test_cases', tests.entries.length);
    assertExact('test_files', tests.files.length);
    assertExact('parity_scenarios', parity.length);
    assertExact('glbs', assets.entries.length);
  }

  const catalogDocuments = new Map([
    ['command_catalog.json', catalog(commands)],
    ['world_api_catalog.json', { ...catalog(world.entries), facets: world.facets }],
    ['test_catalog.json', { ...catalog(tests.entries), files: tests.files, generators: tests.generators }],
    ['parity_scenarios.json', catalog(parity)],
    ['asset_catalog.json', { ...catalog(assets.entries), totals: assets.totals }],
    ['ui_flow_catalog.json', catalog(uiFlows)],
  ]);
  sourceManifest.catalog_sha256 = Object.fromEntries(
    [...catalogDocuments].map(([name, document]) => [name, hashBytes(Buffer.from(renderDocument(document), 'utf8'))]),
  );
  const documents = new Map([['source_manifest.json', sourceManifest], ...catalogDocuments]);

  if (!checkOnly) mkdirSync(outputRoot, { recursive: true });
  for (const [name, document] of documents) writeOrCheck(name, document);
  const mode = checkOnly ? 'checked' : historical ? 'historical-generated' : 'generated';
  process.stdout.write(`${mode} ${documents.size} WOC reference catalogs for ${REFERENCE_COMMIT} at ${outputRoot}\n`);
}

function extractCommands(worldFacets) {
  const relativePath = 'src/world_api.ts';
  const sourceFile = parseTypeScript(relativePath);
  const names = stringArrayVariable(sourceFile, 'COMMAND_NAMES');
  const dispatchOnly = new Set(stringArrayVariable(sourceFile, 'DISPATCH_ONLY_COMMANDS'));
  const commandFacets = stringObjectVariable(sourceFile, 'COMMAND_FACETS');
  const ownershipByFacet = new Map(worldFacets.map((facet) => [facet.name, facet.ownership_class]));
  assertUnfacetedClientCommandOwnership(names, dispatchOnly, commandFacets);
  return names.map((name, index) => {
    const facet = commandFacets.get(name) ?? null;
    const kind = dispatchOnly.has(name) ? 'dispatch_only' : 'client_send';
    const ownership_class = facet === null
      ? commandOwnershipWithoutFacet(name, kind)
      : ownershipByFacet.get(facet);
    invariant(ownership_class, `command ${name} has no reconstruction ownership class`);
    return {
      index,
      name,
      kind,
      facet,
      ownership_class,
      source_owner: `${relativePath}#COMMAND_NAMES`,
      woc_owner: 'scripts/woc_game/src/protocol/commands.zr',
    };
  });
}

function extractWorldApi() {
  const aggregatePath = 'src/world_api.ts';
  const aggregate = parseTypeScript(aggregatePath);
  const importOwners = new Map();
  for (const statement of aggregate.statements) {
    if (!ts.isImportDeclaration(statement) || !statement.importClause?.namedBindings ||
        !ts.isNamedImports(statement.importClause.namedBindings) || !ts.isStringLiteral(statement.moduleSpecifier)) continue;
    for (const element of statement.importClause.namedBindings.elements) {
      importOwners.set(element.name.text, resolveModulePath(aggregatePath, statement.moduleSpecifier.text));
    }
  }
  const aggregateInterface = aggregate.statements.find((statement) =>
    ts.isInterfaceDeclaration(statement) && statement.name.text === 'IWorld');
  invariant(aggregateInterface, 'IWorld aggregate interface is missing');
  const facetNames = aggregateInterface.heritageClauses
    ?.flatMap((clause) => clause.types.map((type) => type.expression.getText(aggregate))) ?? [];
  assertWorldFacetOwnership(facetNames);
  const entries = [];
  const facets = [];
  for (const facet of facetNames) {
    const owner = importOwners.get(facet);
    invariant(owner, `missing import owner for ${facet}`);
    const sourceFile = parseTypeScript(owner);
    const declaration = sourceFile.statements.find((statement) =>
      ts.isInterfaceDeclaration(statement) && statement.name.text === facet);
    invariant(declaration, `missing ${facet} in ${owner}`);
    const facetEntries = declaration.members.map((member) => {
      invariant(ts.isMethodSignature(member) || ts.isPropertySignature(member),
        `${facet} contains unsupported member syntax: ${member.getText(sourceFile)}`);
      const name = memberName(member.name, sourceFile);
      const location = sourceFile.getLineAndCharacterOfPosition(member.getStart(sourceFile));
      return {
        facet,
        name,
        kind: ts.isMethodSignature(member) ? 'method' : 'data',
        signature: normalizeWhitespace(member.getText(sourceFile)),
        ownership_class: WORLD_FACET_OWNERSHIP[facet],
        source_owner: `${owner}:${location.line + 1}`,
        woc_owner: `scripts/woc_game/src/world_api/${toSnakeCase(facet.replace(/^IWorld/, ''))}.zr`,
      };
    });
    facets.push({
      name: facet,
      source_owner: owner,
      woc_owner: facetEntries[0]?.woc_owner ?? `scripts/woc_game/src/world_api/${toSnakeCase(facet)}.zr`,
      member_count: facetEntries.length,
      ownership_class: WORLD_FACET_OWNERSHIP[facet],
    });
    entries.push(...facetEntries);
  }
  assertUnique(entries, (entry) => entry.name, 'IWorld member');
  return { entries, facets };
}

function assertWorldFacetOwnership(facetNames) {
  const declared = Object.keys(WORLD_FACET_OWNERSHIP).sort();
  const extracted = [...facetNames].sort();
  invariant(
    JSON.stringify(declared) === JSON.stringify(extracted),
    `IWorld facet ownership map drift: declared ${declared.join(', ')}, extracted ${extracted.join(', ')}`,
  );
}

function commandOwnershipWithoutFacet(name, kind) {
  if (kind === 'dispatch_only') return 'client';
  const ownership = UNFACETED_CLIENT_COMMAND_OWNERSHIP[name];
  invariant(ownership, `client command ${name} is missing its explicit reconstruction ownership class`);
  return ownership;
}

function assertUnfacetedClientCommandOwnership(names, dispatchOnly, commandFacets) {
  const extracted = names
    .filter((name) => !dispatchOnly.has(name) && !commandFacets.has(name))
    .sort();
  const declared = Object.keys(UNFACETED_CLIENT_COMMAND_OWNERSHIP).sort();
  invariant(
    JSON.stringify(declared) === JSON.stringify(extracted),
    `unfaceted client command ownership map drift: declared ${declared.join(', ')}, extracted ${extracted.join(', ')}`,
  );
}

function extractTests(trackedFiles) {
  const testFiles = trackedFiles.filter((path) => path.endsWith('.test.ts'));
  const files = testFiles.map((path) => ({
    path,
    ownership_class: testOwnershipClass(path),
    source_owner: path,
    woc_owner: mapTestOwner(path),
  }));
  const entries = [];
  const generators = [];
  for (const file of testFiles) {
    const sourceFile = parseTypeScript(file);
    const aliases = testAliases(sourceFile);
    let fileOrdinal = 0;
    visit(sourceFile, (node) => {
      if (!ts.isCallExpression(node) || !isTestRegistration(node, aliases)) return;
      const location = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
      const title = node.arguments.length > 0 ? expressionLabel(node.arguments[0], sourceFile) : '<missing-title>';
      const entry = {
        id: `${file}:${location.line + 1}:${location.character + 1}:${fileOrdinal}`,
        file,
        line: location.line + 1,
        column: location.character + 1,
        ordinal: fileOrdinal,
        title,
        mode: testMode(node.expression, aliases),
        registration_kind: registrationKind(node.expression),
        ownership_class: testOwnershipClass(file),
        source_owner: file,
        woc_owner: mapTestOwner(file),
      };
      if (entry.registration_kind.startsWith('parameterized:')) generators.push(entry);
      else entries.push(entry);
      fileOrdinal += 1;
    });
  }
  assertUnique(entries, (entry) => entry.id, 'test case');
  assertUnique(generators, (entry) => entry.id, 'parameterized test generator');
  return { entries, files, generators };
}

function extractParity(trackedFiles) {
  const scenariosPath = 'tests/parity/scenarios.ts';
  const sourceFile = parseTypeScript(scenariosPath);
  const initializer = variableInitializer(sourceFile, 'SCENARIOS');
  const array = unwrapExpression(initializer);
  invariant(ts.isArrayLiteralExpression(array), 'SCENARIOS must be an array literal');
  const functions = new Map(sourceFile.statements
    .filter(ts.isFunctionDeclaration)
    .filter((declaration) => declaration.name)
    .map((declaration) => [declaration.name.text, declaration]));
  const goldenFiles = trackedFiles.filter((path) => /^tests\/parity\/golden\/[^/]+\.json$/.test(path));
  const goldenByName = new Map(goldenFiles.map((path) => [path.slice(path.lastIndexOf('/') + 1, -5), path]));
  const entries = array.elements.map((element, index) => {
    invariant(ts.isCallExpression(element) && ts.isIdentifier(element.expression),
      `SCENARIOS[${index}] must call a named factory`);
    const factory = element.expression.text;
    const declaration = functions.get(factory);
    invariant(declaration, `missing scenario factory ${factory}`);
    const name = findReturnedName(declaration, element, sourceFile);
    const coverage = findScenarioCoverage(declaration, sourceFile);
    const golden = goldenByName.get(name);
    invariant(golden, `scenario ${name} has no golden file`);
    return {
      index,
      name,
      factory,
      source_owner: `${scenariosPath}#${factory}`,
      golden,
      golden_sha256: hashBytes(referenceBytes(golden)),
      coverage,
      ownership_class: 'simulation',
      woc_owner: `scripts/woc_game/tests/parity/${name}.zr`,
    };
  });
  assertUnique(entries, (entry) => entry.name, 'parity scenario');
  invariant(entries.length === goldenFiles.length,
    `scenario/golden mismatch: ${entries.length} scenarios, ${goldenFiles.length} goldens`);
  return entries;
}

function extractAssets(trackedFiles) {
  const paths = trackedFiles.filter((path) => path.startsWith('public/models/') && path.endsWith('.glb'));
  const entries = paths.map((path) => {
    const bytes = referenceBytes(path);
    const glb = parseGlb(bytes, path);
    return {
      path,
      sha256: hashBytes(bytes),
      byte_length: bytes.length,
      gltf_version: glb.asset?.version ?? null,
      extensions_used: [...(glb.extensionsUsed ?? [])].sort(),
      extensions_required: [...(glb.extensionsRequired ?? [])].sort(),
      animation_count: glb.animations?.length ?? 0,
      skin_count: glb.skins?.length ?? 0,
      ownership_class: 'presentation',
      source_owner: path,
      woc_owner: `assets/models/${path.slice('public/models/'.length)}`,
    };
  });
  const totals = {
    glbs: entries.length,
    bytes: entries.reduce((sum, entry) => sum + entry.byte_length, 0),
    animations: entries.reduce((sum, entry) => sum + entry.animation_count, 0),
    skins: entries.reduce((sum, entry) => sum + entry.skin_count, 0),
    meshopt: entries.filter((entry) => entry.extensions_used.includes('EXT_meshopt_compression')).length,
    webp: entries.filter((entry) => entry.extensions_used.includes('EXT_texture_webp')).length,
    quantization: entries.filter((entry) => entry.extensions_used.includes('KHR_mesh_quantization')).length,
  };
  return { entries, totals };
}

function extractUiFlows(trackedFiles) {
  const sourcePrefixes = [
    ['src/ui/', 'gameplay_ui', 'client'],
    ['src/admin/', 'admin', 'admin'],
    ['src/editor/', 'authoring', 'editor'],
    ['src/guide/', 'guide', 'guide'],
  ];
  const entries = [];
  for (const path of trackedFiles) {
    if (path.endsWith('.html')) {
      entries.push({
        id: `entrypoint:${path}`,
        kind: 'entrypoint',
        flow: entrypointFlow(path),
        path,
        ownership_class: 'client',
        source_owner: path,
        woc_owner: 'native/apps/woc_client',
      });
      continue;
    }
    const owner = sourcePrefixes.find(([prefix]) => path.startsWith(prefix));
    if (!owner || !['.ts', '.js', '.svelte'].includes(extname(path))) continue;
    entries.push({
      id: `${owner[1]}:${path}`,
      kind: 'source',
      flow: owner[1],
      path,
      ownership_class: 'client',
      source_owner: path,
      woc_owner: `native/apps/woc_${owner[2]}`,
    });
  }
  assertUnique(entries, (entry) => entry.id, 'UI flow source');
  invariant(entries.length > 0, 'UI flow catalog is empty');
  return entries;
}

function buildSourceManifest(trackedFiles, commands, world, tests, parity, assets, uiFlows) {
  const sourceFiles = trackedFiles.filter((path) => SOURCE_EXTENSIONS.has(extname(path)));
  const sourceCharacters = sourceFiles.reduce((sum, path) =>
    sum + referenceText(path).length, 0);
  const paritySources = trackedFiles.filter((path) => path.startsWith('tests/parity/') && path.endsWith('.ts'));
  const goldenFiles = trackedFiles.filter((path) => /^tests\/parity\/golden\/[^/]+\.json$/.test(path));
  return {
    schema_version: 1,
    source_commit: REFERENCE_COMMIT,
    source_repository: 'world-of-claudecraft',
    generated_by: 'examples/woc/tools/reference_inventory.mjs',
    source_extensions: [...SOURCE_EXTENSIONS],
    identities: {
      package_manifest: fileIdentity('package.json'),
      parity_sources: treeIdentity(paritySources),
      golden_directory: treeIdentity(goldenFiles),
    },
    audited_totals: {
      source_files: sourceFiles.length,
      source_characters: sourceCharacters,
      commands: commands.length,
      dispatch_only_commands: commands.filter((entry) => entry.kind === 'dispatch_only').length,
      world_members: world.entries.length,
      world_methods: world.entries.filter((entry) => entry.kind === 'method').length,
      world_data_members: world.entries.filter((entry) => entry.kind === 'data').length,
      world_facets: world.facets.length,
      test_cases: tests.entries.length,
      test_files: tests.files.length,
      test_case_generators: tests.generators.length,
      parity_scenarios: parity.length,
      glbs: assets.entries.length,
      glb_animations: assets.totals.animations,
      glb_skins: assets.totals.skins,
      ui_flow_sources: uiFlows.length,
    },
  };
}

function parseGlb(bytes, path) {
  invariant(bytes.length >= 20 && bytes.toString('ascii', 0, 4) === 'glTF', `${path} is not a GLB`);
  invariant(bytes.readUInt32LE(4) === 2, `${path} is not GLB version 2`);
  invariant(bytes.readUInt32LE(8) === bytes.length, `${path} GLB length header is invalid`);
  let offset = 12;
  while (offset + 8 <= bytes.length) {
    const length = bytes.readUInt32LE(offset);
    const type = bytes.readUInt32LE(offset + 4);
    const start = offset + 8;
    const end = start + length;
    invariant(end <= bytes.length, `${path} contains a truncated GLB chunk`);
    if (type === 0x4e4f534a) return JSON.parse(bytes.toString('utf8', start, end).replace(/\0+$/u, '').trimEnd());
    offset = end;
  }
  throw new Error(`${path} has no JSON chunk`);
}

function testAliases(sourceFile) {
  const aliases = new Set(['test', 'it']);
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier) ||
        statement.moduleSpecifier.text !== 'vitest' || !statement.importClause?.namedBindings ||
        !ts.isNamedImports(statement.importClause.namedBindings)) continue;
    for (const element of statement.importClause.namedBindings.elements) {
      const imported = element.propertyName?.text ?? element.name.text;
      if (imported === 'test' || imported === 'it') aliases.add(element.name.text);
    }
  }
  return aliases;
}

function isTestRegistration(node, aliases) {
  const expression = node.expression;
  if (ts.isIdentifier(expression)) return aliases.has(expression.text);
  if (ts.isPropertyAccessExpression(expression)) {
    const root = leftmostIdentifier(expression);
    if (!root || !aliases.has(root.text)) return false;
    return !new Set(['each', 'for', 'runIf', 'skipIf']).has(expression.name.text);
  }
  if (ts.isCallExpression(expression)) {
    const root = leftmostIdentifier(expression.expression);
    return Boolean(root && aliases.has(root.text));
  }
  return false;
}

function testMode(expression, aliases) {
  const text = expression.getText();
  for (const mode of ['skip', 'todo', 'only', 'fails', 'concurrent']) {
    if (new RegExp(`\\.${mode}(?:\\.|\\(|$)`).test(text)) return mode;
  }
  const root = leftmostIdentifier(expression);
  return root && aliases.has(root.text) ? 'normal' : 'unknown';
}

function registrationKind(expression) {
  if (ts.isIdentifier(expression)) return 'direct';
  if (ts.isPropertyAccessExpression(expression)) return `modifier:${expression.name.text}`;
  if (ts.isCallExpression(expression)) {
    const callee = expression.expression;
    return ts.isPropertyAccessExpression(callee) ? `parameterized:${callee.name.text}` : 'parameterized:call';
  }
  return 'unknown';
}

function leftmostIdentifier(expression) {
  let current = expression;
  while (ts.isPropertyAccessExpression(current) || ts.isElementAccessExpression(current)) current = current.expression;
  while (ts.isCallExpression(current)) current = current.expression;
  while (ts.isPropertyAccessExpression(current) || ts.isElementAccessExpression(current)) current = current.expression;
  return ts.isIdentifier(current) ? current : null;
}

function findReturnedName(declaration, call, sourceFile) {
  const argumentsByParameter = new Map();
  declaration.parameters.forEach((parameter, index) => {
    if (ts.isIdentifier(parameter.name) && call.arguments[index]) {
      argumentsByParameter.set(parameter.name.text, literalValue(call.arguments[index]));
    }
  });
  let name = null;
  visit(declaration.body, (node) => {
    if (name || !ts.isPropertyAssignment(node) || memberName(node.name, sourceFile) !== 'name') return;
    name = evaluateString(node.initializer, argumentsByParameter);
  });
  invariant(name, `scenario factory ${declaration.name?.text ?? '<anonymous>'} has no literal name`);
  return name;
}

function findScenarioCoverage(declaration, sourceFile) {
  let coverage = null;
  visit(declaration.body, (node) => {
    if (coverage || !ts.isPropertyAssignment(node) || memberName(node.name, sourceFile) !== 'coverage') return;
    const initializer = unwrapExpression(node.initializer);
    if (!ts.isArrayLiteralExpression(initializer)) return;
    coverage = initializer.elements.map((element) => {
      invariant(ts.isStringLiteralLike(element), `${declaration.name?.text ?? '<anonymous>'} coverage must contain strings`);
      return element.text;
    });
  });
  invariant(coverage, `scenario factory ${declaration.name?.text ?? '<anonymous>'} has no coverage array`);
  return coverage;
}

function evaluateString(expression, environment) {
  if (ts.isStringLiteralLike(expression)) return expression.text;
  if (ts.isConditionalExpression(expression)) {
    const condition = evaluateBoolean(expression.condition, environment);
    if (condition === true) return evaluateString(expression.whenTrue, environment);
    if (condition === false) return evaluateString(expression.whenFalse, environment);
  }
  return null;
}

function evaluateBoolean(expression, environment) {
  if (expression.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (expression.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (ts.isIdentifier(expression)) return environment.get(expression.text) ?? null;
  if (ts.isPrefixUnaryExpression(expression) && expression.operator === ts.SyntaxKind.ExclamationToken) {
    const value = evaluateBoolean(expression.operand, environment);
    return typeof value === 'boolean' ? !value : null;
  }
  return null;
}

function literalValue(expression) {
  if (expression.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (expression.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (ts.isStringLiteralLike(expression) || ts.isNumericLiteral(expression)) return expression.text;
  return null;
}

function visit(node, callback) {
  if (!node) return;
  callback(node);
  ts.forEachChild(node, (child) => visit(child, callback));
}

function parseTypeScript(relativePath) {
  const source = referenceText(relativePath);
  return ts.createSourceFile(relativePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
}

function variableInitializer(sourceFile, variableName) {
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === variableName && declaration.initializer) {
        return declaration.initializer;
      }
    }
  }
  throw new Error(`missing ${variableName} in ${sourceFile.fileName}`);
}

function stringArrayVariable(sourceFile, variableName) {
  const initializer = unwrapExpression(variableInitializer(sourceFile, variableName));
  invariant(ts.isArrayLiteralExpression(initializer), `${variableName} must be an array literal`);
  return initializer.elements.map((element) => {
    invariant(ts.isStringLiteralLike(element), `${variableName} contains a non-string element`);
    return element.text;
  });
}

function stringObjectVariable(sourceFile, variableName) {
  const initializer = unwrapExpression(variableInitializer(sourceFile, variableName));
  invariant(ts.isObjectLiteralExpression(initializer), `${variableName} must be an object literal`);
  return new Map(initializer.properties.filter(ts.isPropertyAssignment).map((property) => {
    invariant(ts.isStringLiteralLike(property.initializer), `${variableName}.${memberName(property.name, sourceFile)} is not a string`);
    return [memberName(property.name, sourceFile), property.initializer.text];
  }));
}

function unwrapExpression(expression) {
  let current = expression;
  while (ts.isAsExpression(current) || ts.isSatisfiesExpression(current) ||
         ts.isParenthesizedExpression(current) || ts.isTypeAssertionExpression(current)) current = current.expression;
  return current;
}

function memberName(name, sourceFile) {
  if (ts.isIdentifier(name) || ts.isPrivateIdentifier(name) || ts.isStringLiteralLike(name) || ts.isNumericLiteral(name)) return name.text;
  return name.getText(sourceFile);
}

function expressionLabel(expression, sourceFile) {
  if (ts.isStringLiteralLike(expression)) return expression.text;
  if (ts.isNoSubstitutionTemplateLiteral(expression)) return expression.text;
  return normalizeWhitespace(expression.getText(sourceFile));
}

function resolveModulePath(ownerPath, moduleSpecifier) {
  invariant(moduleSpecifier.startsWith('.'), `non-relative facet import ${moduleSpecifier}`);
  const candidate = toPosix(resolve(dirname(join(sourceRoot, fromPosix(ownerPath))), moduleSpecifier));
  const relativeCandidate = toPosix(relative(sourceRoot, candidate));
  for (const suffix of ['.ts', '/index.ts']) {
    if (trackedFileSet.has(relativeCandidate + suffix)) return relativeCandidate + suffix;
  }
  throw new Error(`cannot resolve ${moduleSpecifier} from ${ownerPath}`);
}

function mapTestOwner(path) {
  if (path.startsWith('tests/parity/')) return 'native/crates/woc_parity/tests';
  if (path.includes('/server') || path.startsWith('server/')) return 'native/apps/woc_server/tests';
  if (path.includes('/ui') || path.includes('/render')) return 'native/apps/woc_client/tests';
  return 'scripts/woc_game/tests';
}

function testOwnershipClass(path) {
  const owner = mapTestOwner(path);
  if (owner === 'native/apps/woc_server/tests') return 'service';
  if (owner === 'native/apps/woc_client/tests') return 'client';
  return 'simulation';
}

function entrypointFlow(path) {
  const name = path.toLowerCase();
  if (name.includes('admin')) return 'admin';
  if (name.includes('guide') || name.includes('wiki')) return 'guide';
  if (name.includes('editor')) return 'authoring';
  return 'gameplay_ui';
}

function fileIdentity(path) {
  const bytes = referenceBytes(path);
  return { path, bytes: bytes.length, sha256: hashBytes(bytes) };
}

function treeIdentity(paths) {
  const identities = paths.map((path) => fileIdentity(path));
  const digest = createHash('sha256');
  for (const identity of identities) digest.update(`${identity.path}\0${identity.sha256}\n`, 'utf8');
  return { file_count: identities.length, sha256: digest.digest('hex'), files: identities };
}

function catalog(entries) {
  return { schema_version: 1, source_commit: REFERENCE_COMMIT, entries };
}

function writeOrCheck(name, document) {
  const output = renderDocument(document);
  const path = join(outputRoot, name);
  if (checkOnly) {
    invariant(existsSync(path), `${name} is missing; run npm run generate`);
    invariant(readFileSync(path, 'utf8') === output, `${name} is stale; run npm run generate`);
  } else {
    writeFileSync(path, output, 'utf8');
  }
}

function renderDocument(document) {
  return `${JSON.stringify(document, null, 2)}\n`;
}

function git(...args) {
  return execFileSync('git', ['-C', sourceRoot, ...args], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
}

function referenceText(path) {
  const cached = referenceTextCache.get(path);
  if (cached !== undefined) return cached;
  const text = referenceBytes(path).toString('utf8');
  referenceTextCache.set(path, text);
  return text;
}

function primeReferenceText(paths) {
  const pending = paths.filter((path) => !referenceTextCache.has(path));
  if (pending.length === 0) return;
  const input = `${pending.map((path) => `${REFERENCE_COMMIT}:${path}`).join('\n')}\n`;
  const output = execFileSync('git', ['-C', sourceRoot, 'cat-file', '--batch'], {
    input,
    maxBuffer: 128 * 1024 * 1024,
  });
  let offset = 0;
  for (const path of pending) {
    const headerEnd = output.indexOf(0x0a, offset);
    invariant(headerEnd >= 0, `missing Git blob header for ${path}`);
    const header = output.subarray(offset, headerEnd).toString('utf8');
    const match = /^[0-9a-f]+ blob (\d+)$/.exec(header);
    invariant(match, `invalid Git blob header for ${path}: ${header}`);
    const length = Number(match[1]);
    const start = headerEnd + 1;
    const end = start + length;
    invariant(end < output.length && output[end] === 0x0a, `truncated Git blob for ${path}`);
    referenceTextCache.set(path, output.subarray(start, end).toString('utf8'));
    offset = end + 1;
  }
  invariant(offset === output.length, 'Git batch output contains trailing bytes');
}

function referenceBytes(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${REFERENCE_COMMIT}:${path}`], {
    maxBuffer: 64 * 1024 * 1024,
  });
}

function hashBytes(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function assertUnique(entries, key, label) {
  const seen = new Set();
  for (const entry of entries) {
    const value = key(entry);
    invariant(!seen.has(value), `duplicate ${label}: ${value}`);
    seen.add(value);
  }
}

function groupCounts(entries, key) {
  const counts = {};
  for (const entry of entries) {
    const value = key(entry);
    counts[value] = (counts[value] ?? 0) + 1;
  }
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) => left.localeCompare(right)));
}

function assertExact(field, actual) {
  invariant(actual === EXPECTED[field], `${field}: extracted ${actual}, expected ${EXPECTED[field]}`);
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function normalizeWhitespace(value) {
  return value.replace(/\s+/gu, ' ').trim();
}

function toSnakeCase(value) {
  return value.replace(/([a-z0-9])([A-Z])/g, '$1_$2').replace(/[-\s]+/g, '_').toLowerCase();
}

function toPosix(value) {
  return value.split(sep).join('/');
}

function fromPosix(value) {
  return value.split('/').join(sep);
}

function readOption(name) {
  const index = process.argv.indexOf(name);
  if (index < 0) return null;
  invariant(process.argv[index + 1], `${name} requires a value`);
  return process.argv[index + 1];
}
