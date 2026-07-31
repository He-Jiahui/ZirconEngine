import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const PATHS = ['src/sim/content/talents.ts', 'src/sim/content/talent_rows.ts', 'src/sim/content/talents_warrior.ts', 'src/sim/content/talents_classic.ts', 'src/sim/talent_loadouts.ts', 'src/sim/talent_save_migration.ts'];
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(root, '..', '..', 'dev', 'world-of-claudecraft');
const reference = join(root, 'reference', 'current-head', 'talent_loadout_migration_contract.json');
const zr = join(root, 'scripts', 'woc_game', 'src', 'generated', 'talent_loadout_migration_contract.zr');
const check = process.argv.includes('--check');

const blobs = Object.fromEntries(PATHS.map((path) => [path, execFileSync('git', ['-C', source, 'show', `${COMMIT}:${path}`], { encoding: 'buffer' })]));
const talents = blobs[PATHS[0]].toString('utf8');
const rows = blobs['src/sim/content/talent_rows.ts'].toString('utf8');
const loadouts = blobs['src/sim/talent_loadouts.ts'].toString('utf8');
const migration = blobs['src/sim/talent_save_migration.ts'].toString('utf8');
if (!talents.includes('export const MAX_LOADOUTS = 10;') || !talents.includes('export const SAVED_LOADOUT_BAR_SLOTS = 22;')) throw new Error('talent loadout bounds drifted');
if (!loadouts.includes(".slice(0, MAX_LOADOUTS)") || !loadouts.includes(".slice(0, SAVED_LOADOUT_BAR_SLOTS)")) throw new Error('loadout repair bounds drifted');
if (!migration.includes('export const CURRENT_CHARACTER_CONTENT_REVISION = 1;') || !migration.includes('if (\n    Number.isSafeInteger(state.contentRevision)')) throw new Error('Talents V2 migration revision guard drifted');
if (!talents.includes('export const SPEC_UNLOCK_LEVEL = ROW_LEVELS[0];')) throw new Error('Talent V2 specialization unlock rule drifted');
const rowLevelsMatch = rows.match(/export const ROW_LEVELS = \[([^\]]+)\]/);
if (!rowLevelsMatch) throw new Error('Talent row levels are missing');
const rowLevels = rowLevelsMatch[1].split(',').map((value) => Number(value.trim()));
if (rowLevels.length !== 6 || rowLevels.some((value) => !Number.isSafeInteger(value))) throw new Error('Talent row levels are invalid');
const specUnlockLevel = rowLevels[0];
const warrior = blobs['src/sim/content/talents_warrior.ts'].toString('utf8');
const classic = blobs['src/sim/content/talents_classic.ts'].toString('utf8');
const specs = [...warrior.matchAll(/id:\s*'([^']+)',\s*class:\s*'([^']+)'/g), ...classic.matchAll(/spec\(\s*'([^']+)'\s*,\s*'([^']+)'\s*,/g)].map((m) => ({ id: m[1], classId: m[2] }));
if (specs.length !== 27) throw new Error(`expected 27 target specs, found ${specs.length}`);
const doc = { schema_version: 1, source_commit: COMMIT, generated_by: 'examples/woc/tools/talent_loadout_migration_contract_codegen.mjs', source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, bytes]) => [path, createHash('sha256').update(bytes).digest('hex')])), max_loadouts: 10, bar_slots: 22, name_limit: 24, content_revision: 1, spec_unlock_level: specUnlockLevel, specs };
const json = `${JSON.stringify(doc, null, 2)}\n`;
const zrs = `// Generated from ${COMMIT}; do not edit by hand.\npub maxLoadouts(required: bool): int { return required ? 10 : 0; }\npub barSlots(required: bool): int { return required ? 22 : 0; }\npub nameLimit(required: bool): int { return required ? 24 : 0; }\npub contentRevision(required: bool): int { return required ? 1 : 0; }\npub specUnlockLevel(required: bool): int { return required ? ${specUnlockLevel} : 0; }\npub knownSpec(classId: string, specId: string): bool {\n${specs.map((spec) => `    if (classId == "${spec.classId}" && specId == "${spec.id}") return true;`).join('\n')}\n    return false;\n}\n`;
for (const [path, text, label] of [[reference, json, 'JSON'], [zr, zrs, 'Zr']]) {
  if (check) { if (!existsSync(path) || readFileSync(path, 'utf8') !== text) throw new Error(`${label} contract stale`); }
  else writeFileSync(path, text, 'utf8');
}
console.log(`${check ? 'checked' : 'generated'} talent loadout migration contract for ${COMMIT}`);
