import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const projectionRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const types = gitShow('src/sim/types.ts');
const encounter = gitShow('src/sim/encounters/nythraxis.ts');
const projection = readFileSync(
  resolve(projectionRoot, 'src', 'combat', 'nythraxis_quest_interact_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(projectionRoot, 'src', 'combat', 'nythraxis_quest_interact_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(readFileSync(
  resolve(projectionRoot, 'woc_m4_nythraxis_quest_interact_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  'export function questObjectiveRequired(',
  'return progress?.resolvedCounts?.[objectiveIndex] ?? quest.objectives[objectiveIndex]?.count ?? 0;',
]) {
  invariant(types.includes(needle), `missing current-head resolved quest-count behavior: ${needle}`);
}
for (const needle of [
  "obj.objectItemId === 'crypt_ritual_circle'",
  'qp.counts[killIdx] < questObjectiveRequired(quest, qp, killIdx)',
  'qp.counts[objectiveIndex] >= questObjectiveRequired(quest, qp, objectiveIndex)',
  'const required = questObjectiveRequired(quest, memberQp, objectiveIndex);',
  'memberQp.counts[objectiveIndex] >= required',
  'required,',
  'NYTHRAXIS_PARTY_INTERACT_RANGE = 30;',
  'memberQp.counts[objectiveIndex] >= objective.count',
  'return eligible.some((member) => member.entityId === actor.entityId) ? eligible : [actor];',
]) {
  invariant(encounter.includes(needle), `missing current-head Nythraxis quest-interact behavior: ${needle}`);
}

for (const needle of [
  'pub class NythraxisQuestInteractState',
  'pub resolvedObjectiveRequired(',
  'pub interactQuestObject(',
  'isSharedNythraxisObject(',
  'NYTHRAXIS_PARTY_INTERACT_RANGE',
  'state.guardianSummonRequests = state.guardianSummonRequests + 1;',
  'if (actor.objectiveCount >= resolvedObjectiveRequired(state, actor)) { return true; }',
  'if (member.objectiveCount < required) {',
  'state.visionRecipientIds.add(<int>recipients[index]);',
  'shared.objectiveBaseRequired = 3;',
  'ritual.objectItemId = "crypt_ritual_circle";',
]) {
  invariant(projection.includes(needle), `Nythraxis quest-interact projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("combat/nythraxis_quest_interact_state")') &&
    testMain.includes('questInteract.contractTest()'),
  'missing Nythraxis quest-interact test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_nythraxis_quest_interact_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-nythraxis-quest-interact-state-tests' &&
    testProject.entry === 'combat/nythraxis_quest_interact_state_test_main',
  'Nythraxis quest-interact test project contract drifted',
);

process.stdout.write(`checked M4 Nythraxis quest-interact source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
