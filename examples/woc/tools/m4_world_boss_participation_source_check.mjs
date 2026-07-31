import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/world_boss.ts');
const sourceSim = gitShow('src/sim/sim.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'world_boss_participation_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'world_boss_participation_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(
    resolve(wocRoot, 'scripts', 'woc_game', 'woc_m4_world_boss_participation_state_tests.zrp'),
    'utf8',
  ),
);

for (const needle of [
  'export const WORLD_BOSS_INTERVAL_SECONDS = 1 * 3600;',
  'export const WORLD_BOSS_CORPSE_SECONDS = 900;',
  "templateId: 'thunzharr_waking_peak',",
  'pos: { x: 110, z: 760 },',
  'hpScale: { base: 40_000, perPlayer: 5_000, max: 1_000_000 },',
  "export const WORLD_BOSS_LOCKOUT_PREFIX = 'worldboss:';",
  'return until === undefined || until <= nowMs;',
  'if (untilMs > 0) meta.raidLockouts.set(worldBossLockoutId(bossId), untilMs);',
  'return out.sort((a, b) => a.entityId - b.entityId);',
  'for (const pid of mob.bossDamagers) add(pid);',
  'if (boss.maxHp >= def.hpScale.max) return;',
  'boss.hp = Math.min(boss.maxHp, boss.hp + delta);',
]) {
  invariant(source.includes(needle), `source world-boss rule drifted: ${needle}`);
}

for (const needle of [
  'if (boss.corpseTimer <= 0) {',
  'this.worldBossNextAt[i] += def.intervalSeconds;',
  'if (this.worldBossEntityIds[i] === null) {',
]) {
  invariant(sourceSim.includes(needle), `source world-boss scheduler drifted: ${needle}`);
}

for (const needle of [
  'pub worldBossIntervalSeconds(): int { return 3600; }',
  'pub worldBossCorpseSeconds(): int { return 900; }',
  'pub worldBossTemplateId(): string { return "thunzharr_waking_peak"; }',
  'pub worldBossLockoutPrefix(): string { return "worldboss:"; }',
  'return ownerId != <uint>0 ? ownerId : attackerId;',
  'appendActiveSorted(',
  'pub worldBossContributors(',
  'pub worldBossLootContributors(',
  'pub scaledWorldBossMaximumForParticipants(participants: int): int {',
  'boss.hp = boss.hp + delta;',
  'pub resetWorldBossPull(state: WorldBossParticipationState): void {',
  'state.participants[index].lockoutUntilMs <= nowMs;',
  'pub class WorldBossScheduleState {',
  'pub initializeWorldBossSchedule(state: WorldBossScheduleState, atBoot: bool): void {',
  'if (state.bossPresent && state.bossDead && state.corpseTimer <= 0.0) {',
  'if (now < state.nextAt) { return; }',
  'state.nextAt = state.nextAt + <float>worldBossIntervalSeconds();',
]) {
  invariant(projection.includes(needle), `WOC world-boss projection is missing: ${needle}`);
}

for (const needle of [
  '%import("world/world_boss_participation_state")',
  'state.addThreat(<uint>50, <uint>2);',
  'boss.scaledWorldBossMaximumForParticipants(201) != boss.worldBossMaximumHp()',
  'boss.markWorldBossLooted(state, <uint>5, 9999999999)',
  'boss.resetWorldBossPull(state);',
  'boss.updateWorldBossSchedule(schedule, 7200.0, true);',
  'boss.initializeWorldBossSchedule(atBoot, true);',
]) {
  invariant(testMain.includes(needle), `WOC world-boss contract is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m4_world_boss_participation_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-world-boss-participation-state-tests' &&
    testProject.entry === 'world/world_boss_participation_state_test_main',
  'world-boss participation test project contract drifted',
);

process.stdout.write(`checked M4 world-boss participation source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
