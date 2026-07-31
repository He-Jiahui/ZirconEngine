// Extracts the Raise Dead channel contract from the pinned simulation AST.
import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const sourceRoot = process.env.WOC_GIT_ROOT;
const sourceCommit = process.env.WOC_GIT_COMMIT;
if (!sourceRoot || !sourceCommit) throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
const sourcePath = 'src/sim/delves/runs.ts';
const sourceText = execFileSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${sourcePath}`], {
  encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
});
const sourceFile = ts.createSourceFile(sourcePath, sourceText, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
const start = findFunction('startDelveRaiseDeadChannel');
const tick = findFunction('tickDelveRaiseDeadChannel');
const interact = findFunction('delveInteract');
const content = {
  channel_seconds: numericConst('DELVE_RAISE_DEAD_CHANNEL'),
  interrupt_object_kind: stringLiteralAssignedToComparison(interact, 'state.kind'),
  completion_requires_living_boss: containsText(tick, '!boss.dead'),
  completion_spawns_boss_adds: containsText(tick, 'ctx.spawnBossAdds'),
  start_requires_cracked_grave: containsText(start, "'cracked_grave'"),
};
process.stdout.write(JSON.stringify(content));

function findFunction(name) { for (const statement of sourceFile.statements) if (ts.isFunctionDeclaration(statement) && statement.name?.text === name) return statement; throw new Error(`missing ${name}`); }
function numericConst(name) { for (const statement of sourceFile.statements) if (ts.isVariableStatement(statement)) for (const declaration of statement.declarationList.declarations) if (ts.isIdentifier(declaration.name) && declaration.name.text === name && ts.isNumericLiteral(declaration.initializer)) return Number(declaration.initializer.text); throw new Error(`missing numeric ${name}`); }
function containsText(node, text) { return node.getText(sourceFile).includes(text); }
function stringLiteralAssignedToComparison(node, leftText) { const text = node.getText(sourceFile); const match = text.match(new RegExp(`${leftText.replace('.', '\\.')} === '([^']+)'`)); if (!match) throw new Error('missing interaction object kind'); return match[1]; }
