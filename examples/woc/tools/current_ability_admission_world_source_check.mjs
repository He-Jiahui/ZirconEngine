import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const source = readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'world', 'state.zr'),
  'utf8',
);

for (const needle of [
  'var knownAbilities = %import("combat/known_ability_state");',
  'temporalReversalCatalogAdmission(state: WorldState, casterIndex: int, abilityCode: uint): bool',
  'var classIndex = <int><uint>state.entityTemplates[casterIndex] - 1;',
  'var specCode = <uint>state.entityTalentSpecCodes[casterIndex];',
  'var specId = specCode == <uint>0 ? "" : talentSelectionCatalog.specId(specCode);',
  'knownAbilities.sourceAbilityAdmission(',
  'if (!temporalReversalCatalogAdmission(state, casterIndex, abilityCode) ||',
  'var arcaneCode = talentSelectionCatalog.specCode("mage", "arcane");',
  'state.entityTalentSpecCodes[0] = arcaneCode;',
  'state.entityTalentSpecCodes[1] = arcaneCode;',
  'if (temporalReversalCatalogAdmission(state, 0, code)) { return -7; }',
  'if (!temporalReversalCatalogAdmission(state, 0, code)) { return -8; }',
]) {
  invariant(source.includes(needle), `world ability admission omitted: ${needle}`);
}

process.stdout.write('checked current ability admission world integration source projection\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
