import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const profileRunner = gitShow('src/sim/mob/combat_profile.ts');
const profiles = gitShow('src/sim/mob_combat.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_melee_pursuit_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_melee_pursuit_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_mob_melee_pursuit_state_tests.zrp'), 'utf8'));

for (const needle of [
  'export function tryMobMeleeSwingInRange(ctx: SimContext, mob: Entity, target: Entity): boolean {',
  'if (dist2d(mob.pos, target.pos) > mobEffectiveMeleeRange(mob)) return false;',
  "mob.aiState = 'attack';",
  'mob.swingTimer <= 0',
  'mob.swingTimer = mob.weapon.speed * ctx.swingIntervalMult(mob);',
  'function updatePursuitProfileCombat(',
  'mob.swingTimer = Math.max(0, mob.swingTimer - DT);',
  "if (profile.swingWhilePursuing || mob.aiState === 'attack')",
  'if (dist2d(mob.pos, target.pos) > profile.desiredRange) {',
  'mob.moveSpeed * profile.chaseSpeedMult * ctx.moveSpeedMult(mob),',
  'profile.immediateSwingOnEnterRange ||',
  "mob.aiState = dist2d(mob.pos, target.pos) <= profile.meleeRange ? 'attack' : 'chase';",
]) {
  invariant(profileRunner.includes(needle), `source mob pursuit drifted: ${needle}`);
}

for (const needle of [
  'export function effectiveMobMeleeRange(profile: MobCombatProfile, mobMoved: boolean): number {',
  'return profile.meleeRange + profile.movingRangeBonus;',
]) {
  invariant(profiles.includes(needle), `source effective melee range drifted: ${needle}`);
}

for (const needle of [
  'pub class MobMeleePursuitState',
  'pub class MobMeleePursuitTarget',
  'pub initializeMobMeleePursuit(',
  'pub requestMobMeleeSwingIfInRange(',
  'pub stepMobMeleePursuit(',
  '%import("combat/mob_combat_state")',
  'profiles.combatProfileForMob(',
  'profile.swingWhilePursuing',
  'profile.immediateSwingOnEnterRange',
  'mobMotion.stepMobToward(',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC mob melee pursuit is missing: ${needle}`);
}

invariant(
  testMain.includes('%import("combat/mob_melee_pursuit_state")') && testMain.includes('pursuit.contractTest()'),
  'missing mob-melee-pursuit test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_mob_melee_pursuit_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-melee-pursuit-state-tests' &&
    testProject.entry === 'combat/mob_melee_pursuit_state_test_main',
  'mob melee pursuit test project contract drifted',
);

process.stdout.write(`checked M4 mob melee pursuit source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
