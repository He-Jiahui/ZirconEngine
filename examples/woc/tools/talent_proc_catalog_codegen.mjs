import { spawnSync, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATHS = [
  'src/sim/content/talents.ts',
  'src/sim/content/talent_rows.ts',
  'src/sim/content/talents_warrior.ts',
  'src/sim/content/talents_classic.ts',
  'src/sim/content/warrior_rows.ts',
  'src/sim/content/choice_rows_classic.ts',
];
const TRIGGER_CODES = new Map([
  ['castNth', 1], ['spellCrit', 2], ['shieldConsumed', 3], ['hotExpired', 4],
  ['bigHitTaken', 5], ['meleeSwingWhile', 6], ['thornsReflect', 7],
]);
const RESPONSE_CODES = new Map([
  ['empowerNext', 1], ['cooldownRefund', 2], ['resource', 3], ['heal', 4],
  ['absorb', 5], ['aura', 6], ['echo', 7],
]);
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const extractorPath = join(scriptDirectory, 'talent_proc_catalog_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const selectionCatalogPath = join(projectRoot, 'reference', 'current-head', 'talent_selection_catalog.json');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'talent_proc_catalog.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'talent_proc_catalog.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const sourceBlobs = Object.fromEntries(SOURCE_PATHS.map((path) => [path, sourceBlob(path)]));
  const selectionCatalog = readJson(selectionCatalogPath);
  invariant(selectionCatalog.source_commit === SOURCE_COMMIT,
    'talent selection catalog is not pinned to the current target');
  const optionCodes = new Map(selectionCatalog.classes.flatMap((playerClass) =>
    playerClass.rows.flatMap((row) => row.options.map((option) => [option.id, option.code]))));
  const specCodes = new Map(selectionCatalog.classes.flatMap((playerClass) =>
    playerClass.specs.map((spec) => [spec.id, spec.code])));
  const extracted = extract();
  invariant(Array.isArray(extracted.entries), 'talent proc extractor did not return entries');
  const seenProcIds = new Set();
  const entries = extracted.entries.map((entry, index) => normalizeEntry(entry, index, optionCodes, specCodes, seenProcIds));
  invariant(entries.length === 55, `current target talent proc count drifted: expected 55, got ${entries.length}`);
  const triggerKinds = new Set(entries.map((entry) => entry.trigger.kind));
  const responseKinds = new Set(entries.flatMap((entry) => entry.responses.map((response) => response.kind)));
  invariant(triggerKinds.size === TRIGGER_CODES.size && [...TRIGGER_CODES.keys()].every((kind) => triggerKinds.has(kind)),
    'current target must retain every talent proc trigger family');
  invariant(responseKinds.size === RESPONSE_CODES.size && [...RESPONSE_CODES.keys()].every((kind) => responseKinds.has(kind)),
    'current target must retain every talent proc response family');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/talent_proc_catalog_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(sourceBlobs)
      .map(([path, blob]) => [path, sha256(blob)])),
    talent_selection_catalog_sha256: selectionCatalog.catalog_sha256,
    catalog_sha256: hashText(JSON.stringify(entries)),
    trigger_codes: Object.fromEntries(TRIGGER_CODES),
    response_codes: Object.fromEntries(RESPONSE_CODES),
    entries,
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'talent proc JSON catalog');
  writeOrCheck(zrOutput, renderZr(document), 'talent proc Zr catalog');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} ${entries.length} current talent procs (${document.catalog_sha256.slice(0, 15)})\n`);
}

function normalizeEntry(raw, index, optionCodes, specCodes, seenProcIds) {
  invariant(raw && typeof raw === 'object', `proc entry ${index} is not an object`);
  invariant(raw.origin === 'option' || raw.origin === 'spec', `invalid proc origin ${raw.origin}`);
  invariant(typeof raw.class_id === 'string' && raw.class_id.length > 0, `invalid proc class ${index}`);
  invariant(typeof raw.id === 'string' && raw.id.length > 0, `invalid proc origin id ${index}`);
  const originCode = raw.origin === 'option' ? optionCodes.get(raw.id) : specCodes.get(raw.id);
  invariant(Number.isInteger(originCode) && originCode > 0, `missing selection code for proc origin ${raw.id}`);
  const proc = raw.proc;
  invariant(proc && typeof proc === 'object' && typeof proc.id === 'string' && proc.id.length > 0,
    `invalid proc at ${raw.id}`);
  invariant(!seenProcIds.has(proc.id), `duplicate current talent proc id ${proc.id}`);
  seenProcIds.add(proc.id);
  invariant(typeof proc.name === 'string' && proc.name.length > 0, `invalid proc name ${proc.id}`);
  invariant(TRIGGER_CODES.has(proc.trigger?.on), `unknown trigger for ${proc.id}`);
  invariant(Array.isArray(proc.responses) && proc.responses.length > 0, `missing responses for ${proc.id}`);
  const trigger = normalizeTrigger(proc.id, proc.trigger);
  const responses = proc.responses.map((response, responseIndex) => normalizeResponse(proc.id, response, responseIndex));
  return {
    index,
    origin: raw.origin,
    class_id: raw.class_id,
    origin_id: raw.id,
    origin_code: originCode,
    id: proc.id,
    name: proc.name,
    school: proc.school ?? '',
    trigger,
    responses,
  };
}

function normalizeTrigger(procId, trigger) {
  const kind = trigger.on;
  const result = {
    kind,
    ability_ids: Array.isArray(trigger.abilities) ? trigger.abilities : [],
    ability_id: trigger.ability ?? '',
    aura_kind: trigger.auraKind ?? '',
    nth: trigger.n ?? 1,
    has_icd: trigger.icd !== undefined,
    icd: trigger.icd ?? 0,
    has_chance: trigger.chance !== undefined,
    chance: trigger.chance ?? 0,
    hp_fraction: trigger.hpFrac ?? 0,
  };
  if (kind === 'castNth') {
    invariant(Array.isArray(trigger.abilities) && trigger.abilities.length > 0 && Number.isInteger(trigger.n) && trigger.n >= 1,
      `invalid castNth trigger ${procId}`);
  }
  if (kind === 'spellCrit') invariant(trigger.abilities === undefined || result.ability_ids.length > 0, `invalid spellCrit trigger ${procId}`);
  if (kind === 'shieldConsumed' || kind === 'hotExpired' || kind === 'thornsReflect') invariant(result.ability_id.length > 0, `missing direct ability for ${procId}`);
  if (kind === 'meleeSwingWhile') invariant(result.aura_kind.length > 0, `missing melee aura for ${procId}`);
  if (kind === 'bigHitTaken') invariant(result.hp_fraction > 0 && result.has_icd && result.icd > 0, `invalid bigHit trigger ${procId}`);
  if (result.has_chance) invariant(result.chance >= 0 && result.chance <= 1, `invalid chance for ${procId}`);
  return result;
}

function normalizeResponse(procId, response, index) {
  invariant(response && RESPONSE_CODES.has(response.kind), `unknown response ${procId}:${index}`);
  const kind = response.kind;
  const result = {
    kind,
    aura_kind: response.aura ?? response.auraKind ?? '',
    ability_id: response.ability ?? '',
    ability_ids: Array.isArray(response.abilities) ? response.abilities : [],
    resource_type: response.resourceType ?? '',
    amount: response.amount ?? 0,
    has_amount_pct_max_hp: response.amountPctMaxHp !== undefined,
    amount_pct_max_hp: response.amountPctMaxHp ?? 0,
    duration: response.duration ?? 0,
    has_cost_pct: response.costPct !== undefined,
    cost_pct: response.costPct ?? 0,
    reset_cooldown: response.seconds === 'reset',
    below_fraction: response.belowFrac ?? 0,
    window: response.window ?? 0,
    name: response.name ?? '',
    value: response.value ?? 0,
  };
  if (kind === 'empowerNext') invariant(result.aura_kind.length > 0 && result.duration > 0, `invalid empower response ${procId}:${index}`);
  if (kind === 'cooldownRefund') invariant(result.ability_id.length > 0 && (result.reset_cooldown || Number.isFinite(response.seconds)), `invalid refund ${procId}:${index}`);
  if (kind === 'absorb') invariant(result.duration > 0, `invalid absorb ${procId}:${index}`);
  if (kind === 'aura') invariant(result.aura_kind.length > 0 && result.duration > 0, `invalid aura ${procId}:${index}`);
  if (kind === 'echo') invariant(result.window > 0 && result.below_fraction > 0, `invalid echo ${procId}:${index}`);
  return result;
}

function renderZr(document) {
  const entries = document.entries;
  const responseRows = entries.flatMap((entry) => entry.responses.map((response, responseIndex) => ({ entry, response, responseIndex })));
  const abilityRows = entries.flatMap((entry) => entry.trigger.ability_ids.map((abilityId, abilityIndex) => ({ entry, abilityId, abilityIndex })));
  const responseAbilityRows = responseRows.flatMap(({ entry, response, responseIndex }) => response.ability_ids.map((abilityId, abilityIndex) => ({ entry, responseIndex, abilityId, abilityIndex })));
  const cases = (rows, condition, value, fallback) => rows.map((row) => `    if (${condition(row)}) return ${value(row)};`).join('\n') + `\n    return ${fallback};`;
  const quoted = (value) => JSON.stringify(value);
  return '// Generated by examples/woc/tools/talent_proc_catalog_codegen.mjs. Do not edit.\n' +
    `pub catalogSha(): string { return ${quoted(document.catalog_sha256)}; }\n` +
    `pub procCount(): int { return ${entries.length}; }\n\n` +
    'pub procOriginCode(index: int): uint {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => `<uint>${entry.origin_code}`, '<uint>0') + '\n}\n\n' +
    'pub procOriginIsSpec(index: int): bool {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => entry.origin === 'spec' ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub procId(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.id), '""') + '\n}\n\n' +
    'pub procName(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.name), '""') + '\n}\n\n' +
    'pub procSchool(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.school), '""') + '\n}\n\n' +
    'pub procTriggerCode(index: int): int {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => String(TRIGGER_CODES.get(entry.trigger.kind)), '0') + '\n}\n\n' +
    'pub procNth(index: int): int {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => String(entry.trigger.nth), '1') + '\n}\n\n' +
    'pub procHasIcd(index: int): bool {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => entry.trigger.has_icd ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub procIcd(index: int): float {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => floatLiteral(entry.trigger.icd), '0.0') + '\n}\n\n' +
    'pub procHasChance(index: int): bool {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => entry.trigger.has_chance ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub procChance(index: int): float {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => floatLiteral(entry.trigger.chance), '0.0') + '\n}\n\n' +
    'pub procHpFraction(index: int): float {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => floatLiteral(entry.trigger.hp_fraction), '0.0') + '\n}\n\n' +
    'pub procDirectAbilityId(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.trigger.ability_id), '""') + '\n}\n\n' +
    'pub procRequiredAuraKind(index: int): string {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => quoted(entry.trigger.aura_kind), '""') + '\n}\n\n' +
    'pub procTriggerAbilityCount(index: int): int {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => String(entry.trigger.ability_ids.length), '0') + '\n}\n\n' +
    'pub procTriggerAbilityId(index: int, abilityIndex: int): string {\n' + cases(abilityRows, (row) => `index == ${row.entry.index} && abilityIndex == ${row.abilityIndex}`, (row) => quoted(row.abilityId), '""') + '\n}\n\n' +
    'pub procResponseCount(index: int): int {\n' + cases(entries, (entry) => `index == ${entry.index}`, (entry) => String(entry.responses.length), '0') + '\n}\n\n' +
    'pub responseKind(index: int, responseIndex: int): int {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => String(RESPONSE_CODES.get(row.response.kind)), '0') + '\n}\n\n' +
    'pub responseAuraKind(index: int, responseIndex: int): string {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => quoted(row.response.aura_kind), '""') + '\n}\n\n' +
    'pub responseAbilityId(index: int, responseIndex: int): string {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => quoted(row.response.ability_id), '""') + '\n}\n\n' +
    'pub responseAbilityCount(index: int, responseIndex: int): int {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => String(row.response.ability_ids.length), '0') + '\n}\n\n' +
    'pub responseAbilityAt(index: int, responseIndex: int, abilityIndex: int): string {\n' + cases(responseAbilityRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex} && abilityIndex == ${row.abilityIndex}`, (row) => quoted(row.abilityId), '""') + '\n}\n\n' +
    'pub responseResourceType(index: int, responseIndex: int): string {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => quoted(row.response.resource_type), '""') + '\n}\n\n' +
    'pub responseAmount(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.amount), '0.0') + '\n}\n\n' +
    'pub responseHasAmountPctMaxHp(index: int, responseIndex: int): bool {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => row.response.has_amount_pct_max_hp ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub responseAmountPctMaxHp(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.amount_pct_max_hp), '0.0') + '\n}\n\n' +
    'pub responseDuration(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.duration), '0.0') + '\n}\n\n' +
    'pub responseHasCostPct(index: int, responseIndex: int): bool {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => row.response.has_cost_pct ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub responseCostPct(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.cost_pct), '0.0') + '\n}\n\n' +
    'pub responseResetCooldown(index: int, responseIndex: int): bool {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => row.response.reset_cooldown ? 'true' : 'false', 'false') + '\n}\n\n' +
    'pub responseBelowFraction(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.below_fraction), '0.0') + '\n}\n\n' +
    'pub responseWindow(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.window), '0.0') + '\n}\n\n' +
    'pub responseName(index: int, responseIndex: int): string {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => quoted(row.response.name), '""') + '\n}\n\n' +
    'pub responseValue(index: int, responseIndex: int): float {\n' + cases(responseRows, (row) => `index == ${row.entry.index} && responseIndex == ${row.responseIndex}`, (row) => floatLiteral(row.response.value), '0.0') + '\n}\n';
}

function floatLiteral(value) { return Number.isInteger(value) ? `${value}.0` : String(value); }
function extract() {
  const child = spawnSync(process.execPath, ['--no-warnings', '--experimental-loader', loaderUrl, extractorPath, 'wocgit:///src/sim/content/talents.ts'], {
    encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `talent proc extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 16 * 1024 * 1024 }); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:talent-proc-catalog`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:talent-proc-catalog`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function hashText(value) { return createHash('sha256').update(value, 'utf8').digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
