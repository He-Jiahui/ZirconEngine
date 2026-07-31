import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const HAND_PATH = 'src/sim/minigames/card_hand.ts';
const MATCH_PATH = 'src/sim/social/card_duel.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'card_duel_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'card_duel_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = JSON.parse(readFileSync(join(referenceRoot, 'source_manifest.json'), 'utf8'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before Card Duel contracts');
  const handBytes = sourceBlob(HAND_PATH);
  const matchBytes = sourceBlob(MATCH_PATH);
  const deckSize = exportedNumber(handBytes.toString('utf8'), 'DECK_SIZE');
  const startingHandSize = exportedNumber(handBytes.toString('utf8'), 'STARTING_HAND_SIZE');
  const roundsToWin = exportedNumber(matchBytes.toString('utf8'), 'CARD_DUEL_ROUNDS_TO_WIN');
  const deadlineSeconds = exportedNumber(matchBytes.toString('utf8'), 'CARD_DUEL_ROUND_DEADLINE_S');
  invariant(deckSize > 0 && deckSize % 2 === 0, 'Card Duel deck size must be a positive even number');
  invariant(startingHandSize > 0 && startingHandSize < deckSize, 'Card Duel starting hand size is invalid');
  invariant(roundsToWin > 0 && deadlineSeconds > 0, 'Card Duel match constants are invalid');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/card_duel_contract_codegen.mjs',
    source_blobs: {
      [HAND_PATH]: sha256(handBytes),
      [MATCH_PATH]: sha256(matchBytes),
    },
    constants: {
      deck_size: deckSize,
      card_max_value: deckSize / 2,
      starting_hand_size: startingHandSize,
      rounds_to_win: roundsToWin,
      round_deadline_seconds: deadlineSeconds,
    },
  };
  const json = render(document);
  const zr = renderZr(document);
  writeOrCheck(jsonOutput, json, 'Card Duel JSON contract');
  writeOrCheck(zrOutput, zr, 'Card Duel Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Card Duel contract for ${SOURCE_COMMIT}\n`);
}

function exportedNumber(source, name) {
  const match = new RegExp(`export\\s+const\\s+${name}\\s*=\\s*(\\d+(?:\\.\\d+)?)\\s*;`).exec(source);
  invariant(match, `missing numeric export ${name}`);
  return Number(match[1]);
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer', maxBuffer: 16 * 1024 * 1024,
  });
}

function renderZr(document) {
  const constants = document.constants;
  const deadline = Number.isInteger(constants.round_deadline_seconds)
    ? `${constants.round_deadline_seconds}.0`
    : String(constants.round_deadline_seconds);
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub deckSize(required: bool): int { return required ? ${constants.deck_size} : 0; }\n` +
    `pub cardMaxValue(required: bool): int { return required ? ${constants.card_max_value} : 0; }\n` +
    `pub startingHandSize(required: bool): int { return required ? ${constants.starting_hand_size} : 0; }\n` +
    `pub roundsToWin(required: bool): int { return required ? ${constants.rounds_to_win} : 0; }\n` +
    `pub roundDeadlineSeconds(required: bool): float { return required ? ${deadline} : 0.0; }\n`;
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:card-duel-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:card-duel-contract`);
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function render(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
