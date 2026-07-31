import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const entity = gitShow('src/sim/entity.ts');
const sim = gitShow('src/sim/sim.ts');
const types = gitShow('src/sim/types.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'spell_combat_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'spell_combat_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_spell_combat_state_tests.zrp'), 'utf8'));

for (const needle of [
  "else if (a.kind === 'buff_crit' || a.kind === 'buff_reckless' || a.kind === 'bloodbath')",
  'bonusCrit += a.value;',
  'e.sharedCritBonus =',
  'bonusCrit + (mods?.stats.crit ?? 0) + setEff.crit + critFractionFromRating(e.critRating);',
  '0.05 +',
  's.agi * 0.0005 +',
  'e.sharedCritBonus +',
  "a.kind === 'berserker_stance'",
]) {
  invariant(entity.includes(needle), `missing pinned entity shared-crit behavior: ${needle}`);
}
for (const needle of [
  'export const CRIT_RATING_PER_PCT = 10;',
  'return rating / (CRIT_RATING_PER_PCT * 100);',
]) {
  invariant(types.includes(needle), `missing pinned crit-rating conversion: ${needle}`);
}
invariant(
  sim.includes('return 0.05 + p.stats.int * 0.0008 + (p.sharedCritBonus ?? 0) + spellCritBonusFromAuras(p);'),
  'missing pinned spell shared-crit composition',
);

for (const needle of [
  'pub flatCritBonusFromAuras(', '"buff_crit" || aura.kind == "buff_reckless" || aura.kind == "bloodbath"',
  'pub critFractionFromRating(critRating: float): float', 'return critRating / 1000.0;',
  'pub sharedCritBonus(', 'pub spellCritChance(', 'pub meleeCritChance(',
  'return 0.05 + intellect * 0.0008 + sharedBonus + spellCritBonusFromAuras(liveAuras);',
  'return 0.05 + agility * 0.0005 + sharedBonus + berserkerBonus;',
  'berserkerBonus = 0.03;',
]) {
  invariant(projection.includes(needle), `shared-crit projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("combat/spell_combat_state")') && testMain.includes('spell.contractTest()'),
  'missing spell-combat test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_spell_combat_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-spell-combat-state-tests' &&
    testProject.entry === 'combat/spell_combat_state_test_main',
  'spell-combat test project contract drifted',
);

process.stdout.write(`checked M4 shared spell-crit core projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
