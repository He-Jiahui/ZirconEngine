import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const state = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);

for (const needle of [
  'var socialAggro = %import("world/fleeing_social_aggro_state");',
  'offlineMobTemplateId(state: WorldState, index: int): string',
  'stateThreatValue(state: WorldState, entityIndex: int, targetId: uint): float',
  'applyOfflineIdleSocialPull(state: WorldState, primaryIndex: int, targetId: uint): void',
  'socialAggro.normalSocialPull(',
  'state.entityLeashAnchorPresent[primaryIndex] = true;',
  'state.entityAggroTargetIds[primaryIndex] = targetId;',
  'state.entityCombatTimers[actorIndex] = 0.0;',
  'applyOfflineIdleSocialPull(state, primaryIndex, targetId);',
  'state.entityThreatValues[0] != 2.0',
]) {
  invariant(state.includes(needle), `WOS idle social pull is missing: ${needle}`);
}

process.stdout.write('checked WOS50 idle social pull source\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
