import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const root = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;
if (!root || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const readSource = (sourcePath) =>
  execFileSync('git', ['-C', root, 'show', `${commit}:${sourcePath}`], { encoding: 'utf8' });
const path = 'src/sim/delves/runs.ts';
const text = readSource(path);
const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
const update = source.statements.find(
  (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === 'updateDelveRuns',
);
if (!update) {
  throw new Error('updateDelveRuns missing from source');
}
for (const marker of [
  'if (ctx.tickCount % 20 !== 0) return;',
  'Math.abs(e.pos.x - origin.x) < 120',
  'Math.abs(e.pos.z - origin.z) < delveOccupancyRadius(run)',
  'if (occupied) run.emptyFor = 0;',
  'run.emptyFor += 1;',
  'if (run.emptyFor >= INSTANCE_EMPTY_TIMEOUT) freeDelveRun(ctx, run);',
]) {
  if (!update.getText(source).includes(marker)) {
    throw new Error(`Delve empty reset source drifted: ${marker}`);
  }
}
const typesText = readSource('src/sim/types.ts');
if (!typesText.includes('export const INSTANCE_EMPTY_TIMEOUT = 300;')) {
  throw new Error('INSTANCE_EMPTY_TIMEOUT source drifted');
}

process.stdout.write(
  JSON.stringify({
    empty_check_tick_interval: 20,
    occupancy_x_radius: 120,
    empty_timeout_seconds: 300,
    strict_bounds: true,
  }),
);
