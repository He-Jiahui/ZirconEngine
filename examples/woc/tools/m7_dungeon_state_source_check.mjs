import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const dungeons = gitShow('src/sim/instances/dungeons.ts');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src');
const projection = readFileSync(
  resolve(wocSourceRoot, 'instances', 'dungeon_state.zr'),
  'utf8',
);

for (const needle of [
  'inst.enteredBy.add(r.meta.entityId);',
  'inst.enteredBy = new Set();',
  'awardHeroicMarks pays the mail',
  'function scrubInstanceThreat(ctx: SimContext, inst: InstanceSlot, pid: number): void {',
  'dropThreat(mob, pid);',
  'if (ctx.entities.get(srcId)?.ownerId === pid) dropThreat(mob, srcId);',
  'if (mob.aggroTargetId === pid || tgt?.ownerId === pid) mob.aggroTargetId = null;',
  'const credited = recipients.length > 0;',
  'const presentIds = new Set(recipients.map((meta) => meta.entityId));',
  'if (presentIds.has(meta.entityId)) {',
  'ctx.mailHeroicMarks(meta.entityId, HEROIC_MARK_ITEM_ID, tuning.marksPerParticipant);',
]) {
  invariant(dungeons.includes(needle), `missing current dungeon participation or reward behavior: ${needle}`);
}

for (const needle of [
  '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a',
  'pub var enteredBy:',
  'pub enteredDungeon(',
  'recordDungeonEntry(state, player);',
  'clearDungeonEntries(state);',
  'pub heroicRewardDelivery(',
  'if (!creditedKill || alreadyLocked) { return 0; }',
  'if (presentAtKill) { return 1; }',
  'return enteredThisRun ? 2 : 0;',
  'class DungeonMobThreatState',
  'pub addDungeonThreatSource(',
  'pub scrubLeavingPlayerThreat(',
  'state.threatSourceOwnerIds.removeAt(index);',
  'state.aggroTargetOwnerId = 0;',
  'dungeonCurrentHeadParticipationTest(): int',
]) {
  invariant(projection.includes(needle), `dungeon projection omitted current participation or reward behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("instances/dungeon_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['instances/dungeon_state_test_main.zr', 'instances/m7_scenario_matrix.zr']),
  `dungeon_state escaped the M7 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M7 dungeon participation, departure threat, and heroic reward routing: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
