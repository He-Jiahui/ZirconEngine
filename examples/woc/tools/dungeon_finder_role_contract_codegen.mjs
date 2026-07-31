import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOCIAL_SOURCE_PATH = 'src/sim/social/dungeon_finder.ts';
const CONTENT_SOURCE_PATH = 'src/sim/content/dungeon_finder.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'dungeon_finder_role_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'dungeon_finder_role_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const socialSource = sourceBlob(SOCIAL_SOURCE_PATH);
  const contentSource = sourceBlob(CONTENT_SOURCE_PATH);
  for (const needle of [
    "export const FINDER_ROLE_ORDER: readonly Role[] = ['tank', 'healer', 'dps'];",
  ]) {
    invariant(contentSource.includes(needle), 'Dungeon Finder role catalog drifted: ' + needle);
  }
  for (const needle of [
    'export function matchFinderRoles(',
    'const holders: Record<Role, number[]> = { tank: [], healer: [], dps: [] };',
    'const roleOf = new Map<number, Role>();',
    'for (const role of FINDER_ROLE_ORDER) {',
    'if (role === except || !members[i].roles.includes(role) || visited.has(role)) continue;',
    'if (holders[role].length < caps[role]) {',
    'if (seat(otherIdx, visited, role)) {',
    'export function assignFinderRoles(',
    'if (members.length !== caps.tank + caps.healer + caps.dps) return null;',
    'if (match.assigned.size !== members.length) return null;',
  ]) {
    invariant(socialSource.includes(needle), 'Dungeon Finder role source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/dungeon_finder_role_contract_codegen.mjs',
    source_blobs: {
      [SOCIAL_SOURCE_PATH]: sha256(socialSource),
      [CONTENT_SOURCE_PATH]: sha256(contentSource),
    },
    role_codes: {
      none: 0,
      tank: 1,
      healer: 2,
      dps: 3,
    },
    role_order: ['tank', 'healer', 'dps'],
    default_compositions: {
      five: { tank: 1, healer: 1, dps: 3 },
      ten: { tank: 2, healer: 2, dps: 6 },
    },
    matching: {
      algorithm: 'kuhn_augmenting_path',
      member_order: 'input',
      visited_scope: 'one_augmenting_path',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'Dungeon Finder role JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Dungeon Finder role Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' Dungeon Finder role contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  for (const [name, value] of Object.entries(document.role_codes)) {
    lines.push('pub role' + titleCase(name) + '(required: bool): int { return required ? ' + value + ' : 0; }\n');
  }
  for (const [composition, caps] of Object.entries(document.default_compositions)) {
    for (const [role, value] of Object.entries(caps)) {
      lines.push('pub ' + composition + titleCase(role) + '(required: bool): int { return required ? ' + value + ' : 0; }\n');
    }
  }
  return lines.join('');
}

function titleCase(value) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
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
