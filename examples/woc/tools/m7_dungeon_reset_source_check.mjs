import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const projectionRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const dungeons = gitShow('src/sim/instances/dungeons.ts');
const projection = readFileSync(
  resolve(projectionRoot, 'src', 'instances', 'dungeon_reset_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(projectionRoot, 'src', 'instances', 'dungeon_reset_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(readFileSync(
  resolve(projectionRoot, 'woc_m7_dungeon_reset_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  'function resetOwnerPids(ctx: SimContext, pid: number): number[] {',
  'function resetCooldownKey(ctx: SimContext, pid: number, dungeonId: string): string {',
  'function activeResetLock(',
  'export function inheritDungeonResetLocks(ctx: SimContext, pid: number): void {',
  'lock.claimId !== inst?.exitId',
  'export function resetDungeonInstances(ctx: SimContext, pid?: number): void {',
  'inst.difficulty !== claimDifficultyForDungeon(inst.dungeonId, selected)',
  'inst.resetAvailableAt > ctx.time ||',
  "'Instances can only be reset once every 5 minutes.'",
  "'You cannot reset instances while someone is still inside.'",
  "'You cannot reset instances while loot remains inside.'",
  'claimInstance(ctx, inst, key, claimDifficultyForDungeon(inst.dungeonId, selected));',
  'ctx.dungeonResetLocks.set(resetCooldownKey(ctx, ownerPid, inst.dungeonId), {',
  'claimId: inst.exitId,',
]) {
  invariant(dungeons.includes(needle), `missing current-head dungeon reset behavior: ${needle}`);
}

for (const needle of [
  'pub class DungeonResetState',
  'pub class DungeonResetClaim',
  'pub addResetOwner(',
  'pub addResetClaim(',
  'pub resetOwnedInstances(',
  'pub enterResetClaim(',
  'pub inheritResetLocks(',
  'RESET_COOLDOWN_SECONDS',
  'claim.difficulty != effectiveClaimDifficulty(state, claim)',
  'candidate.someoneInside || candidate.corpseInside',
  'candidate.lootRemaining',
  'state.lockClaimIds.add(claimId);',
  'lockedClaimId != claimId',
  'return -4;',
  'return -5;',
  'return -6;',
]) {
  invariant(projection.includes(needle), `dungeon reset projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("instances/dungeon_reset_state")') &&
    testMain.includes('reset.contractTest()'),
  'missing dungeon reset test entry behavior',
);
invariant(
  testProject.name === 'woc_m7_dungeon_reset_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m7-dungeon-reset-state-tests' &&
    testProject.entry === 'instances/dungeon_reset_state_test_main',
  'dungeon reset test project contract drifted',
);

process.stdout.write(`checked M7 dungeon reset source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
