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
const declaration = source.statements.find(
  (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === 'delveMarkPayout',
);
if (!declaration) {
  throw new Error('delveMarkPayout missing from source');
}

const functionSource = declaration.getText(source);
for (const marker of [
  "const isHeroic = run.tierId === 'heroic';",
  'if (meta.delveDaily.markClears < 3) return isHeroic ? 2 : 1;',
  'if (isHeroic) return 1;',
  'return ctx.rng.chance(0.5) ? 1 : 0;',
]) {
  if (!functionSource.includes(marker)) {
    throw new Error(`Delve Mark payout source drifted: ${marker}`);
  }
}

process.stdout.write(
  JSON.stringify({
    first_full_clear_limit: 3,
    first_normal_marks: 1,
    first_heroic_marks: 2,
    repeat_heroic_marks: 1,
    repeat_normal_probability: 0.5,
  }),
);
