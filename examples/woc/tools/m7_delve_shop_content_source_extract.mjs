import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const root = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;

if (!root || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const path = 'src/sim/content/delves/shop.ts';
const text = execFileSync('git', ['-C', root, 'show', `${commit}:${path}`], {
  encoding: 'utf8',
});
const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);

const initializerFor = (name) => {
  for (const statement of source.statements) {
    if (!ts.isVariableStatement(statement)) {
      continue;
    }
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name && declaration.initializer) {
        return declaration.initializer;
      }
    }
  }
  throw new Error(`missing declaration ${name}`);
};

const stringValue = (node, label) => {
  if (!ts.isStringLiteral(node)) {
    throw new Error(`${label} must be a string literal`);
  }
  return node.text;
};

const numberValue = (node, label) => {
  if (!ts.isNumericLiteral(node)) {
    throw new Error(`${label} must be a numeric literal`);
  }
  return Number(node.text);
};

const property = (object, name) => {
  const entry = object.properties.find(
    (candidate) =>
      ts.isPropertyAssignment(candidate) &&
      ((ts.isIdentifier(candidate.name) && candidate.name.text === name) ||
        (ts.isStringLiteral(candidate.name) && candidate.name.text === name)),
  );
  if (!entry || !ts.isPropertyAssignment(entry)) {
    throw new Error(`missing shop property ${name}`);
  }
  return entry.initializer;
};

const readOffers = (name) => {
  const array = initializerFor(name);
  if (!ts.isArrayLiteralExpression(array)) {
    throw new Error(`${name} must be an array`);
  }
  return array.elements.map((element) => {
    if (!ts.isObjectLiteralExpression(element)) {
      throw new Error(`${name} contains a non-object offer`);
    }
    return {
      item_id: stringValue(property(element, 'itemId'), `${name}.itemId`),
      marks: numberValue(property(element, 'marks'), `${name}.marks`),
      gate: stringValue(property(element, 'gate'), `${name}.gate`),
    };
  });
};

const table = initializerFor('DELVE_SHOPS');
if (!ts.isObjectLiteralExpression(table)) {
  throw new Error('DELVE_SHOPS must be an object literal');
}
const shops = table.properties.map((entry) => {
  if (!ts.isPropertyAssignment(entry) || !ts.isIdentifier(entry.name) || !ts.isIdentifier(entry.initializer)) {
    throw new Error('DELVE_SHOPS must map identifiers to named source arrays');
  }
  return { id: entry.name.text, offers: readOffers(entry.initializer.text) };
});

const functionText = (name) => {
  const declaration = source.statements.find(
    (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === name,
  );
  if (!declaration) {
    throw new Error(`missing ${name}`);
  }
  return declaration.getText(source);
};

for (const marker of [
  "if (gate === 'available') return true;",
  "if (gate === 'heroicClear') return (clears[`${delveId}:heroic`] ?? 0) > 0;",
  "key.startsWith(`${delveId}:`)",
]) {
  if (!functionText('delveShopGateUnlocked').includes(marker)) {
    throw new Error(`shop gate source drifted: ${marker}`);
  }
}
for (const marker of [
  'return (DELVE_SHOPS[delveId] ?? []).map',
  "requiresHeroicClear: e.gate === 'heroicClear'",
  "requiresClears: e.gate.startsWith('clears:')",
]) {
  if (!functionText('resolveDelveShopOffers').includes(marker)) {
    throw new Error(`shop offer source drifted: ${marker}`);
  }
}

if (shops.length !== 2 || shops[0].id !== 'collapsed_reliquary' || shops[1].id !== 'drowned_litany') {
  throw new Error('Delve shop roster drifted');
}
if (shops.some((shop) => shop.offers.length !== 9)) {
  throw new Error('Delve shop offer count drifted');
}

process.stdout.write(
  JSON.stringify({
    shops,
    gate_kinds: { available: 1, clears: 2, heroicClear: 3 },
  }),
);
