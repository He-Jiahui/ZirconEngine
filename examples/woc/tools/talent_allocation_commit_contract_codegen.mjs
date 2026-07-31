import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const PATHS = ['src/sim/content/talents.ts', 'src/sim/content/talent_rows.ts', 'src/sim/progression/talents.ts'];
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(root, '..', '..', 'dev', 'world-of-claudecraft');
const reference = join(root, 'reference', 'current-head', 'talent_allocation_commit_contract.json');
const zr = join(root, 'scripts', 'woc_game', 'src', 'generated', 'talent_allocation_commit_contract.zr');
const check = process.argv.includes('--check');

const blobs = Object.fromEntries(PATHS.map((path) => [
  path,
  execFileSync('git', ['-C', source, 'show', `${COMMIT}:${path}`], { encoding: 'buffer' }),
]));
const content = blobs['src/sim/content/talents.ts'].toString('utf8');
const rowSource = blobs['src/sim/content/talent_rows.ts'].toString('utf8');
const progression = blobs['src/sim/progression/talents.ts'].toString('utf8');

const rowMatch = rowSource.match(/export const ROW_LEVELS = \[([^\]]+)\]/);
if (!rowMatch) throw new Error('target Talent row levels are missing');
const rowLevels = rowMatch[1].split(',').map((value) => Number(value.trim()));
if (rowLevels.length !== 6 || rowLevels.some((value) => !Number.isSafeInteger(value))) {
  throw new Error('target Talent row levels drifted');
}
const specUnlockMatch = content.match(/export const SPEC_UNLOCK_LEVEL = ROW_LEVELS\[0\];/);
if (!specUnlockMatch || rowLevels[0] !== 5) throw new Error('target specialization unlock rule drifted');
if (!content.includes('optionId.length > 128')) throw new Error('target Talent option length bound drifted');

const slice = (text, start, end) => {
  const from = text.indexOf(start);
  if (from < 0) throw new Error(`target source is missing ${start}`);
  const to = end ? text.indexOf(end, from) : text.length;
  if (to < 0) throw new Error(`target source is missing ${end}`);
  return text.slice(from, to);
};
const commitBody = slice(progression, 'function commitTalentAllocation(', '// Commit a whole staged allocation');
const ordered = [
  'const lock = talentLockReason(ctx, player);',
  'const check = validateAllocation(meta.cls, alloc, player.level);',
  'if (allocationsEqual(meta.talents, sanitized)) return true;',
  'meta.talents = sanitized;',
  'recomputeTalents(ctx, meta);',
  'ctx.revalidateOffhandForSpec(player.id);',
  'dismissSpecLockedPet(ctx, player, meta);',
  'stripTemporalEchoes(ctx, player.id);',
  "if (successText) ctx.emit({ type: 'log'",
];
let cursor = -1;
for (const marker of ordered) {
  const next = commitBody.indexOf(marker);
  if (next < 0 || next <= cursor) throw new Error(`commitTalentAllocation ordering drifted at ${marker}`);
  cursor = next;
}

const validation = slice(content, 'export function validateAllocation(', 'export function repairAllocation(');
const validationOrder = [
  [/if\s*\(\s*value\.spec\s*!==\s*null\s*&&\s*typeof\s+value\.spec\s*!==\s*'string'\s*\)/, 'invalid specialization shape'],
  [/if\s*\(\s*typeof\s+value\.spec\s*===\s*'string'\s*\)/, 'specialization membership'],
  [/if\s*\(\s*!isPlainRecord\(value\.rows\)\s*\)/, 'row record shape'],
  [/for\s*\(\s*const\s*\[rawLevel,\s*optionId\]\s*of\s*Object\.entries\(value\.rows\)\s*\)/, 'row traversal'],
];
cursor = -1;
for (const [pattern, label] of validationOrder) {
  const next = validation.search(pattern);
  if (next < 0 || next <= cursor) throw new Error(`validateAllocation ordering drifted at ${label}`);
  cursor = next;
}

const setSpec = slice(progression, 'export function setTalentSpec(', '/** Select or clear one canonical class-wide row');
if (setSpec.indexOf('!ct?.specs.some') < 0 || setSpec.indexOf('!ct?.specs.some') > setSpec.indexOf('return applyTalentAllocation')) {
  throw new Error('setTalentSpec unknown-spec precheck ordering drifted');
}
const selectRow = slice(progression, 'export function selectTalentRow(', '// Free respec');
if (selectRow.indexOf('const row = rowForLevel') < 0 || selectRow.indexOf('const row = rowForLevel') > selectRow.indexOf('return applyTalentAllocation')) {
  throw new Error('selectTalentRow structural validation ordering drifted');
}
const respec = slice(progression, 'export function respecTalents(', '// Save the current build');
if (!respec.includes('{ spec: r.meta.talents.spec, rows: {} }')) {
  throw new Error('respecTalents allocation reset drifted');
}

const doc = {
  schema_version: 1,
  source_commit: COMMIT,
  generated_by: 'examples/woc/tools/talent_allocation_commit_contract_codegen.mjs',
  source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, bytes]) => [
    path,
    createHash('sha256').update(bytes).digest('hex'),
  ])),
  row_levels: rowLevels,
  spec_unlock_level: rowLevels[0],
  option_id_max_length: 128,
  commit_order: [
    'combat_or_arena_lock', 'validate_allocation', 'equal_short_circuit', 'assign_allocation',
    'recompute', 'spec_offhand_revalidation', 'spec_pet_dismissal', 'temporal_echo_cleanup', 'success_log',
  ],
  entrypoint_order: {
    apply: ['lock', 'validate', 'equal', 'commit'],
    set_spec: ['unknown_spec_precheck', 'apply'],
    select_row: ['row_and_option_precheck', 'apply'],
    respec: ['lock', 'validate_retained_spec_and_empty_rows', 'commit'],
  },
};
const json = `${JSON.stringify(doc, null, 2)}\n`;
const zrs = `// Generated from ${COMMIT}; do not edit by hand.\n` +
  `pub specUnlockLevel(required: bool): int { return required ? ${rowLevels[0]} : 0; }\n` +
  'pub rowCount(required: bool): int { return required ? 6 : 0; }\n' +
  'pub optionIdMaxLength(required: bool): int { return required ? 128 : 0; }\n';
for (const [path, text, label] of [[reference, json, 'JSON'], [zr, zrs, 'Zr']]) {
  if (check) {
    if (!existsSync(path) || readFileSync(path, 'utf8') !== text) throw new Error(`${label} contract stale`);
  } else {
    writeFileSync(path, text, 'utf8');
  }
}
console.log(`${check ? 'checked' : 'generated'} talent allocation commit contract for ${COMMIT}`);
