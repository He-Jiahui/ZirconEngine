import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const sim = gitShow('src/sim/sim.ts');
const types = gitShow('src/sim/types.ts');
const warrior = gitShow('src/sim/combat/warrior_hit_table.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_swing_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_swing_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_mob_swing_state_tests.zrp'), 'utf8'));

for (const needle of [
  'mobSwing(mob: Entity, target: Entity): void {',
  'const missChance = swingMissChance(mob, target);',
  "const dodgeChance = target.kind === 'player' ? target.dodgeChance : 0.05;",
  'const { parryChance, blockChance } = warriorMeleeDefense(target, mob);',
  'const roll = this.rng.next();',
  'this.rng.range(mob.weapon.min, mob.weapon.max)',
  'const crit = this.rng.chance(0.05);',
  'dmg *= 1 - armorReduction(this.effectiveArmor(target), mob.level);',
  'roll < missChance + dodgeChance + parryChance + blockChance',
  'const dealt = Math.max(1, Math.round(dmg));',
  'runMobSwingAffixes(this.ctx, mob, target, { dealt, crit, rawDmg });',
]) {
  invariant(sim.includes(needle), `source mobSwing drifted: ${needle}`);
}

for (const needle of [
  'export function swingMissChance(attacker: Entity, target: Entity): number {',
  'if (mobAttacker && playerSide) return Math.min(miss, MOB_VS_PLAYER_MAX_MISS);',
  'export function armorReduction(armor: number, attackerLevel: number): number {',
  'return Math.min(0.75, a / (a + 85 * attackerLevel + 400));',
]) {
  invariant(types.includes(needle), `source combat rule drifted: ${needle}`);
}

for (const needle of [
  "if (defender.kind !== 'player' || defender.templateId !== 'warrior')",
  'Math.abs(normAngle(angleTo(defender.pos, attacker.pos) - defender.facing)) < WARRIOR_FRONT_ARC',
  'parryChance: warriorParryChance(defender.stats.str)',
  'blockChance: defender.blockValue > 0 && defender.blockChance > 0 ? defender.blockChance : 0',
]) {
  invariant(warrior.includes(needle), `source warrior defense drifted: ${needle}`);
}

for (const needle of [
  'pub class MobSwingState',
  'pub sourceSwingMissChance(',
  'pub armorReductionFromArmor(',
  'pub initializeAuthoritativeRng(',
  'pub authoritativeRngState(',
  'state.rngState = (state.rngState + <uint>1831565813) & <uint>4294967295;',
  'state.rngDigest = rng.fold(state.rngDigest, value, 1);',
  'state.targetParryChance',
  'state.targetBlockChance',
  'state.blockEvents = state.blockEvents + 1;',
  'pub swing(state: MobSwingState): string',
  'pub contractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC mob swing is missing: ${needle}`);
}

invariant(
  testMain.includes('%import("combat/mob_swing_state")') && testMain.includes('mobSwing.contractTest()'),
  'missing mob-swing test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_mob_swing_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-swing-state-tests' &&
    testProject.entry === 'combat/mob_swing_state_test_main',
  'mob-swing test project contract drifted',
);

process.stdout.write(`checked M4 mob swing source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
