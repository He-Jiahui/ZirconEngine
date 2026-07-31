import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const source = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_ai.ts');
const rules = read('scripts', 'woc_game', 'src', 'instances', 'pet_target_rules.zr');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  'export function petPickTarget',
  "if (pet.petMode === 'passive') return null;",
  'const ownerIdle = !ownerMeta || ctx.tickCount - ownerMeta.lastActiveTick > PET_OWNER_IDLE_TICKS;',
  'const engagingUs =',
  'const ownerOffense =',
  "pet.petMode === 'aggressive'",
  'if (d < bestD)',
]) requireText(source, expected, 'source pet target selection');

for (const expected of [
  'pub canSelectTarget(passiveMode: bool): bool',
  'pub candidateEligible(',
  'pub shouldReplaceCandidate(candidateDistance: float, bestDistance: float): bool',
  'pub contractTest(): int',
]) requireText(rules, expected, 'source-locked target rules');

for (const expected of [
  'var petTarget = %import("instances/pet_target_rules");',
  'offlineEmberkinAssistTargetIndex(',
  'var passive = <uint>state.entityPetModes[petIndex] == <uint>1;',
  'var aggressive = <uint>state.entityPetModes[petIndex] == <uint>3;',
  'petTarget.candidateEligible(',
  'petTarget.shouldReplaceCandidate(distance, bestDistance)',
  'targetIndex = offlineEmberkinAssistTargetIndex(state, petIndex, ownerIndex);',
  'pub emberkinAssistTargetStateTest(): int',
]) requireText(world, expected, 'WOS78 reducer');

requireText(contract, 'WOS78 attaches the existing source-locked pet target predicates', 'WOS78 contract');
requireText(contract, 'activity timestamp is not in the current WOS envelope', 'WOS78 contract');
requireText(contract, 'pulls intentionally remain disabled', 'WOS78 contract');

process.stdout.write('WOS78 Emberkin pet target runtime static guard passed\n');
