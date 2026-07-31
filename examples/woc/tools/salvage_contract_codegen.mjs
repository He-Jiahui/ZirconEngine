import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/professions/salvage.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'salvage_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'salvage_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const qualities = orderedStrings(source, 'QUALITY_ORDER');
  const materials = stringRecord(source, 'SALVAGE_MATERIAL_BY_QUALITY');
  for (const needle of [
    "(def.kind === 'weapon' || def.kind === 'armor')",
    "def.quality !== 'poor'",
    'const tierBonus = Math.floor(requiredLevelFor(def) / 10);',
    'const bonus = rng.next() < 0.5 ? 0 : 1;',
    'removePreferFungible(ctx, itemId, 1, pid);',
    'ctx.addItem(materialItemId, count, pid);',
  ]) {
    invariant(source.includes(needle), 'salvage source drifted: ' + needle);
  }
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/salvage_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    quality_order: qualities,
    material_by_quality: materials,
    material_fallback: 'bone_fragments',
    tier_levels_per_bonus: 10,
    roll_threshold: 0.5,
    semantics: {
      eligibility: 'only weapon or armor definitions with a non-poor quality are salvageable',
      yield: 'quality index plus floor(required level / 10) plus one and one bonus when RNG is at least 0.5',
      mutation: 'unknown, ineligible and missing-held-item outcomes consume no item or RNG; success removes one fungible item before granting material',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'salvage JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'salvage Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' salvage contract for ' + SOURCE_COMMIT + '\n');
}

function orderedStrings(source, name) {
  const expression = new RegExp('const ' + name + '[\\s\\S]*?= \\[([\\s\\S]*?)\\];');
  const match = source.match(expression);
  invariant(match, 'salvage source no longer exposes ' + name);
  const values = [...match[1].matchAll(/'([^']+)'/g)].map((entry) => entry[1]);
  invariant(values.length === 6, 'unexpected salvage quality-order length');
  return values;
}

function stringRecord(source, name) {
  const expression = new RegExp('const ' + name + '[\\s\\S]*?= \\{([\\s\\S]*?)\\};');
  const match = source.match(expression);
  invariant(match, 'salvage source no longer exposes ' + name);
  const result = Object.fromEntries([...match[1].matchAll(/(\w+): '([^']+)'/g)]
    .map((entry) => [entry[1], entry[2]]));
  invariant(Object.keys(result).length === 5, 'unexpected salvage material map');
  return result;
}

function renderZr(document) {
  const lines = [
    '// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.',
    'pub qualityCount(): int { return ' + document.quality_order.length + '; }',
    'pub qualityAt(index: int): string {',
  ];
  document.quality_order.forEach((quality, index) => {
    lines.push('    if (index == ' + index + ') return "' + quality + '";');
  });
  lines.push('    return "";');
  lines.push('}');
  lines.push('pub materialForQuality(quality: string): string {');
  Object.entries(document.material_by_quality).forEach(([quality, material]) => {
    lines.push('    if (quality == "' + quality + '") return "' + material + '";');
  });
  lines.push('    return "' + document.material_fallback + '";');
  lines.push('}');
  lines.push('pub tierLevelsPerBonus(): int { return ' + document.tier_levels_per_bonus + '; }');
  lines.push('pub rollThreshold(): float { return ' + document.roll_threshold + '; }');
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
