import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const source = gitShow('src/sim/mob_combat.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_combat_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_combat_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_mob_combat_state_tests.zrp'), 'utf8'));

for (const needle of [
  'export const DEFAULT_MOB_COMBAT_PROFILE: MobCombatProfile = {',
  'meleeRange: MELEE_RANGE,',
  'desiredRange: MELEE_RANGE * 0.8,',
  'chaseSpeedMult: 1,',
  'canLeash: true,',
  'swingWhilePursuing: true,',
  'immediateSwingOnEnterRange: true,',
  'movingRangeBonus: 1,',
  'export const NYTHRAXIS_BOSS_COMBAT_PROFILE',
  'export const NYTHRAXIS_ADD_COMBAT_PROFILE',
  'const THUNZHARR_REACH_SCALE = 5;',
  "templateId === 'nythraxis_scourge_of_thornpeak'",
  "templateId === 'nythraxis_skeleton_warrior'",
  "templateId === 'thunzharr_waking_peak'",
  'export function effectiveMobMeleeRange(profile: MobCombatProfile, mobMoved: boolean): number {',
  'return profile.meleeRange + profile.movingRangeBonus;',
]) {
  invariant(source.includes(needle), `source mob combat profile drifted: ${needle}`);
}

for (const needle of [
  'pub class MobCombatProfile',
  'pub scaledDefaultMobMeleeRange(',
  'pub combatProfileForMob(',
  'pub effectiveMobMeleeRange(',
  'pub var chaseSpeedMult: float;',
  '"nythraxis_scourge_of_thornpeak"',
  '"nythraxis_skeleton_warrior"',
  '"thunzharr_waking_peak"',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC mob combat profile is missing: ${needle}`);
}

invariant(
  testMain.includes('%import("combat/mob_combat_state")') && testMain.includes('mobCombat.contractTest()'),
  'missing mob-combat-profile test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_mob_combat_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-combat-state-tests' &&
    testProject.entry === 'combat/mob_combat_state_test_main',
  'mob combat profile test project contract drifted',
);

process.stdout.write(`checked M4 mob combat profile source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
