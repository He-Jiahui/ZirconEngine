import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const root = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;
if (!root || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const path = 'src/sim/delves/runs.ts';
const text = execFileSync('git', ['-C', root, 'show', `${commit}:${path}`], { encoding: 'utf8' });
const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
const declaration = source.statements
  .filter(ts.isVariableStatement)
  .flatMap((statement) => [...statement.declarationList.declarations])
  .find((candidate) => ts.isIdentifier(candidate.name) && candidate.name.text === 'DELVE_LORE_ORDER');
if (!declaration || !declaration.initializer || !ts.isAsExpression(declaration.initializer)) {
  throw new Error('DELVE_LORE_ORDER source declaration is missing');
}
const initializer = declaration.initializer.expression;
if (!ts.isArrayLiteralExpression(initializer)) {
  throw new Error('DELVE_LORE_ORDER must be an array');
}
const loreOrder = initializer.elements.map((element) => {
  if (!ts.isStringLiteral(element)) {
    throw new Error('DELVE_LORE_ORDER must contain string ids');
  }
  return element.text;
});
const unlock = source.statements.find(
  (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === 'unlockNextDelveLore',
);
if (!unlock) {
  throw new Error('unlockNextDelveLore missing from source');
}
for (const marker of [
  'const idx = meta.delveLoreUnlocked.size;',
  'if (idx >= DELVE_LORE_ORDER.length) return;',
  'const loreId = DELVE_LORE_ORDER[idx];',
  'meta.delveLoreUnlocked.add(loreId);',
]) {
  if (!unlock.getText(source).includes(marker)) {
    throw new Error(`Delve lore source drifted: ${marker}`);
  }
}
if (
  loreOrder.length !== 5 ||
  loreOrder[0] !== 'eastbrook_ledger' ||
  loreOrder[4] !== 'tessa_note'
) {
  throw new Error('Delve lore order drifted');
}

process.stdout.write(JSON.stringify({ lore_order: loreOrder }));
