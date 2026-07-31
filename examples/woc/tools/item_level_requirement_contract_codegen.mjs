import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/item_level_req.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'item_level_requirement_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'item_level_requirement_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const types = sourceBlob(TYPES_PATH);
  const qualityFallbacks = {
    poor: 1,
    common: 1,
    uncommon: 1,
    rare: 12,
    epic: 18,
  };
  const maxLevel = literal(types, /export const MAX_LEVEL = (\d+);/, 'MAX_LEVEL');
  for (const [quality, level] of Object.entries(qualityFallbacks)) {
    invariant(source.includes('  ' + quality + ': ' + level + ','), 'item-level fallback drifted: ' + quality);
  }
  for (const needle of [
    'legendary: MAX_LEVEL,',
    "const GATED_QUALITIES = new Set<Quality>(['rare', 'epic', 'legendary']);",
    'if (Number.isFinite(item.requiredLevel)) {',
    'return clampLevel(item.requiredLevel as number);',
    "const quality = item.quality ?? 'common';",
    'if (!GATED_QUALITIES.has(quality)) return 1;',
    'const source = itemSourceLevel(item.id);',
    'return clampLevel(source ?? QUALITY_REQUIRED_LEVEL[quality]);',
    'return Math.max(1, Math.min(MAX_LEVEL, Math.floor(raw)));',
    'return level >= requiredLevelFor(item);',
  ]) {
    invariant(source.includes(needle), 'item-level requirement source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/item_level_requirement_contract_codegen.mjs',
    source_blobs: {
      [SOURCE_PATH]: sha256(source),
      [TYPES_PATH]: sha256(types),
    },
    max_level: maxLevel,
    quality_fallback_levels: {
      ...qualityFallbacks,
      legendary: maxLevel,
    },
    gated_qualities: ['rare', 'epic', 'legendary'],
    semantics: {
      explicit_required_level: 'finite explicit requiredLevel wins and is floored then clamped',
      missing_quality: 'common',
      ungated_qualities: ['poor', 'common', 'uncommon'],
      source_level: 'only gated qualities use itemSourceLevel when present',
      clamp: '[1, MAX_LEVEL]',
      input_boundary: 'Zr callers provide typed integers and set explicitPresent only after source Number.isFinite normalization',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'item-level requirement JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'item-level requirement Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' item-level requirement contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  lines.push('pub maxLevel(): int { return ' + document.max_level + '; }\n');
  for (const [quality, level] of Object.entries(document.quality_fallback_levels)) {
    lines.push('pub ' + quality + 'FallbackLevel(): int { return ' + level + '; }\n');
  }
  return lines.join('');
}

function literal(source, expression, label) {
  const match = source.match(expression);
  invariant(match, 'item-level requirement source no longer exposes ' + label);
  return Number(match[1]);
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
