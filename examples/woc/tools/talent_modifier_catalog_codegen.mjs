import { spawnSync, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATHS = [
  'src/sim/content/talents.ts', 'src/sim/content/talent_rows.ts',
  'src/sim/content/talents_warrior.ts', 'src/sim/content/talents_classic.ts',
  'src/sim/content/warrior_rows.ts', 'src/sim/content/choice_rows_classic.ts',
];
const STAT_FIELDS = [
  'str', 'agi', 'sta', 'int', 'spi', 'armor', 'ap', 'crit', 'dodge', 'apPct',
  'staPct', 'armorPct', 'armorFromStrPct', 'maxHpPct', 'strPct', 'agiPct', 'intPct', 'spiPct',
];
const GLOBAL_FIELDS = [
  'meleeDmgPct', 'spellDmgPct', 'healPct', 'manaPct', 'manaRegenPct', 'dotDmgPct',
  'hotHealPct', 'absorbPct', 'meleeHastePct', 'petDmgPct', 'petDmgSharePct', 'threatPct',
  'critDmgSpellPct', 'critDmgPhysPct', 'critDmgHealPct', 'spellHastePct', 'critVsRooted',
  'extraAttackPct', 'moonwingPartyCritPct', 'autoRagePct', 'abilityRagePct', 'onKillSpeedPct',
  'onKillSpeedDuration', 'secondWindPctPerSec', 'battleRhythm', 'bloodbathPct',
  'bloodbathDuration', 'bloodbathMaxPct', 'cdrPerRage', 'stanceMastery', 'fearBreakPct',
  'masteryTwoHandDmgPct', 'cheatDeathIcd', 'barrierDrPct', 'temporalRift', 'manaDefCdrPer10',
  'blinkCast', 'convergence', 'ignitionPct',
];
const ABILITY_NUMBER_FIELDS = [
  'dmgPct', 'dmgPctVsDotted', 'flatDmg', 'costPct', 'cooldownPct', 'cooldownFlat',
  'castPct', 'buffPct', 'critPct', 'bonusCharges',
];
const NESTED_EFFECT_CODES = new Map([
  ['root', 1], ['slow', 2], ['aoeRoot', 3], ['absorb', 4], ['dot', 5],
  ['extendDot', 6], ['interrupt', 7], ['consumeDot', 8],
]);
const NESTED_NUMBER_FIELDS = [
  'amount', 'duration', 'mult', 'radius', 'min', 'max', 'total', 'directPct',
  'interval', 'leechPct', 'seconds', 'maxBonus', 'lockout',
];
const NESTED_STRING_FIELDS = ['school', 'dot'];
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(root, '..', '..', 'dev', 'world-of-claudecraft');
const extractor = join(dirname(fileURLToPath(import.meta.url)), 'talent_modifier_catalog_source_extract.mjs');
const loaderUrl = pathToFileURL(join(dirname(fileURLToPath(import.meta.url)), 'typescript_git_loader.mjs')).href;
const selectionPath = join(root, 'reference', 'current-head', 'talent_selection_catalog.json');
const jsonPath = join(root, 'reference', 'current-head', 'talent_modifier_catalog.json');
const zrPath = join(root, 'scripts', 'woc_game', 'src', 'generated', 'talent_modifier_catalog.zr');
const check = process.argv.includes('--check');

const sourceBlobs = Object.fromEntries(SOURCE_PATHS.map((path) => [path, blob(path)]));
const selection = JSON.parse(readFileSync(selectionPath, 'utf8'));
if (selection.source_commit !== COMMIT) throw new Error('selection catalog is not current-head pinned');
const extracted = extract();
if (!Array.isArray(extracted.entries)) throw new Error('modifier extractor did not return entries');
const specCodes = new Map(selection.classes.flatMap((c) => c.specs.map((spec) => [`${c.id}:${spec.id}`, spec.code])));
const optionCodes = new Map(selection.classes.flatMap((c) => c.rows.flatMap((row) =>
  row.options.map((option) => [`${c.id}:${row.level}:${option.id}`, option.code]))));
const entries = extracted.entries.map((entry, index) => normalize(entry, index, specCodes, optionCodes));
if (entries.length !== 189) throw new Error(`expected 189 current modifier entries, got ${entries.length}`);
if (entries.filter((entry) => entry.origin === 'spec').length !== 27 ||
    entries.filter((entry) => entry.origin === 'option').length !== 162) {
  throw new Error('current modifier origin counts drifted');
}
const document = {
  schema_version: 1,
  source_commit: COMMIT,
  generated_by: 'examples/woc/tools/talent_modifier_catalog_codegen.mjs',
  source_blobs: Object.fromEntries(Object.entries(sourceBlobs).map(([path, value]) => [path, sha256(value)])),
  talent_selection_catalog_sha256: selection.catalog_sha256,
  catalog_sha256: sha256(Buffer.from(JSON.stringify(entries))),
  stat_fields: STAT_FIELDS,
  global_fields: GLOBAL_FIELDS,
  ability_number_fields: ABILITY_NUMBER_FIELDS,
  entries,
};
writeOrCheck(jsonPath, `${JSON.stringify(document, null, 2)}\n`, 'modifier JSON catalog');
writeOrCheck(zrPath, renderZr(document), 'modifier Zr catalog');
console.log(`${check ? 'checked' : 'generated'} ${entries.length} current talent modifier entries (${document.catalog_sha256.slice(0, 15)})`);

function normalize(raw, index, specCodes, optionCodes) {
  if (!raw || typeof raw !== 'object' || (raw.origin !== 'spec' && raw.origin !== 'option')) {
    throw new Error(`invalid modifier origin at ${index}`);
  }
  const key = raw.origin === 'spec' ? `${raw.class_id}:${raw.id}` : `${raw.class_id}:${raw.level}:${raw.id}`;
  const code = raw.origin === 'spec' ? specCodes.get(key) : optionCodes.get(key);
  if (!Number.isInteger(code) || code <= 0) throw new Error(`missing selection code for ${key}`);
  const effect = raw.effect;
  if (!effect || typeof effect !== 'object') throw new Error(`missing effect for ${key}`);
  const allowed = new Set(['stats', 'global', 'ability', 'grant', 'proc']);
  for (const name of Object.keys(effect)) if (!allowed.has(name)) throw new Error(`unknown effect field ${name} at ${key}`);
  return {
    index, origin: raw.origin, class_id: raw.class_id, level: raw.level ?? 0, origin_id: raw.id,
    origin_code: code,
    stats: numericRecord(effect.stats, STAT_FIELDS, `${key}.stats`),
    global: numericRecord(effect.global, GLOBAL_FIELDS, `${key}.global`),
    abilities: normalizeAbilities(effect.ability, key),
    grant: normalizeGrant(effect.grant, key),
    proc_id: effect.proc?.id ?? '',
  };
}

function numericRecord(value, fields, label) {
  if (value === undefined) return {};
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(`invalid numeric record ${label}`);
  const allowed = new Set(fields);
  const result = {};
  for (const [name, number] of Object.entries(value)) {
    if (!allowed.has(name) || !Number.isFinite(number)) throw new Error(`unknown numeric field ${label}.${name}`);
    result[name] = number;
  }
  return result;
}

function normalizeAbilities(value, label) {
  if (value === undefined) return [];
  if (!Array.isArray(value)) throw new Error(`invalid abilities ${label}`);
  return value.map((ability, index) => {
    if (!ability || typeof ability !== 'object' || typeof ability.ability !== 'string' || ability.ability.length === 0) {
      throw new Error(`invalid ability modifier ${label}:${index}`);
    }
    const allowed = new Set(['ability', ...ABILITY_NUMBER_FIELDS, 'dmgPctVsDottedAbility', 'castWhileMoving', 'damagePushbackImmune', 'addEffects']);
    for (const name of Object.keys(ability)) if (!allowed.has(name)) throw new Error(`unknown ability field ${label}:${index}.${name}`);
    const numbers = pickNumberFields(ability, ABILITY_NUMBER_FIELDS, `${label}:${index}`);
    if (ability.dmgPctVsDottedAbility !== undefined && typeof ability.dmgPctVsDottedAbility !== 'string') {
      throw new Error(`invalid dotted ability ${label}:${index}`);
    }
    if (ability.castWhileMoving !== undefined && typeof ability.castWhileMoving !== 'boolean') throw new Error(`invalid movement flag ${label}:${index}`);
    if (ability.damagePushbackImmune !== undefined && typeof ability.damagePushbackImmune !== 'boolean') throw new Error(`invalid pushback flag ${label}:${index}`);
    if (ability.addEffects !== undefined && !Array.isArray(ability.addEffects)) throw new Error(`invalid addEffects ${label}:${index}`);
    return {
      ability_id: ability.ability, numbers,
      dotted_ability_id: ability.dmgPctVsDottedAbility ?? '',
      cast_while_moving: ability.castWhileMoving ?? false,
      damage_pushback_immune: ability.damagePushbackImmune ?? false,
      add_effects: normalizeNestedEffects(ability.addEffects, `${label}:${index}`),
    };
  });
}

function normalizeNestedEffects(value, label) {
  if (value === undefined) return [];
  if (!Array.isArray(value)) throw new Error(`invalid addEffects ${label}`);
  return value.map((effect, index) => {
    if (!effect || typeof effect !== 'object' || !NESTED_EFFECT_CODES.has(effect.type)) {
      throw new Error(`unknown nested AbilityEffect ${label}:${index}`);
    }
    const allowed = new Set(['type', ...NESTED_NUMBER_FIELDS, ...NESTED_STRING_FIELDS]);
    for (const name of Object.keys(effect)) if (!allowed.has(name)) throw new Error(`unknown nested field ${label}:${index}.${name}`);
    return {
      type: effect.type,
      numbers: pickNumberFields(effect, NESTED_NUMBER_FIELDS, `${label}:${index}`),
      strings: Object.fromEntries(NESTED_STRING_FIELDS.map((name) => {
        const value = effect[name] ?? '';
        if (typeof value !== 'string') throw new Error(`invalid nested string ${label}:${index}.${name}`);
        return [name, value];
      })),
    };
  });
}

function pickNumberFields(value, fields, label) {
  const result = {};
  for (const name of fields) {
    if (value[name] === undefined) continue;
    if (!Number.isFinite(value[name])) throw new Error(`invalid numeric field ${label}.${name}`);
    result[name] = value[name];
  }
  return result;
}

function normalizeGrant(value, label) {
  if (value === undefined) return null;
  if (!value || typeof value !== 'object' || typeof value.ability !== 'string' || value.ability.length === 0 ||
      (value.rank !== undefined && (!Number.isInteger(value.rank) || value.rank < 1))) {
    throw new Error(`invalid grant ${label}`);
  }
  return { ability_id: value.ability, rank: value.rank ?? 1 };
}

function renderZr(document) {
  const entries = document.entries;
  const abilityRows = entries.flatMap((entry) => entry.abilities.map((ability, abilityIndex) => ({ entry, ability, abilityIndex })));
  const nestedRows = abilityRows.flatMap((row) => row.ability.add_effects.map((effect, effectIndex) => ({ ...row, effect, effectIndex })));
  const cases = (rows, condition, value, fallback) => rows.map((row) => `    if (${condition(row)}) return ${value(row)};`).join('\n') + `\n    return ${fallback};`;
  const quoted = (value) => JSON.stringify(value);
  const float = (value) => Number.isInteger(value) ? `${value}.0` : String(value);
  return '// Generated by examples/woc/tools/talent_modifier_catalog_codegen.mjs. Do not edit.\n' +
    `pub catalogSha(): string { return ${quoted(document.catalog_sha256)}; }\n` +
    `pub entryCount(): int { return ${entries.length}; }\n` +
    `pub statFieldCount(): int { return ${STAT_FIELDS.length}; }\n` +
    `pub globalFieldCount(): int { return ${GLOBAL_FIELDS.length}; }\n` +
    `pub abilityNumberFieldCount(): int { return ${ABILITY_NUMBER_FIELDS.length}; }\n\n` +
    `pub nestedEffectNumberFieldCount(): int { return ${NESTED_NUMBER_FIELDS.length}; }\n` +
    `pub nestedEffectStringFieldCount(): int { return ${NESTED_STRING_FIELDS.length}; }\n\n` +
    'pub statFieldName(field: int): string {\n' + cases(STAT_FIELDS.map((name, field) => ({ name, field })), (row) => `field == ${row.field}`, (row) => quoted(row.name), '""') + '\n}\n\n' +
    'pub globalFieldName(field: int): string {\n' + cases(GLOBAL_FIELDS.map((name, field) => ({ name, field })), (row) => `field == ${row.field}`, (row) => quoted(row.name), '""') + '\n}\n\n' +
    'pub globalUsesMax(field: int): bool { return field == 32; }\n\n' +
    'pub abilityNumberFieldName(field: int): string {\n' + cases(ABILITY_NUMBER_FIELDS.map((name, field) => ({ name, field })), (row) => `field == ${row.field}`, (row) => quoted(row.name), '""') + '\n}\n\n' +
    'pub entryOriginCode(index: int): uint {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => `<uint>${entry.origin_code}`, '<uint>0') + '\n}\n\n' +
    'pub entryIsSpec(index: int): bool {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => entry.origin === 'spec' ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub entryStat(index: int, field: int): float {\n' + cases(entries.flatMap((entry) => STAT_FIELDS.map((name, field) => ({ entry, field, value: entry.stats[name] ?? 0 }))), (row) => `index == ${row.entry.index} && field == ${row.field}`, (row) => float(row.value), '0.0') + '\n}\n\n' +
    'pub entryGlobal(index: int, field: int): float {\n' + cases(entries.flatMap((entry) => GLOBAL_FIELDS.map((name, field) => ({ entry, field, value: entry.global[name] ?? 0 }))), (row) => `index == ${row.entry.index} && field == ${row.field}`, (row) => float(row.value), '0.0') + '\n}\n\n' +
    'pub entryGrantAbilityId(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.grant?.ability_id ?? ''), '""') + '\n}\n\n' +
    'pub entryGrantRank(index: int): int {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => String(entry.grant?.rank ?? 0), '0') + '\n}\n\n' +
    'pub entryProcId(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.proc_id), '""') + '\n}\n\n' +
    'pub entryAbilityCount(index: int): int {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => String(entry.abilities.length), '0') + '\n}\n\n' +
    'pub abilityId(index: int, abilityIndex: int): string {\n' + cases(abilityRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex}`, (row) => quoted(row.ability.ability_id), '""') + '\n}\n\n' +
    'pub abilityNumber(index: int, abilityIndex: int, field: int): float {\n' + cases(abilityRows.flatMap((row) => ABILITY_NUMBER_FIELDS.map((name, field) => ({ ...row, field, value: row.ability.numbers[name] ?? 0 }))), (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex} && field == ${row.field}`, (row) => float(row.value), '0.0') + '\n}\n\n' +
    'pub abilityDottedAbilityId(index: int, abilityIndex: int): string {\n' + cases(abilityRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex}`, (row) => quoted(row.ability.dotted_ability_id), '""') + '\n}\n\n' +
    'pub abilityCastWhileMoving(index: int, abilityIndex: int): bool {\n' + cases(abilityRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex}`, (row) => row.ability.cast_while_moving ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub abilityDamagePushbackImmune(index: int, abilityIndex: int): bool {\n' + cases(abilityRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex}`, (row) => row.ability.damage_pushback_immune ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub abilityAddEffectCount(index: int, abilityIndex: int): int {\n' + cases(abilityRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex}`, (row) => String(row.ability.add_effects.length), '0') + '\n}\n\n' +
    'pub nestedEffectType(index: int, abilityIndex: int, effectIndex: int): int {\n' + cases(nestedRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex} && effectIndex == ${row.effectIndex}`, (row) => String(NESTED_EFFECT_CODES.get(row.effect.type)), '0') + '\n}\n\n' +
    'pub nestedEffectNumber(index: int, abilityIndex: int, effectIndex: int, field: int): float {\n' + cases(nestedRows.flatMap((row) => NESTED_NUMBER_FIELDS.map((name, field) => ({ ...row, field, value: row.effect.numbers[name] ?? 0 }))), (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex} && effectIndex == ${row.effectIndex} && field == ${row.field}`, (row) => float(row.value), '0.0') + '\n}\n\n' +
    'pub nestedEffectString(index: int, abilityIndex: int, effectIndex: int, field: int): string {\n' + cases(nestedRows.flatMap((row) => NESTED_STRING_FIELDS.map((name, field) => ({ ...row, field, value: row.effect.strings[name] }))), (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex} && effectIndex == ${row.effectIndex} && field == ${row.field}`, (row) => quoted(row.value), '""') + '\n}\n';
}

function extract() {
  const child = spawnSync(process.execPath, ['--no-warnings', '--experimental-loader', loaderUrl, extractor, 'wocgit:///src/sim/content/talents.ts'], {
    encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: COMMIT },
  });
  if (child.status !== 0) throw new Error(child.stderr || `modifier extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}
function blob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 32 * 1024 * 1024 }); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function writeOrCheck(path, text, label) { if (check) { if (!existsSync(path) || readFileSync(path, 'utf8') !== text) throw new Error(`${label} stale`); } else writeFileSync(path, text, 'utf8'); }
