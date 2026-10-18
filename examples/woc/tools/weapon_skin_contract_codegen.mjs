import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CATALOG_SOURCE_PATH = 'src/sim/content/weapon_skins.ts';
const RULES_SOURCE_PATH = 'src/sim/content/weapon_skin_rules.ts';
const SIM_SOURCE_PATH = 'src/sim/sim.ts';
const SERVER_SOURCE_PATH = 'server/game.ts';
const EXPECTED_TYPES = [
  'sword',
  'axe',
  'mace',
  'dagger',
  'staff',
  'wand',
  'bow',
  'crossbow',
];
const EXPECTED_TYPE_COUNTS = new Map([
  ['sword', 4],
  ['axe', 4],
  ['mace', 4],
  ['dagger', 4],
  ['staff', 4],
  ['wand', 4],
  ['bow', 3],
  ['crossbow', 2],
]);
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'weapon_skin_contract.json');
const zrOutput = join(
  projectRoot,
  'scripts',
  'woc_game',
  'src',
  'generated',
  'weapon_skin_contract.zr',
);
const rustOutput = join(
  projectRoot,
  'native',
  'crates',
  'woc_protocol',
  'src',
  'weapon_skin_contract.rs',
);
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const catalogSource = sourceBlob(CATALOG_SOURCE_PATH);
  const rulesSource = sourceBlob(RULES_SOURCE_PATH);
  const simSource = sourceBlob(SIM_SOURCE_PATH);
  const serverSource = sourceBlob(SERVER_SOURCE_PATH);
  const catalog = extractCatalog(catalogSource);
  const itemWeaponTypes = extractItemWeaponTypes(rulesSource);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/weapon_skin_contract_codegen.mjs',
    source_blobs: {
      [CATALOG_SOURCE_PATH]: sha256(catalogSource),
      [RULES_SOURCE_PATH]: sha256(rulesSource),
      [SIM_SOURCE_PATH]: sha256(simSource),
      [SERVER_SOURCE_PATH]: sha256(serverSource),
    },
    weapon_skin_types: catalog.types,
    weapon_skins: catalog.skins,
    item_weapon_types: itemWeaponTypes,
    semantics: {
      ownership: 'applying requires the account weaponSkinIds set to contain the known skin id',
      apply: 'the generated skin weapon type must match the requesting player current displayed mainhand type',
      detach: 'null skin plus one canonical weapon type removes only an existing entry of that type',
      hunter: 'hunter resolves crossbow before bow and applying either removes the other loadout entry',
      account_scope: 'a successful change replaces the account loadout and re-resolves every live character on that account',
      rng: 'weapon skin changes consume no simulation RNG',
    },
  };

  invariant(JSON.stringify(catalog.types) === JSON.stringify(EXPECTED_TYPES),
    'weapon skin type order drifted');
  invariant(catalog.skins.length === 29,
    'unexpected weapon skin count: ' + catalog.skins.length);
  invariant(new Set(catalog.skins.map((skin) => skin.id)).size === catalog.skins.length,
    'weapon skin ids must be unique');
  invariant(catalog.skins.every((skin) => /^[a-z0-9_]+$/u.test(skin.id)),
    'weapon skin ids must remain ASCII protocol identifiers');
  for (const type of EXPECTED_TYPES) {
    invariant(catalog.skins.filter((skin) => skin.weapon_type === type).length ===
      EXPECTED_TYPE_COUNTS.get(type), 'weapon skin type count drifted: ' + type);
  }
  invariant(itemWeaponTypes.length >= 90,
    'weapon item type table unexpectedly shrank: ' + itemWeaponTypes.length);
  invariant(itemWeaponTypes.some((entry) => entry.weapon_type === 'polearm'),
    'weapon item table must retain the explicit non-skinnable polearm type');
  for (const needle of [
    "if (cls === 'hunter') return ['crossbow', 'bow'];",
    "if (def.weaponType === 'bow') delete next.crossbow;",
    "else if (def.weaponType === 'crossbow') delete next.bow;",
    'if (!weaponSkinTypeMatches(cls, e.mainhandItemId, def.weaponType)) return false;',
  ]) {
    invariant((needle.includes('weaponSkinTypeMatches') ? simSource : rulesSource).includes(needle),
      'weapon skin semantics drifted: ' + needle);
  }
  for (const needle of [
    'if (!current.weaponSkinIds.includes(skinId)) return;',
    'if (!current.weaponSkinLoadout[wtype]) return;',
    'this.updateLiveAccountCosmetics(session.accountId, { ...current, weaponSkinLoadout });',
  ]) {
    invariant(serverSource.includes(needle), 'weapon skin server semantics drifted: ' + needle);
  }

  writeOrCheck(
    jsonOutput,
    JSON.stringify(document, null, 2) + '\n',
    'weapon skin JSON contract',
  );
  writeOrCheck(zrOutput, renderZr(document), 'weapon skin Zr contract');
  writeOrCheck(rustOutput, renderRust(document), 'weapon skin Rust contract');
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} weapon skin contract: ` +
      `${catalog.skins.length} skins, ${catalog.types.length} types, ` +
      `${itemWeaponTypes.length} item mappings\n`,
  );
}

function extractCatalog(sourceText) {
  const sourceFile = parseSource(CATALOG_SOURCE_PATH, sourceText);
  const collectionAliases = objectStringMap(findVariable(sourceFile, 'S1').initializer);
  const types = arrayStrings(findVariable(sourceFile, 'WEAPON_SKIN_TYPES').initializer);
  const skinsObject = asObject(findVariable(sourceFile, 'WEAPON_SKINS').initializer, 'WEAPON_SKINS');
  const skins = skinsObject.properties.map((property) => {
    invariant(ts.isPropertyAssignment(property), 'WEAPON_SKINS contains a non-property row');
    const key = propertyName(property.name);
    const definition = asObject(property.initializer, 'weapon skin ' + key);
    const id = stringProperty(definition, 'id');
    invariant(id === key, `weapon skin key/id mismatch: ${key}/${id}`);
    const collectionNode = propertyExpression(definition, 'collection');
    invariant(ts.isPropertyAccessExpression(collectionNode) &&
      ts.isIdentifier(collectionNode.expression) && collectionNode.expression.text === 'S1',
    'weapon skin collection must use an S1 alias: ' + id);
    const collection = collectionAliases[collectionNode.name.text];
    invariant(typeof collection === 'string', 'unknown weapon skin collection alias: ' + id);
    return {
      id,
      collection,
      rarity: stringProperty(definition, 'rarity'),
      weapon_type: stringProperty(definition, 'weaponType'),
      model: stringProperty(definition, 'model'),
      season: numberProperty(definition, 'season'),
      badge: optionalStringProperty(definition, 'badge'),
      handling: optionalStringProperty(definition, 'handling'),
    };
  });
  return { types, skins };
}

function extractItemWeaponTypes(sourceText) {
  const sourceFile = parseSource(RULES_SOURCE_PATH, sourceText);
  const object = asObject(
    findVariable(sourceFile, 'WEAPON_TYPE_BY_ITEM').initializer,
    'WEAPON_TYPE_BY_ITEM',
  );
  return object.properties.map((property) => {
    invariant(ts.isPropertyAssignment(property),
      'WEAPON_TYPE_BY_ITEM contains a non-property row');
    const itemId = propertyName(property.name);
    const value = unwrapExpression(property.initializer);
    invariant(ts.isStringLiteralLike(value), 'weapon item type is not a string: ' + itemId);
    invariant(EXPECTED_TYPES.includes(value.text) || value.text === 'polearm',
      'unknown item weapon type: ' + value.text);
    return { item_id: itemId, weapon_type: value.text };
  });
}

function parseSource(path, sourceText) {
  return ts.createSourceFile(path, sourceText, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
}

function findVariable(sourceFile, name) {
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name) {
        invariant(declaration.initializer, 'variable has no initializer: ' + name);
        return declaration;
      }
    }
  }
  throw new Error('missing variable: ' + name);
}

function asObject(expression, label) {
  const value = unwrapExpression(expression);
  invariant(ts.isObjectLiteralExpression(value), label + ' is not an object literal');
  return value;
}

function objectStringMap(expression) {
  const object = asObject(expression, 'string map');
  const out = {};
  for (const property of object.properties) {
    invariant(ts.isPropertyAssignment(property), 'string map contains a non-property row');
    const value = unwrapExpression(property.initializer);
    invariant(ts.isStringLiteralLike(value), 'string map value is not a string');
    out[propertyName(property.name)] = value.text;
  }
  return out;
}

function arrayStrings(expression) {
  const value = unwrapExpression(expression);
  invariant(ts.isArrayLiteralExpression(value), 'expected a string array literal');
  return value.elements.map((element) => {
    const item = unwrapExpression(element);
    invariant(ts.isStringLiteralLike(item), 'array item is not a string');
    return item.text;
  });
}

function findProperty(object, name) {
  return object.properties.find((property) =>
    ts.isPropertyAssignment(property) && propertyName(property.name) === name);
}

function propertyExpression(object, name) {
  const property = findProperty(object, name);
  invariant(property, 'missing property: ' + name);
  return unwrapExpression(property.initializer);
}

function stringProperty(object, name) {
  const value = propertyExpression(object, name);
  invariant(ts.isStringLiteralLike(value), 'property is not a string: ' + name);
  return value.text;
}

function optionalStringProperty(object, name) {
  const property = findProperty(object, name);
  if (!property) return null;
  const value = unwrapExpression(property.initializer);
  invariant(ts.isStringLiteralLike(value), 'property is not a string: ' + name);
  return value.text;
}

function numberProperty(object, name) {
  const value = propertyExpression(object, name);
  invariant(ts.isNumericLiteral(value), 'property is not a number: ' + name);
  return Number(value.text);
}

function propertyName(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
  throw new Error('unsupported computed property');
}

function unwrapExpression(expression) {
  let current = expression;
  while (ts.isAsExpression(current) || ts.isSatisfiesExpression(current) ||
      ts.isParenthesizedExpression(current)) {
    current = current.expression;
  }
  return current;
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.'];
  lines.push('pub weaponSkinTypeCount(): int { return ' + document.weapon_skin_types.length + '; }');
  lines.push('pub weaponSkinCount(): int { return ' + document.weapon_skins.length + '; }');
  renderStringAccessor(lines, 'weaponSkinTypeId', document.weapon_skin_types);
  lines.push('pub weaponSkinTypeCodeForId(id: string): uint {');
  document.weapon_skin_types.forEach((type, index) =>
    lines.push('    if (id == "' + type + '") return <uint>' + (index + 1) + ';'));
  lines.push('    return <uint>0;');
  lines.push('}');
  renderStringAccessor(lines, 'weaponSkinId', document.weapon_skins.map((skin) => skin.id));
  renderStringAccessor(lines, 'weaponSkinCollection',
    document.weapon_skins.map((skin) => skin.collection));
  renderStringAccessor(lines, 'weaponSkinRarity',
    document.weapon_skins.map((skin) => skin.rarity));
  renderStringAccessor(lines, 'weaponSkinModel',
    document.weapon_skins.map((skin) => skin.model));
  renderStringAccessor(lines, 'weaponSkinBadge',
    document.weapon_skins.map((skin) => skin.badge ?? ''));
  renderStringAccessor(lines, 'weaponSkinHandling',
    document.weapon_skins.map((skin) => skin.handling ?? ''));
  lines.push('pub weaponSkinTypeCode(index: int): uint {');
  document.weapon_skins.forEach((skin, index) => lines.push(
    '    if (index == ' + index + ') return <uint>' +
      (document.weapon_skin_types.indexOf(skin.weapon_type) + 1) + ';',
  ));
  lines.push('    return <uint>0;');
  lines.push('}');
  lines.push('pub weaponSkinSeason(index: int): int {');
  document.weapon_skins.forEach((skin, index) =>
    lines.push('    if (index == ' + index + ') return ' + skin.season + ';'));
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub weaponSkinCodeForId(id: string): uint {');
  document.weapon_skins.forEach((skin, index) =>
    lines.push('    if (id == "' + skin.id + '") return <uint>' + (index + 1) + ';'));
  lines.push('    return <uint>0;');
  lines.push('}');
  lines.push('pub itemWeaponTypeCodeForId(id: string): uint {');
  document.item_weapon_types.forEach((entry) => lines.push(
    '    if (id == "' + entry.item_id + '") return <uint>' +
      (entry.weapon_type === 'polearm' ? 9 :
        document.weapon_skin_types.indexOf(entry.weapon_type) + 1) + ';',
  ));
  lines.push('    return <uint>0;');
  lines.push('}');
  renderIndexedUtf8Accessors(
    lines,
    'weaponSkinId',
    document.weapon_skins.map((skin) => skin.id),
  );
  return lines.join('\n') + '\n';
}

function renderRust(document) {
  const typeCodes = document.weapon_skins.map((skin) =>
    document.weapon_skin_types.indexOf(skin.weapon_type) + 1,
  );
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub const WEAPON_SKIN_COUNT: usize = ' + document.weapon_skins.length + ';',
    'pub const WEAPON_SKIN_TYPE_COUNT: usize = ' + document.weapon_skin_types.length + ';',
    'pub const WEAPON_SKIN_TYPE_BY_CODE: [u8; WEAPON_SKIN_COUNT] = [',
  ];
  lines.push('    ' + typeCodes.join(', ') + ',');
  lines.push('];');
  lines.push('');
  lines.push('pub fn weapon_skin_code_matches_loadout_type(skin_code: u8, loadout_type_index: usize) -> bool {');
  lines.push('    skin_code != 0');
  lines.push('        && usize::from(skin_code) <= WEAPON_SKIN_COUNT');
  lines.push('        && loadout_type_index < WEAPON_SKIN_TYPE_COUNT');
  lines.push('        && WEAPON_SKIN_TYPE_BY_CODE[usize::from(skin_code) - 1]');
  lines.push('            == u8::try_from(loadout_type_index + 1).expect("weapon skin type count fits u8")');
  lines.push('}');
  return lines.join('\n') + '\n';
}

function renderStringAccessor(lines, name, values) {
  lines.push('pub ' + name + '(index: int): string {');
  values.forEach((value, index) =>
    lines.push('    if (index == ' + index + ') return "' + value + '";'));
  lines.push('    return "";');
  lines.push('}');
}

function renderIndexedUtf8Accessors(lines, name, values) {
  const encoded = values.map((value) => [...Buffer.from(value, 'utf8')]);
  lines.push('pub ' + name + 'Utf8Length(index: int): int {');
  encoded.forEach((bytes, index) =>
    lines.push('    if (index == ' + index + ') return ' + bytes.length + ';'));
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub ' + name + 'Utf8Byte(index: int, byteIndex: int): uint {');
  encoded.forEach((bytes, index) => {
    lines.push('    if (index == ' + index + ') {');
    bytes.forEach((byte, byteIndex) => lines.push(
      '        if (byteIndex == ' + byteIndex + ') return <uint>' + byte + ';',
    ));
    lines.push('        return <uint>0;');
    lines.push('    }');
  });
  lines.push('    return <uint>0;');
  lines.push('}');
}

function sourceBlob(path) {
  return execFileSync(
    'git',
    ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path],
    { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), label + ' is missing; run its generate script');
    invariant(readFileSync(path, 'utf8') === output,
      label + ' is stale; run its generate script');
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
