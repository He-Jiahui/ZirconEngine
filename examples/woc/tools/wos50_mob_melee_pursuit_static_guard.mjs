import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const state = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);
const pursuit = gitShow('src/sim/mob/combat_profile.ts');

for (const needle of [
  'function updatePursuitProfileCombat(',
  'mob.swingTimer = Math.max(0, mob.swingTimer - DT);',
  'if (profile.swingWhilePursuing || mob.aiState === \'attack\')',
  'mob.moveSpeed * profile.chaseSpeedMult * ctx.moveSpeedMult(mob),',
  "mob.aiState = dist2d(mob.pos, target.pos) <= profile.meleeRange ? 'attack' : 'chase';",
]) {
  invariant(pursuit.includes(needle), `source pursuit drifted: ${needle}`);
}

for (const needle of [
  'var mobMeleePursuit = %import("combat/mob_melee_pursuit_state");',
  'isOfflineEastbrookMeleePursuitMob(state: WorldState, index: int): bool',
  'stepOfflineEastbrookMobMeleePursuit(state: WorldState): void',
  'mobMeleePursuit.initializeMobMeleePursuit(',
  'mobMeleePursuit.setMobMeleePursuitTarget(',
  'mobMeleePursuit.stepMobMeleePursuit(',
  'state.entitySwingTimer[index] = pursuit.swingTimer;',
  'state.entityAiStates[index] = pursuing ?',
  'stepOfflineEastbrookMobMeleePursuit(state);',
  'pub offlineEastbrookMobMeleePursuitStateTest(): int',
  'offlineEastbrookMobMeleePursuitStateTest() != 1',
]) {
  invariant(state.includes(needle), `WOS melee pursuit integration is missing: ${needle}`);
}

process.stdout.write(`checked WOS50 mob melee pursuit source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
