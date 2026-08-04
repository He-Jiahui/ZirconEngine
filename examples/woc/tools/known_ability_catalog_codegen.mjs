import { spawnSync, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CLASSES_PATH = 'src/sim/content/classes.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractorPath = join(scriptDirectory, 'known_ability_catalog_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'known_ability_catalog.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'current_known_ability_catalog.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const sourceBlob = execFileSync(
    'git',
    ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${CLASSES_PATH}`],
    { encoding: 'buffer' },
  );
  const source = sourceBlob.toString('utf8');
  for (const statement of [
    'export const ABILITIES:',
    'export const CLASSES:',
    'export function abilitiesKnownAt(',
    'const baseIds = CLASSES[cls].abilities;',
  ]) invariant(source.includes(statement), `current ability catalog source drifted: ${statement}`);

  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
    `wocgit:///${CLASSES_PATH}`,
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `ability catalog extractor exited ${child.status}`);
  const extracted = JSON.parse(child.stdout);
  invariant(Array.isArray(extracted.classes) && extracted.classes.length === 9,
    'current target must expose nine player classes');
  invariant(Array.isArray(extracted.abilities) && extracted.abilities.length > 0,
    'current target ability catalog is empty');

  const abilityIds = new Set();
  for (const ability of extracted.abilities) {
    invariant(typeof ability.id === 'string' && ability.id.length > 0, 'ability id is invalid');
    invariant(!abilityIds.has(ability.id), `duplicate ability id ${ability.id}`);
    abilityIds.add(ability.id);
    invariant(Number.isInteger(ability.learn_level) && ability.learn_level >= 0,
      `invalid learn level for ${ability.id}`);
    invariant(typeof ability.class_id === 'string' && ability.class_id.length > 0,
      `invalid class for ${ability.id}`);
    invariant(Number.isInteger(ability.base_cost) && ability.base_cost >= 0,
      `invalid base cost for ${ability.id}`);
    invariant(Number.isFinite(ability.base_cast_time) && ability.base_cast_time >= 0,
      `invalid base cast time for ${ability.id}`);
    invariant(Number.isFinite(ability.base_cooldown) && ability.base_cooldown >= 0,
      `invalid base cooldown for ${ability.id}`);
    invariant(typeof ability.school === 'string' && ability.school.length > 0,
      `invalid school for ${ability.id}`);
    invariant(typeof ability.exclusive_group === 'string' &&
      typeof ability.requires_form === 'string' &&
      typeof ability.requires_stealth === 'boolean' &&
      typeof ability.usable_in_form === 'boolean' &&
      typeof ability.cast_while_moving === 'boolean' &&
      typeof ability.primary_self_buff_kind === 'string' &&
      Number.isFinite(ability.primary_self_buff_value),
    `invalid action-bar metadata for ${ability.id}`);
    invariant(Array.isArray(ability.ranks), `invalid ranks for ${ability.id}`);
    for (const rank of ability.ranks) {
      invariant(Number.isInteger(rank.rank) && rank.rank >= 1 &&
        Number.isInteger(rank.level) && rank.level >= 0,
      `invalid rank for ${ability.id}`);
    }
  }
  const classIds = new Set();
  for (const playerClass of extracted.classes) {
    invariant(typeof playerClass.id === 'string' && playerClass.id.length > 0,
      'class id is invalid');
    invariant(!classIds.has(playerClass.id), `duplicate class id ${playerClass.id}`);
    classIds.add(playerClass.id);
    invariant(Array.isArray(playerClass.abilities), `class abilities are invalid for ${playerClass.id}`);
    for (const id of playerClass.abilities) {
      invariant(abilityIds.has(id), `${playerClass.id} references missing ability ${id}`);
    }
  }

  const codeById = new Map(extracted.abilities.map((ability, index) => [ability.id, index + 1]));
  const classes = extracted.classes.map((playerClass) => ({
    ...playerClass,
    ability_codes: playerClass.abilities.map((id) => codeById.get(id)),
  }));
  const catalogSha = hashText(JSON.stringify({ abilities: extracted.abilities, classes }));
  const document = {
    schema_version: 4,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/known_ability_catalog_codegen.mjs',
    source_blobs: { [CLASSES_PATH]: createHash('sha256').update(sourceBlob).digest('hex') },
    catalog_sha256: catalogSha,
    abilities: extracted.abilities.map((ability, index) => ({ code: index + 1, ...ability })),
    classes,
  };
  const json = `${JSON.stringify(document, null, 2)}\n`;
  const zr = renderZr(document);
  for (const [path, output, label] of [
    [jsonOutput, json, 'known-ability catalog JSON'],
    [zrOutput, zr, 'known-ability catalog Zr'],
  ]) {
    if (checkOnly) {
      invariant(existsSync(path), `${label} is missing; run npm run generate:known-ability-catalog`);
      invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:known-ability-catalog`);
    } else {
      writeFileSync(path, output, 'utf8');
    }
  }
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} ${document.abilities.length} current known abilities ` +
      `across ${document.classes.length} classes (${catalogSha.slice(0, 15)})\n`,
  );
}

function renderZr(document) {
  const abilityCodeRows = document.abilities
    .map((ability) => `    if (id == ${JSON.stringify(ability.id)}) { return ${ability.code}; }`)
    .join('\n');
  const abilityIdRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${JSON.stringify(ability.id)}; }`)
    .join('\n');
  const learnRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${ability.learn_level}; }`)
    .join('\n');
  const passiveRows = document.abilities
    .filter((ability) => ability.passive)
    .map((ability) => `    if (code == ${ability.code}) { return true; }`)
    .join('\n');
  const abilityClassRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${JSON.stringify(ability.class_id)}; }`)
    .join('\n');
  const baseCostRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${ability.base_cost}; }`)
    .join('\n');
  const baseCastTimeRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${floatLiteral(ability.base_cast_time)}; }`)
    .join('\n');
  const baseCooldownRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${floatLiteral(ability.base_cooldown)}; }`)
    .join('\n');
  const schoolRows = document.abilities
    .map((ability) => `    if (code == ${ability.code}) { return ${JSON.stringify(ability.school)}; }`)
    .join('\n');
  const primarySelfBuffKindRows = document.abilities
    .filter((ability) => ability.primary_self_buff_kind.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${JSON.stringify(ability.primary_self_buff_kind)}; }`)
    .join('\n');
  const primarySelfBuffValueRows = document.abilities
    .filter((ability) => ability.primary_self_buff_kind.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${floatLiteral(ability.primary_self_buff_value)}; }`)
    .join('\n');
  const exclusiveGroupRows = document.abilities
    .filter((ability) => ability.exclusive_group.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${JSON.stringify(ability.exclusive_group)}; }`)
    .join('\n');
  const requiredFormRows = document.abilities
    .filter((ability) => ability.requires_form.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${JSON.stringify(ability.requires_form)}; }`)
    .join('\n');
  const requiredStealthRows = document.abilities
    .filter((ability) => ability.requires_stealth)
    .map((ability) => `    if (code == ${ability.code}) { return true; }`)
    .join('\n');
  const usableInFormRows = document.abilities
    .filter((ability) => ability.usable_in_form)
    .map((ability) => `    if (code == ${ability.code}) { return true; }`)
    .join('\n');
  const castWhileMovingRows = document.abilities
    .filter((ability) => ability.cast_while_moving)
    .map((ability) => `    if (code == ${ability.code}) { return true; }`)
    .join('\n');
  const rankRows = document.abilities.map((ability) => {
    const updates = ability.ranks
      .map((rank) => `        if (level >= ${rank.level}) { resolved = ${rank.rank}; }`)
      .join('\n');
    return `    if (code == ${ability.code}) {\n        var resolved = 1;${updates.length > 0 ? `\n${updates}` : ''}\n        return resolved;\n    }`;
  }).join('\n');
  const requiredRows = document.abilities
    .filter((ability) => ability.specs.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return true; }`)
    .join('\n');
  const requiredMatchRows = document.abilities
    .filter((ability) => ability.specs.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${stringSetMatch(ability.specs, 'spec')}; }`)
    .join('\n');
  const excludedRows = document.abilities
    .filter((ability) => ability.exclude_specs.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return true; }`)
    .join('\n');
  const excludedMatchRows = document.abilities
    .filter((ability) => ability.exclude_specs.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${stringSetMatch(ability.exclude_specs, 'spec')}; }`)
    .join('\n');
  const exclusionLevelRows = document.abilities
    .filter((ability) => ability.exclude_specs.length > 0)
    .map((ability) => `    if (code == ${ability.code}) { return ${ability.exclude_specs_at_level}; }`)
    .join('\n');
  const classIdRows = document.classes
    .map((playerClass, index) => `    if (index == ${index}) { return ${JSON.stringify(playerClass.id)}; }`)
    .join('\n');
  const classIndexRows = document.classes
    .map((playerClass, index) => `    if (id == ${JSON.stringify(playerClass.id)}) { return ${index}; }`)
    .join('\n');
  const classCountRows = document.classes
    .map((playerClass, index) => `    if (classIndex == ${index}) { return ${playerClass.ability_codes.length}; }`)
    .join('\n');
  const classCodeRows = document.classes.map((playerClass, index) => {
    const rows = playerClass.ability_codes
      .map((code, slot) => `        if (slot == ${slot}) { return <uint>${code}; }`)
      .join('\n');
    return `    if (classIndex == ${index}) {\n${rows}\n        throw "WOC class ability slot is out of range";\n    }`;
  }).join('\n');
  return `// Generated by examples/woc/tools/known_ability_catalog_codegen.mjs. Do not edit.\n` +
    `pub catalogSha(): string { return ${JSON.stringify(document.catalog_sha256)}; }\n` +
    `pub abilityCount(): int { return ${document.abilities.length}; }\n` +
    `pub classCount(): int { return ${document.classes.length}; }\n\n` +
    'pub abilityCode(id: string): int {\n' + abilityCodeRows + '\n    return 0;\n}\n\n' +
    'pub abilityId(code: int): string {\n' + abilityIdRows + '\n    return "";\n}\n\n' +
    `pub abilityExists(code: int): bool { return code > 0 && code <= ${document.abilities.length}; }\n\n` +
    'pub learnLevel(code: int): int {\n    if (!abilityExists(code)) { return 0; }\n' + learnRows + '\n    return 0;\n}\n\n' +
    'pub isPassive(code: int): bool {\n' + passiveRows + '\n    return false;\n}\n\n' +
    'pub abilityClassId(code: int): string {\n    if (!abilityExists(code)) { return ""; }\n' +
    abilityClassRows + '\n    return "";\n}\n\n' +
    'pub baseCost(code: int): int {\n    if (!abilityExists(code)) { return 0; }\n' +
    baseCostRows + '\n    return 0;\n}\n\n' +
    'pub baseCastTime(code: int): float {\n    if (!abilityExists(code)) { return 0.0; }\n' +
    baseCastTimeRows + '\n    return 0.0;\n}\n\n' +
    'pub baseCooldown(code: int): float {\n    if (!abilityExists(code)) { return 0.0; }\n' +
    baseCooldownRows + '\n    return 0.0;\n}\n\n' +
    'pub abilitySchool(code: int): string {\n    if (!abilityExists(code)) { return ""; }\n' +
    schoolRows + '\n    return "";\n}\n\n' +
    'pub primarySelfBuffKind(code: int): string {\n    if (!abilityExists(code)) { return ""; }\n' +
    primarySelfBuffKindRows + '\n    return "";\n}\n\n' +
    'pub primarySelfBuffValue(code: int): float {\n    if (!abilityExists(code)) { return 0.0; }\n' +
    primarySelfBuffValueRows + '\n    return 0.0;\n}\n\n' +
    'pub exclusiveGroup(code: int): string {\n' + exclusiveGroupRows + '\n    return "";\n}\n\n' +
    'pub requiresForm(code: int): string {\n' + requiredFormRows + '\n    return "";\n}\n\n' +
    'pub requiresStealth(code: int): bool {\n' + requiredStealthRows + '\n    return false;\n}\n\n' +
    'pub usableInForm(code: int): bool {\n' + usableInFormRows + '\n    return false;\n}\n\n' +
    'pub baseCastWhileMoving(code: int): bool {\n' + castWhileMovingRows + '\n    return false;\n}\n\n' +
    'pub rankAt(code: int, level: int): int {\n    if (!abilityExists(code)) { return 0; }\n' +
    rankRows + '\n    return 1;\n}\n\n' +
    'pub hasRequiredSpecs(code: int): bool {\n' + requiredRows + '\n    return false;\n}\n\n' +
    'pub matchesRequiredSpec(code: int, spec: string): bool {\n' + requiredMatchRows + '\n    return false;\n}\n\n' +
    'pub hasExcludedSpecs(code: int): bool {\n' + excludedRows + '\n    return false;\n}\n\n' +
    'pub matchesExcludedSpec(code: int, spec: string): bool {\n' + excludedMatchRows + '\n    return false;\n}\n\n' +
    'pub excludeAtLevel(code: int): int {\n' + exclusionLevelRows + '\n    return 0;\n}\n\n' +
    'pub isKnownAt(code: int, level: int, granted: bool, committedSpec: string): bool {\n' +
    '    if (!abilityExists(code)) { return false; }\n' +
    '    if (granted) { return true; }\n' +
    '    if (learnLevel(code) > level) { return false; }\n' +
    '    if (hasRequiredSpecs(code) && !matchesRequiredSpec(code, committedSpec)) { return false; }\n' +
    '    if (hasExcludedSpecs(code) && matchesExcludedSpec(code, committedSpec) &&\n' +
    '        level >= excludeAtLevel(code)) { return false; }\n' +
    '    return true;\n}\n\n' +
    'pub classId(index: int): string {\n' + classIdRows + '\n    return "";\n}\n\n' +
    'pub classIndex(id: string): int {\n' + classIndexRows + '\n    return -1;\n}\n\n' +
    'pub classAbilityCount(classIndex: int): int {\n' + classCountRows + '\n    return 0;\n}\n\n' +
    'pub classAbilityCode(classIndex: int, slot: int): uint {\n' + classCodeRows +
    '\n    throw "WOC class ability index is out of range";\n}\n';
}

function stringSetMatch(values, expression) {
  return values.map((value) => `${expression} == ${JSON.stringify(value)}`).join(' || ');
}

function floatLiteral(value) {
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function hashText(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
