import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const root = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;
if (!root || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const readSource = (path) =>
  execFileSync('git', ['-C', root, 'show', `${commit}:${path}`], { encoding: 'utf8' });
const sourceFile = (path) =>
  ts.createSourceFile(path, readSource(path), ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);

const initializerFor = (source, name) => {
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

const property = (object, name) => {
  const entry = object.properties.find(
    (candidate) =>
      ts.isPropertyAssignment(candidate) &&
      ((ts.isIdentifier(candidate.name) && candidate.name.text === name) ||
        (ts.isStringLiteral(candidate.name) && candidate.name.text === name)),
  );
  if (!entry || !ts.isPropertyAssignment(entry)) {
    throw new Error(`missing property ${name}`);
  }
  return entry.initializer;
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

const numericOr = (object, name, fallback) => {
  const entry = object.properties.find(
    (candidate) =>
      ts.isPropertyAssignment(candidate) &&
      ((ts.isIdentifier(candidate.name) && candidate.name.text === name) ||
        (ts.isStringLiteral(candidate.name) && candidate.name.text === name)),
  );
  return entry && ts.isPropertyAssignment(entry)
    ? numberValue(entry.initializer, name)
    : fallback;
};

const readDelve = (path, name) => {
  const source = sourceFile(path);
  const initializer = initializerFor(source, name);
  if (!ts.isObjectLiteralExpression(initializer)) {
    throw new Error(`${name} must be an object literal`);
  }
  const rewards = property(initializer, 'baseRewards');
  const tiers = property(initializer, 'tiers');
  if (!ts.isObjectLiteralExpression(rewards) || !ts.isArrayLiteralExpression(tiers)) {
    throw new Error(`${name} reward data is malformed`);
  }
  const base = {
    first_clear_xp: numberValue(property(rewards, 'firstClearXp'), 'firstClearXp'),
    repeat_clear_xp: numberValue(property(rewards, 'repeatClearXp'), 'repeatClearXp'),
    copper_min: numberValue(property(rewards, 'copperMin'), 'copperMin'),
    copper_max: numberValue(property(rewards, 'copperMax'), 'copperMax'),
  };
  return {
    id: stringValue(property(initializer, 'id'), 'id'),
    tiers: tiers.elements.map((element) => {
      if (!ts.isObjectLiteralExpression(element)) {
        throw new Error(`${name} contains a malformed tier`);
      }
      return {
        id: stringValue(property(element, 'id'), 'tier.id'),
        first_clear_xp: numericOr(element, 'firstClearXp', base.first_clear_xp),
        repeat_clear_xp: numericOr(element, 'repeatClearXp', base.repeat_clear_xp),
        copper_min: numericOr(element, 'copperMin', base.copper_min),
        copper_max: numericOr(element, 'copperMax', base.copper_max),
      };
    }),
  };
};

const delves = [
  readDelve('src/sim/content/delves/collapsed_reliquary.ts', 'COLLAPSED_RELIQUARY_DELVE'),
  readDelve('src/sim/content/delves/drowned_litany.ts', 'DROWNED_LITANY_DELVE'),
];
if (
  delves.length !== 2 ||
  delves[0].id !== 'collapsed_reliquary' ||
  delves[1].id !== 'drowned_litany' ||
  delves.some((delve) => delve.tiers.length !== 2 || delve.tiers[0].id !== 'normal' || delve.tiers[1].id !== 'heroic')
) {
  throw new Error('Delve reward roster drifted');
}

const runsPath = 'src/sim/delves/runs.ts';
const runsText = readSource(runsPath);
const runs = ts.createSourceFile(runsPath, runsText, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
const grant = runs.statements.find(
  (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === 'grantDelveClearTo',
);
if (!grant) {
  throw new Error('grantDelveClearTo missing');
}
for (const marker of [
  "const firstClear = !meta.delveDaily.firstClearXp.has(clearKey);",
  'tier?.firstClearXp ?? delve.baseRewards.firstClearXp',
  'tier?.repeatClearXp ?? delve.baseRewards.repeatClearXp',
  "delve.id === 'drowned_litany' ? 2 : 1",
  'tier?.copperMin ?? delve.baseRewards.copperMin',
  'tier?.copperMax ?? delve.baseRewards.copperMax',
]) {
  if (!grant.getText(runs).includes(marker)) {
    throw new Error(`Delve clear reward source drifted: ${marker}`);
  }
}

process.stdout.write(JSON.stringify({ delves, drowned_litany_mark_multiplier: 2 }));
