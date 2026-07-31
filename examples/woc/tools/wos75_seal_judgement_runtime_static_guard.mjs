import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const sourceAbilities = readFileSync(
  resolve(root, '..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'content', 'classes.ts'),
  'utf8',
);
const catalog = JSON.parse(readFileSync(resolve(root, 'contracts', 'm4_abilities.json'), 'utf8'));
const world = readFileSync(resolve(root, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const main = readFileSync(resolve(root, 'scripts', 'woc_game', 'src', 'main.zr'), 'utf8');
const contract = readFileSync(resolve(root, 'contracts', 'world-state.md'), 'utf8');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(`${label}: missing ${JSON.stringify(expected)}`);
}

function entry(id) {
  const result = catalog.entries.find((candidate) => candidate.id === id);
  if (!result) throw new Error(`catalog entry missing: ${id}`);
  return result;
}

const seal = entry('seal_of_righteousness');
const judgement = entry('judgement');
if (seal.index !== 14 || judgement.index !== 15) throw new Error('M4 paladin indices drifted');
if (seal.definition.cost !== 25 || seal.definition.effects[0].type !== 'imbue') {
  throw new Error('Seal rank-one profile drifted');
}
if (seal.definition.effects[0].bonus !== 4 || seal.definition.effects[0].duration !== 30 ||
  seal.definition.effects[0].judgeMin !== 10 || seal.definition.effects[0].judgeMax !== 18) {
  throw new Error('Seal rank-one effect drifted');
}
if (judgement.definition.cost !== 30 || judgement.definition.cooldown !== 10 ||
  judgement.definition.range !== 10 || judgement.definition.effects[0].type !== 'judgement') {
  throw new Error('Judgement profile drifted');
}

requireText(sourceAbilities, 'seal_of_righteousness:', 'source Seal');
requireText(sourceAbilities, 'judgeMin: 10', 'source Seal judge min');
requireText(sourceAbilities, 'judgeMax: 18', 'source Seal judge max');
requireText(sourceAbilities, 'judgement:', 'source Judgement');

for (const expected of [
  'writer.u16(<uint>71, 1, 1);',
  'schemaVersion != <uint>61 && schemaVersion != <uint>62 &&',
  'pub var entityImbueAbilityCodes: container.Array<uint>;',
  'appendDefaultImbueColumns(this);',
  'imbueStateIsValid(state: WorldState): bool',
  'sealOfRighteousnessAbilityCode(): uint',
  'judgementAbilityCode(): uint',
  'startOfflineSealOfRighteousnessCast(state, casterIndex);',
  'startOfflineJudgementCast(',
  'actor.imbueBonus = offlineImbueBonus(state, playerIndex);',
  'numericEffects.dispatchNumericAbility(numeric, abilityIndex, rank, 0)',
  'clearOfflineImbue(state, casterIndex);',
  'ageOfflineImbues(state);',
  'pub sealJudgementCommandStateTest(): int',
]) requireText(world, expected, 'world reducer');

if ((main.match(/world_state[^\r\n]*WOS71/g) ?? []).length !== 2) {
  throw new Error('plugin state schema must publish WOS71 in both runtime paths');
}
requireText(contract, '# WOC authoritative world state (`WOS71`)', 'world-state contract');
requireText(contract, 'WOS75 adds schema 62', 'WOS75 contract delta');

process.stdout.write('WOS75 Seal/Judgement runtime static guard passed\n');
