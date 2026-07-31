import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const STATION_SOURCE_PATH = 'src/sim/professions/mobile_station.ts';
const PROFESSIONS_CONTENT_SOURCE_PATH = 'src/sim/content/professions.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'mobile_crafting_station_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'mobile_crafting_station_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const station = sourceBlob(STATION_SOURCE_PATH);
  const professions = sourceBlob(PROFESSIONS_CONTENT_SOURCE_PATH);
  const durationTicks = productConstant(
    professions,
    'MOBILE_CRAFTING_STATION_DURATION_TICKS',
  );
  for (const needle of [
    'if (!isSpecialized(crafterSkills, craftId)) return undefined;',
    'expiresAtTick: nowTick + MOBILE_CRAFTING_STATION_DURATION_TICKS,',
    'return nowTick < station.expiresAtTick;',
  ]) {
    invariant(station.includes(needle), 'mobile crafting station source drifted: ' + needle);
  }
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/mobile_crafting_station_contract_codegen.mjs',
    source_blobs: {
      [STATION_SOURCE_PATH]: sha256(station),
      [PROFESSIONS_CONTENT_SOURCE_PATH]: sha256(professions),
    },
    duration_ticks: durationTicks,
    semantics: {
      placement: 'specialization is a caller-provided gate; rejected placement returns no station',
      lifetime: 'expiresAtTick equals nowTick plus the content duration and is active only while nowTick is strictly less',
      scope: 'station storage and any future crafting-location gate remain outside this pure policy projection',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'mobile crafting station JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'mobile crafting station Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' mobile crafting station contract for ' + SOURCE_COMMIT + '\n');
}

function productConstant(source, name) {
  const match = source.match(new RegExp('export const ' + name + ' = ([0-9][0-9\\s*]*);'));
  invariant(match, 'professions content no longer exposes ' + name + ' as a positive product');
  const factors = match[1].split('*').map((factor) => factor.trim());
  invariant(factors.length >= 1 && factors.every((factor) => /^\d+$/.test(factor)),
    name + ' product has an unsupported factor');
  return factors.reduce((product, factor) => product * Number(factor), 1);
}

function renderZr(document) {
  return '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n' +
    'pub durationTicks(): int { return ' + document.duration_ticks + '; }\n';
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
