// 从固定版本 WOC 源码中提取按 UTC 日期重置的地下探索每日状态，供 m7_delve_daily_content_codegen.mjs 消费。
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
const refresh = source.statements.find(
  (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === 'refreshDelveDaily',
);
if (!refresh) {
  throw new Error('refreshDelveDaily missing from source');
}
// 确认 UTC 日期重置分支存在后，才输出三个每日状态标志。
for (const marker of [
  'const today = ctx.utcDay;',
  'if (today && meta.delveDaily.date !== today) {',
  'meta.delveDaily = { date: today, firstClearXp: new Set(), markClears: 0 };',
]) {
  if (!refresh.getText(source).includes(marker)) {
    throw new Error(`Delve daily reset source drifted: ${marker}`);
  }
}

process.stdout.write(
  JSON.stringify({
    empty_utc_day_keeps_state: true,
    resets_first_clear_xp: true,
    resets_mark_clears: true,
  }),
);
