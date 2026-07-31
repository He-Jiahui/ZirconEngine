import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const BATTLEFIELD_XP_SOURCE_PATH = 'src/sim/professions/battlefield_xp.ts';
const GATHERING_SOURCE_PATH = 'src/sim/professions/gathering.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'battlefield_xp_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'battlefield_xp_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const battlefieldXp = sourceBlob(BATTLEFIELD_XP_SOURCE_PATH);
  const gathering = sourceBlob(GATHERING_SOURCE_PATH);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/battlefield_xp_contract_codegen.mjs',
    source_blobs: {
      [BATTLEFIELD_XP_SOURCE_PATH]: sha256(battlefieldXp),
      [GATHERING_SOURCE_PATH]: sha256(gathering),
    },
    trickle: numberConstant(battlefieldXp, 'BATTLEFIELD_XP_TRICKLE'),
    signable_qualities: signableQualities(gathering),
    semantics: {
      observation_scope: 'self-observation only: the instance signer must equal the observer',
      rarity: 'prefer an instance rolled quality when present, otherwise use the resolved item-definition quality; only signable rare-or-better qualities continue',
      profession: 'the caller resolves recipeForResultItem and the item definition; the returned profession must be either active major',
      mutation: 'a successful observation adds the fixed trickle to the recipe profession skill and no rejection mutates skill',
    },
  };
  for (const needle of [
    'instance?.rolled?.quality ?? ITEMS[itemId]?.quality',
    'if (!rarity || !isSignableMaterialRarity(rarity)) return 0;',
    'if (!instance?.signer || instance.signer !== observerName) return 0;',
    'const recipe = recipeForResultItem(itemId);',
    'recipe.professionId === observerActiveArchetype || recipe.professionId === observerPairedMajor',
    'gainCraftSkill(craftSkills, recipe.professionId, BATTLEFIELD_XP_TRICKLE);',
  ]) {
    invariant(battlefieldXp.includes(needle), 'battlefield XP source drifted: ' + needle);
  }
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'battlefield XP JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'battlefield XP Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' battlefield XP contract for ' + SOURCE_COMMIT + '\n');
}

function numberConstant(source, name) {
  const match = source.match(new RegExp('export const ' + name + ' = (\\d+(?:\\.\\d+)?);'));
  invariant(match, 'battlefield XP source no longer exposes ' + name);
  return Number(match[1]);
}

function signableQualities(source) {
  const start = source.indexOf('export function isSignableMaterialRarity');
  invariant(start >= 0, 'gathering source no longer exposes isSignableMaterialRarity');
  const end = source.indexOf('\n}', start);
  invariant(end >= 0, 'isSignableMaterialRarity source is unterminated');
  const body = source.slice(start, end);
  const qualities = [...body.matchAll(/rarity === '([^']+)'/g)].map((match) => match[1]);
  invariant(qualities.join(',') === 'rare,epic,legendary', 'unexpected signable quality set');
  return qualities;
}

function renderZr(document) {
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub battlefieldXpTrickle(): float { return ' + document.trickle + '; }',
    'pub isSignableMaterialRarity(quality: string): bool {',
  ];
  document.signable_qualities.forEach((quality) => {
    lines.push('    if (quality == "' + quality + '") return true;');
  });
  lines.push('    return false;');
  lines.push('}');
  return lines.join('\n') + '\n';
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), label + ' is missing; run its generate script');
    invariant(readFileSync(path, 'utf8') === output, label + ' is stale; run its generate script');
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
