import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const swingAffixes = gitShow('src/sim/mob/mob_swing.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_swing_affix_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'mob_swing_affix_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_mob_swing_affix_state_tests.zrp'), 'utf8'));

for (const needle of [
  "import { isDisarmed } from '../combat/cc';",
  'const disarm = MOBS[mob.templateId]?.disarm;',
  "target.kind === 'player'",
  'ctx.rng.chance(disarm.chance)',
  '!isDisarmed(target)',
  'id: `disarm_${mob.templateId}`,',
  "kind: 'disarm'",
  'remaining: disarm.duration,',
  "school: (disarm.school ?? 'physical') as Aura['school'],",
]) {
  invariant(swingAffixes.includes(needle), `missing pinned mob-disarm behavior: ${needle}`);
}

for (const needle of [
  'pub var hasDisarm: bool;', 'pub var disarmChance: float;',
  'targetIsDisarmed(state: MobSwingAffixState): bool',
  'if (!takeChance(state, "disarm", state.disarmChance)) { return; }',
  'if (targetIsDisarmed(state)) { return; }',
  '"disarm_" + state.templateId,', '"disarm",',
  'applyDisarm(state);', 'crusher.targetAuraRemaining[0] = 1.0;',
  'crusher.randomIndex != 2',
]) {
  invariant(projection.includes(needle), `mob-disarm projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("combat/mob_swing_affix_state")') && testMain.includes('affixState.contractTest()'),
  'missing mob-swing-affix test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_mob_swing_affix_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-swing-affix-state-tests' &&
    testProject.entry === 'combat/mob_swing_affix_state_test_main',
  'mob-swing-affix test project contract drifted',
);

process.stdout.write(`checked M4 mob disarm source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
