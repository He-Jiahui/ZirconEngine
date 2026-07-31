import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const state = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);

for (const needle of [
  'var idleAggro = %import("world/mob_idle_aggro_state");',
  'enterIdleMobAggro(state: WorldState, primaryIndex: int, targetId: uint): bool',
  'stepOfflineEastbrookMobIdleAggro(state: WorldState): void',
  'idleAggro.selectIdleAggroTarget(',
  'campMobCore.metric(templateIndex, "aggro_radius", true)',
  'campMobCore.flag(templateIndex, "elite", true)',
  'campMobCore.flag(templateIndex, "rare", true)',
  'campMobCore.flag(templateIndex, "boss", true)',
  'enterIdleMobAggro(state, index, selection.targetId)',
  'stepOfflineEastbrookMobIdleAggro(state);',
  'pub offlineIdleProximityAggroStateTest(): int',
  'offlineIdleProximityAggroStateTest() != 1',
]) {
  invariant(state.includes(needle), `WOS idle proximity aggro is missing: ${needle}`);
}

process.stdout.write('checked WOS50 idle proximity aggro source\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
