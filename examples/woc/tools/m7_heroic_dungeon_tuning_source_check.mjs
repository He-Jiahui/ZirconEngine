import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const difficulty = gitShow('src/sim/instances/difficulty.ts');
const tuning = gitShow('src/sim/content/dungeon_difficulty.ts');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const projection = readFileSync(
  resolve(wocSourceRoot, 'src', 'instances', 'heroic_dungeon_tuning.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocSourceRoot, 'src', 'instances', 'heroic_dungeon_tuning_test_main.zr'),
  'utf8',
);
const lifecycle = readFileSync(resolve(wocSourceRoot, 'src', 'main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(
  resolve(wocSourceRoot, 'woc_m7_heroic_dungeon_tuning_tests.zrp'),
  'utf8',
));

for (const needle of [
  'export const HEROIC_MIN_MOVE_SPEED = 8;',
  "return selected === 'heroic' && HEROIC_DUNGEON_IDS.has(dungeonId) ? 'heroic' : 'normal';",
  'const dmgMult = role?.summonedAdd ? tuning.addDamageMultiplier : tuning.damageMultiplier;',
  'moveSpeed: Math.max(template.moveSpeed, HEROIC_MIN_MOVE_SPEED),',
  'mob.mechanicDamageMult = role?.summonedAdd ? tuning.addDamageMultiplier : tuning.damageMultiplier;',
  'mob.mechanicHealMult = tuning.healthMultiplier;',
  'mob.ccImmune = true;',
  'mob.slowImmune = true;',
]) {
  invariant(difficulty.includes(needle), `missing pinned heroic difficulty behavior: ${needle}`);
}
for (const needle of [
  "hollow_crypt: {", 'healthMultiplier: 1.9,', 'damageMultiplier: 5.1,', 'addDamageMultiplier: 2.55,',
  "sunken_bastion: {", 'damageMultiplier: 4.65,', 'addDamageMultiplier: 2.3,',
  "drowned_temple: {", 'healthMultiplier: 2.6,', 'damageMultiplier: 4.3,', 'addDamageMultiplier: 2.15,', 'armorMultiplier: 1.25,',
  "gravewyrm_sanctum: {", 'damageMultiplier: 4.05,',
  "nythraxis_boss_arena: {", 'healthMultiplier: 1.6,', 'marksPerParticipant: 3,',
]) {
  invariant(tuning.includes(needle), `missing pinned heroic tuning fact: ${needle}`);
}
for (const needle of [
  'pub claimDifficulty(', 'pub healthMultiplier(', 'pub damageMultiplier(',
  'pub armorMultiplier(', 'pub tunedMoveSpeed(', 'pub bossImmune(',
  'pub finalBossId(', 'pub marksPerParticipant(', 'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `heroic tuning projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("instances/heroic_dungeon_tuning")') &&
    testMain.includes('tuning.contractTest()'),
  'missing M7 heroic tuning test entry behavior',
);
invariant(
  lifecycle.includes('%import("instances/heroic_dungeon_tuning")') &&
    lifecycle.includes('heroicDungeonTuning.contractTest()'),
  'main lifecycle omitted heroic tuning self-test',
);
invariant(
  testProject.name === 'woc_m7_heroic_dungeon_tuning_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m7-heroic-dungeon-tuning-tests' &&
    testProject.entry === 'instances/heroic_dungeon_tuning_test_main',
  'M7 heroic dungeon tuning test project contract drifted',
);

process.stdout.write(`checked M7 heroic-dungeon tuning source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
