import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const source = gitShow('src/sim/sim.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'world', 'fleeing_social_aggro_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'world', 'fleeing_social_aggro_state_test_main.zr'), 'utf8');

for (const needle of [
  'const DEFAULT_SOCIAL_PULL_RADIUS = 5;',
  'const SOCIAL_PULL_RADIUS: Partial<Record<MobFamily, number>> = {',
  'mudfin: 8,',
  "const family = MOBS[mob.templateId]?.family;",
  'const pullRadius = (family && SOCIAL_PULL_RADIUS[family]) ?? DEFAULT_SOCIAL_PULL_RADIUS;',
  "m.kind === 'mob'",
  "m.aiState === 'idle'",
  'm.ownerId === null',
  'm.templateId === mob.templateId',
  'd2 < pullRadius * pullRadius',
  "m.aiState = 'chase';",
  'm.aggroTargetId = target.id;',
  'm.leashAnchor = { ...m.pos };',
  'addThreat(m, target.id, 1);',
]) {
  invariant(source.includes(needle), `source normal social aggro drifted: ${needle}`);
}

for (const needle of [
  'pub var templateId: string;',
  'pub var threatAmounts: container.Array<float>;',
  'addThreat(mob: FleeSocialAggroMob, targetId: int, amount: float): void',
  'pub normalSocialPullRadius(',
  'pub normalSocialPull(',
  'candidate.templateId == pulling.templateId',
  'distanceSquared < radiusSquared',
  'candidate.leashX = candidate.x;',
  'candidate.leashY = candidate.y;',
  'candidate.leashZ = candidate.z;',
  'pub normalSocialPullContractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC normal social aggro is missing: ${needle}`);
}

invariant(
  testMain.includes('socialAggro.normalSocialPullContractTest()'),
  'missing normal social aggro test entry behavior',
);

process.stdout.write(`checked M4 normal social aggro source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
