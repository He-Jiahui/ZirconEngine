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

const companionSource = sourceFile('src/sim/content/delves/companions.ts');
const companionTable = initializerFor(companionSource, 'DELVE_COMPANIONS');
const upgradeCosts = initializerFor(companionSource, 'COMPANION_UPGRADE_COSTS');
if (!ts.isObjectLiteralExpression(companionTable) || !ts.isObjectLiteralExpression(upgradeCosts)) {
  throw new Error('Delve companion tables must be object literals');
}
const companions = companionTable.properties.map((entry) => {
  if (!ts.isPropertyAssignment(entry) || !ts.isIdentifier(entry.name) || !ts.isObjectLiteralExpression(entry.initializer)) {
    throw new Error('DELVE_COMPANIONS structure drifted');
  }
  const value = entry.initializer;
  return {
    id: stringValue(property(value, 'id'), 'companion.id'),
    role: stringValue(property(value, 'role'), 'companion.role'),
    mob_template_id: stringValue(property(value, 'mobTemplateId'), 'companion.mobTemplateId'),
  };
});
const costs = upgradeCosts.properties.map((entry) => {
  if (!ts.isPropertyAssignment(entry) || !ts.isNumericLiteral(entry.name) || !ts.isObjectLiteralExpression(entry.initializer)) {
    throw new Error('COMPANION_UPGRADE_COSTS structure drifted');
  }
  return {
    rank: Number(entry.name.text),
    marks: numberValue(property(entry.initializer, 'marks'), 'upgrade.marks'),
    copper: numberValue(property(entry.initializer, 'copper'), 'upgrade.copper'),
  };
});

const typesText = readSource('src/sim/types.ts');
if (!typesText.includes('export const DELVE_COMPANION_MAX_RANK = 3;')) {
  throw new Error('DELVE_COMPANION_MAX_RANK source drifted');
}
const runsPath = 'src/sim/delves/runs.ts';
const runs = sourceFile(runsPath);
const upgrade = runs.statements.find(
  (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === 'companionUpgrade',
);
if (!upgrade) {
  throw new Error('companionUpgrade missing from source');
}
for (const marker of [
  'const rank = r.meta.companionUpgrades[companionId] ?? 1;',
  'if (rank >= DELVE_COMPANION_MAX_RANK)',
  'const next = rank + 1;',
  'const cost = COMPANION_UPGRADE_COSTS[next];',
  'r.meta.delveMarks -= cost.marks;',
  'r.meta.copper -= cost.copper;',
]) {
  if (!upgrade.getText(runs).includes(marker)) {
    throw new Error(`Delve companion upgrade source drifted: ${marker}`);
  }
}

if (
  companions.length !== 2 ||
  companions[0].id !== 'companion_tessa' ||
  companions[1].id !== 'companion_edda' ||
  costs.length !== 2 ||
  costs[0].rank !== 2 ||
  costs[1].rank !== 3
) {
  throw new Error('Delve companion catalog drifted');
}

process.stdout.write(JSON.stringify({ max_rank: 3, companions, upgrade_costs: costs }));
