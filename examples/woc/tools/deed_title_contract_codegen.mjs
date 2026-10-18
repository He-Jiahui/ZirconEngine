import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CATALOG_SOURCE_PATH = 'src/sim/content/deeds.ts';
const DEEDS_SOURCE_PATH = 'src/sim/deeds.ts';
const SERVER_SOURCE_PATH = 'server/game.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'deed_title_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'deed_title_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const catalogSource = sourceBlob(CATALOG_SOURCE_PATH);
  const deedsSource = sourceBlob(DEEDS_SOURCE_PATH);
  const serverSource = sourceBlob(SERVER_SOURCE_PATH);
  const deeds = extractDeeds(catalogSource);
  const document = {
    schema_version: 2,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/deed_title_contract_codegen.mjs',
    source_blobs: {
      [CATALOG_SOURCE_PATH]: sha256(catalogSource),
      [DEEDS_SOURCE_PATH]: sha256(deedsSource),
      [SERVER_SOURCE_PATH]: sha256(serverSource),
    },
    deeds,
    title_deed_count: deeds.filter((deed) => deed.title_reward).length,
    semantics: {
      order: 'DEED_ORDER is the append-only Object.keys(DEEDS) source order and persisted codes are one-based indexes into that order',
      select: 'a non-null deed id changes both persisted and entity title projections only when the deed is earned and its reward kind is title',
      clear: 'a null deed id clears both title projections unconditionally',
      invalid: 'unknown, unearned, rewardless and non-title deed ids are silent no-ops',
      migration: 'an absent or stale persisted title is normalized to no active title',
      item_discovery: 'first acquisition evaluates source triggers that directly read itemsDiscovered or its static quality projection',
    },
  };

  invariant(catalogSource.includes('Object.keys(DEEDS);'),
    'deed source order contract drifted');
  for (const needle of [
    "if (!meta.deedsEarned.has(deedId)) return;",
    "if (DEEDS[deedId]?.reward?.kind !== 'title') return;",
    'meta.activeTitle = deedId;',
    'e.title = deedId;',
  ]) {
    invariant(deedsSource.includes(needle), 'deed title semantics drifted: ' + needle);
  }
  invariant(serverSource.includes("case 'deed_set_title':"),
    'deed_set_title route is missing');
  invariant(serverSource.includes("msg.deedId === null || typeof msg.deedId === 'string'"),
    'deed_set_title server shape check drifted');
  invariant(deeds.length === 193, 'unexpected deed count: ' + deeds.length);
  invariant(document.title_deed_count === 19,
    'unexpected title deed count: ' + document.title_deed_count);
  invariant(deeds.every((deed) => /^[a-z0-9_]+$/u.test(deed.id)),
    'deed ids must remain ASCII protocol identifiers');

  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'deed title JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'deed title Zr contract');
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} deed title contract: ` +
      `${deeds.length} deeds, ${document.title_deed_count} titles\n`,
  );
}

function extractDeeds(sourceText) {
  const sourceFile = ts.createSourceFile(
    CATALOG_SOURCE_PATH,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const declaration = findVariable(sourceFile, 'DEEDS');
  const initializer = unwrapExpression(declaration.initializer);
  invariant(ts.isObjectLiteralExpression(initializer), 'DEEDS is no longer an object literal');
  return initializer.properties.map((property) => {
    invariant(ts.isPropertyAssignment(property), 'DEEDS contains a non-property entry');
    const id = propertyName(property.name);
    const definition = unwrapExpression(property.initializer);
    invariant(ts.isObjectLiteralExpression(definition), 'deed definition is not an object: ' + id);
    const declaredId = stringProperty(definition, 'id');
    invariant(declaredId === id, `deed key/id mismatch: ${id}/${declaredId}`);
    const rewardProperty = findProperty(definition, 'reward');
    let titleReward = false;
    if (rewardProperty) {
      invariant(ts.isPropertyAssignment(rewardProperty), 'deed reward is not a property: ' + id);
      const reward = unwrapExpression(rewardProperty.initializer);
      invariant(ts.isObjectLiteralExpression(reward), 'deed reward is not an object: ' + id);
      titleReward = stringProperty(reward, 'kind') === 'title';
    }
    return {
      id,
      title_reward: titleReward,
      item_discovery_trigger: itemDiscoveryTrigger(definition),
    };
  });
}

function itemDiscoveryTrigger(definition) {
  const triggerProperty = findProperty(definition, 'trigger');
  invariant(triggerProperty && ts.isPropertyAssignment(triggerProperty),
    'deed trigger is missing');
  const trigger = unwrapExpression(triggerProperty.initializer);
  invariant(ts.isObjectLiteralExpression(trigger), 'deed trigger is not an object');
  const kind = stringProperty(trigger, 'kind');
  if (kind === 'meter') {
    const meter = stringProperty(trigger, 'meter');
    if (meter === 'itemsDiscoveredCount') {
      return { kind: 'items_discovered_count', amount: numberProperty(trigger, 'amount') };
    }
    if (meter === 'poorItemsDiscoveredCount') {
      return { kind: 'poor_items_discovered_count', amount: numberProperty(trigger, 'amount') };
    }
  }
  if (kind === 'visit') {
    const markId = stringProperty(trigger, 'markId');
    if (markId.startsWith('quality:')) {
      return { kind: 'quality', values: [markId.slice('quality:'.length)] };
    }
  }
  if (kind === 'collectItems') {
    return { kind: 'collect_items', values: stringArrayProperty(trigger, 'itemIds') };
  }
  return null;
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

function findProperty(object, name) {
  return object.properties.find((property) =>
    (ts.isPropertyAssignment(property) || ts.isShorthandPropertyAssignment(property)) &&
      propertyName(property.name) === name,
  );
}

function stringProperty(object, name) {
  const property = findProperty(object, name);
  invariant(property && ts.isPropertyAssignment(property), 'missing string property: ' + name);
  const value = unwrapExpression(property.initializer);
  invariant(ts.isStringLiteralLike(value), 'property is not a string: ' + name);
  return value.text;
}

function numberProperty(object, name) {
  const property = findProperty(object, name);
  invariant(property && ts.isPropertyAssignment(property), 'missing number property: ' + name);
  const value = unwrapExpression(property.initializer);
  invariant(ts.isNumericLiteral(value), 'property is not a number: ' + name);
  return Number(value.text);
}

function stringArrayProperty(object, name) {
  const property = findProperty(object, name);
  invariant(property && ts.isPropertyAssignment(property), 'missing array property: ' + name);
  const value = unwrapExpression(property.initializer);
  invariant(ts.isArrayLiteralExpression(value), 'property is not an array: ' + name);
  return value.elements.map((element) => {
    const item = unwrapExpression(element);
    invariant(ts.isStringLiteralLike(item), 'array item is not a string: ' + name);
    return item.text;
  });
}

function propertyName(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
  throw new Error('unsupported computed deed property');
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
  lines.push('pub deedCount(): int { return ' + document.deeds.length + '; }');
  lines.push('pub titleDeedCount(): int { return ' + document.title_deed_count + '; }');
  lines.push('pub deedIdAt(index: int): string {');
  document.deeds.forEach((deed, index) =>
    lines.push('    if (index == ' + index + ') return "' + deed.id + '";'));
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub deedCodeForId(id: string): uint {');
  document.deeds.forEach((deed, index) =>
    lines.push('    if (id == "' + deed.id + '") return <uint>' + (index + 1) + ';'));
  lines.push('    return <uint>0;');
  lines.push('}');
  lines.push('pub deedIsTitle(index: int): bool {');
  document.deeds.forEach((deed, index) => {
    if (deed.title_reward) lines.push('    if (index == ' + index + ') return true;');
  });
  lines.push('    return false;');
  lines.push('}');
  lines.push('pub deedCodeIsTitle(code: uint): bool {');
  lines.push('    if (code == <uint>0 || code > <uint>deedCount()) return false;');
  lines.push('    return deedIsTitle(<int>code - 1);');
  lines.push('}');
  lines.push('pub itemDiscoveryTriggerKind(index: int): int {');
  const triggerKinds = {
    items_discovered_count: 1,
    poor_items_discovered_count: 2,
    quality: 3,
    collect_items: 4,
  };
  document.deeds.forEach((deed, index) => {
    if (deed.item_discovery_trigger) {
      lines.push('    if (index == ' + index + ') return ' +
        triggerKinds[deed.item_discovery_trigger.kind] + ';');
    }
  });
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub itemDiscoveryTriggerAmount(index: int): int {');
  document.deeds.forEach((deed, index) => {
    const amount = deed.item_discovery_trigger?.amount;
    if (amount !== undefined) {
      lines.push('    if (index == ' + index + ') return ' + amount + ';');
    }
  });
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub itemDiscoveryTriggerValueCount(index: int): int {');
  document.deeds.forEach((deed, index) => {
    const values = deed.item_discovery_trigger?.values;
    if (values) lines.push('    if (index == ' + index + ') return ' + values.length + ';');
  });
  lines.push('    return 0;');
  lines.push('}');
  lines.push('pub itemDiscoveryTriggerValue(index: int, valueIndex: int): string {');
  document.deeds.forEach((deed, index) => {
    const values = deed.item_discovery_trigger?.values;
    values?.forEach((value, valueIndex) => lines.push(
      '    if (index == ' + index + ' && valueIndex == ' + valueIndex +
        ') return "' + value + '";',
    ));
  });
  lines.push('    return "";');
  lines.push('}');
  renderIndexedUtf8Accessors(lines, 'deedId', document.deeds.map((deed) => deed.id));
  return lines.join('\n') + '\n';
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
