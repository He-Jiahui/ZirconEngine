import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/mob/social_aggro.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'social_aggro_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'social_aggro_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const radius = numericConstant(source, 'FLEE_HELP_RADIUS');
  for (const needle of [
    "m.kind === 'mob'",
    "m.aiState === 'idle'",
    'm.ownerId === null',
    'd2 < FLEE_HELP_RADIUS * FLEE_HELP_RADIUS',
    "m.aiState = 'chase';",
    'm.aggroTargetId = target.id;',
    'm.leashAnchor = { ...m.pos };',
    'addThreat(m, target.id, 1);',
  ]) {
    invariant(source.includes(needle), 'social aggro source drifted: ' + needle);
  }
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/social_aggro_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    flee_help_radius: radius,
    semantics: {
      candidate: 'same-family, living, hostile, idle and ownerless mobs only',
      distance: 'candidate distance is strictly less than the squared flee-help radius',
      mutation: 'qualifying allies chase the target, enter combat, copy the fleeing position as leash anchor and gain one threat',
      scope: 'the caller owns flee termination after a non-zero rally and spatial-grid discovery',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'social aggro JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'social aggro Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' social aggro contract for ' + SOURCE_COMMIT + '\n');
}

function numericConstant(source, name) {
  const match = source.match(new RegExp('export const ' + name + ' = (\\d+);'));
  invariant(match, 'social aggro source no longer exposes ' + name);
  return Number(match[1]);
}

function renderZr(document) {
  return '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n' +
    'pub fleeHelpRadius(): float { return ' + document.flee_help_radius + '.0; }\n';
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
