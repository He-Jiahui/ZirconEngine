import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CRAFTING_HUB_SOURCE_PATH = 'src/sim/professions/crafting_hub.ts';
const PROFESSIONS_CONTENT_SOURCE_PATH = 'src/sim/content/professions.ts';
const ZONE3_SOURCE_PATH = 'src/sim/content/zone3.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'crafting_hub_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'crafting_hub_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const craftingHub = sourceBlob(CRAFTING_HUB_SOURCE_PATH);
  const professions = sourceBlob(PROFESSIONS_CONTENT_SOURCE_PATH);
  const zone3 = sourceBlob(ZONE3_SOURCE_PATH);
  const hub = zoneHub(zone3);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/crafting_hub_contract_codegen.mjs',
    source_blobs: {
      [CRAFTING_HUB_SOURCE_PATH]: sha256(craftingHub),
      [PROFESSIONS_CONTENT_SOURCE_PATH]: sha256(professions),
      [ZONE3_SOURCE_PATH]: sha256(zone3),
    },
    zone_id: zoneId(zone3),
    position: { x: hub.x, z: hub.z },
    radius: hub.radius,
    min_level: integerConstant(professions, 'CRAFTING_HUB_MIN_LEVEL'),
    semantics: {
      location: 'inside the inclusive Highwatch hub circle in Thornpeak Heights',
      level: 'character level must be at least the configured crafting-hub minimum',
      access: 'both location and level gates must pass before a station-bound recipe is usable',
    },
  };
  for (const needle of [
    'return dx * dx + dz * dz <= CRAFTING_HUB_RADIUS * CRAFTING_HUB_RADIUS;',
    'return level >= CRAFTING_HUB_MIN_LEVEL;',
    'return isAtCraftingHub(pos) && meetsCraftingHubLevel(level);',
  ]) {
    invariant(craftingHub.includes(needle), 'crafting hub source drifted: ' + needle);
  }
  for (const needle of [
    'x: ZONE3_ZONE.hub.x,',
    'z: ZONE3_ZONE.hub.z,',
    'export const CRAFTING_HUB_RADIUS = ZONE3_ZONE.hub.radius;',
  ]) {
    invariant(professions.includes(needle), 'crafting hub content drifted: ' + needle);
  }
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'crafting hub JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'crafting hub Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' crafting hub contract for ' + SOURCE_COMMIT + '\n');
}

function zoneId(source) {
  const match = source.match(/id: '([^']+)',/);
  invariant(match, 'zone3 source no longer exposes its id');
  return match[1];
}

function zoneHub(source) {
  const match = source.match(/hub: \{ x: (-?\d+), z: (-?\d+), radius: (\d+),/);
  invariant(match, 'zone3 source no longer exposes a literal hub');
  return { x: Number(match[1]), z: Number(match[2]), radius: Number(match[3]) };
}

function integerConstant(source, name) {
  const match = source.match(new RegExp('export const ' + name + ' = (\\d+);'));
  invariant(match, 'profession content no longer exposes ' + name);
  return Number(match[1]);
}

function renderZr(document) {
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub hubX(): float { return ' + document.position.x + '.0; }',
    'pub hubZ(): float { return ' + document.position.z + '.0; }',
    'pub hubRadius(): float { return ' + document.radius + '.0; }',
    'pub minLevel(): int { return ' + document.min_level + '; }',
  ];
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
