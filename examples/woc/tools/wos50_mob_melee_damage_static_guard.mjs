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
const sim = gitShow('src/sim/sim.ts');
const zone = gitShow('src/sim/content/zone1.ts');

for (const needle of [
  'mobSwing(mob: Entity, target: Entity): void {',
  'const roll = this.rng.next();',
  'this.rng.range(mob.weapon.min, mob.weapon.max)',
  'const crit = this.rng.chance(0.05);',
  'this.dealDamage(mob, target, dealt, crit, \'physical\', null, \'hit\');',
]) {
  invariant(sim.includes(needle), `source melee damage drifted: ${needle}`);
}

const wolf = sourceTemplate('forest_wolf', 'old_greyjaw');
const boar = sourceTemplate('wild_boar', 'webwood_spider');
for (const fragment of [wolf, boar]) {
  for (const forbidden of ['lifeleech:', 'rampage:', 'cleave:', 'venom:', 'soulrot:']) {
    invariant(!fragment.includes(forbidden), `Eastbrook base mob gained on-hit affix: ${forbidden}`);
  }
}

for (const needle of [
  'var mobSwing = %import("combat/mob_swing_state");',
  'applyOfflineMobSwingWarriorDefense(',
  'applyOfflineMobMeleePlayerDeath(',
  'resolveOfflineEastbrookMobSwingRequests(',
  'mobSwing.armorReductionFromArmor(',
  'mobSwing.sourceSwingMissChance(',
  'mobSwing.initializeAuthoritativeRng(',
  'mobSwing.swing(swing) == "hit"',
  'state.rngState = mobSwing.authoritativeRngState(swing, true);',
  'state.entityHp[playerIndex] = hp > 0 ? hp : 0;',
  'pub offlineEastbrookMobMeleePursuitStateTest(): int',
  '<uint>contact.rngDraws != <uint>3',
  '!<bool>lethal.entityDead[0]',
  '<uint>lethal.entityAiStates[1] != mobLifecycle.idleAiState()',
]) {
  invariant(state.includes(needle), `WOS melee damage integration is missing: ${needle}`);
}
invariant(
  /resolveOfflineEastbrookMobSwingRequests\(\s*state, index, targetIndex, pursuit\.swingRequests/.test(state),
  'WOS melee damage integration is missing: pursuit swing request handoff',
);

process.stdout.write(`checked WOS50 mob melee damage source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function sourceTemplate(start, next) {
  const beginning = zone.indexOf(`id: '${start}'`);
  const ending = zone.indexOf(`id: '${next}'`, beginning + 1);
  invariant(beginning >= 0 && ending > beginning, `cannot locate ${start} source template`);
  return zone.slice(beginning, ending);
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
