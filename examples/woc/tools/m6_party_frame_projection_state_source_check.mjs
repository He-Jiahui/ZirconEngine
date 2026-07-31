import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/party_frame_info.ts');
const types = gitShow('src/sim/types.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'party_frame_projection_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(
    wocRoot,
    'scripts',
    'woc_game',
    'src',
    'social',
    'party_frame_projection_state_test_main.zr',
  ),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_party_frame_projection_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  'const relevant = auras.filter((aura) => isPartyFrameRelevantAura(aura));',
  'relevant.sort((a, b) => partyAuraPriority(a) - partyAuraPriority(b));',
  'return relevant.slice(0, cap).map((aura) => ({',
  '...(aura.value < 0 ? { neg: 1 as const } : {}),',
  'remaining: Math.max(0, Math.ceil(aura.remaining)),',
  "if (aura.kind === 'absorb') total += Math.max(0, aura.value);",
  "entity.kind === 'mob' &&",
  'targets.add(entity.aggroTargetId);',
  "if (effect.type === 'heal' || effect.type === 'chainHeal') {",
  'amount += (effect.min + effect.max) / 2;',
  "return role ?? 'dps';",
]) {
  invariant(source.includes(needle), `source party-frame rule drifted: ${needle}`);
}
invariant(types.includes('export const PARTY_MEMBER_AURA_CAP = 8;'), 'source party-frame cap drifted');

for (const needle of [
  'pub partyMemberAuraCap(): int {',
  'return 8;',
  'pub visibleAuraRemaining(remaining: float): int {',
  'return <float>whole < remaining ? whole + 1 : whole;',
  'sortRelevantAuraIndexes(state: PartyFrameProjectionState): container.Array<int> {',
  'if (<int>state.auraPriorities[left] > <int>state.auraPriorities[right]) {',
  'addAura(capped, "rend", "dot", 5.0, 10.0, true, 0);',
  'addAura(capped, "temporal_echo", "temporal_echo", 0.0, 10.0, true, 1);',
  '<string>capped.summaryIds[7] != "hot_5"',
  '<string>capped.auraIds[8] != "rend"',
  'pub partyFrameAbsorb(state: PartyFrameProjectionState): float {',
  'pub projectPartyFrameAggroTargets(state: PartyFrameProjectionState): void {',
  'pub projectPartyFrameIncomingHeals(state: PartyFrameProjectionState): void {',
  'pub partyFrameRole(role: string, hasRole: bool): string {',
]) {
  invariant(projection.includes(needle), `WOC party-frame projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/party_frame_projection_state")',
  'frames.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC party-frame test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_party_frame_projection_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-party-frame-projection-state-tests' &&
    testProject.entry === 'social/party_frame_projection_state_test_main',
  'party-frame test project contract drifted',
);

process.stdout.write(`checked M6 party-frame projection source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
