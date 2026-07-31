// Extracts the source Delve perception multiplier without requiring a live run object.
import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const sourceRoot = process.env.WOC_GIT_ROOT;
const sourceCommit = process.env.WOC_GIT_COMMIT;
if (!sourceRoot || !sourceCommit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const sourcePath = 'src/sim/delves/runs.ts';
const sourceText = execFileSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${sourcePath}`], {
  encoding: 'utf8',
});
const sourceFile = ts.createSourceFile(sourcePath, sourceText, ts.ScriptTarget.Latest, true,
  ts.ScriptKind.TS);
const declaration = sourceFile.statements.find((statement) =>
  ts.isFunctionDeclaration(statement) && statement.name?.text === 'delveDetectMult');
if (!declaration?.body) {
  throw new Error('delveDetectMult is missing');
}
const body = declaration.getText(sourceFile);
for (const marker of ["!run?.affixes.includes('candleblind')", 'return 1;', 'return 0.65;']) {
  if (!body.includes(marker)) {
    throw new Error(`Candleblind source behavior drifted: ${marker}`);
  }
}

process.stdout.write(JSON.stringify({
  inactive_multiplier: 1,
  active_multiplier: 0.65,
  requires_active_affix: true,
}));
