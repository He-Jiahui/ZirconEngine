import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/item_level.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'item_level_catalog_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'item_level_catalog.zr');
const extractorPath = join(scriptDirectory, 'item_level_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  for (const needle of [
    'function buildSourceIndex(): Map<string, ItemSource> {',
    'for (const mob of Object.values(MOBS)) {',
    'for (const quest of Object.values(QUESTS)) {',
    'for (const offer of HEROIC_VENDOR_STOCK)',
    'for (const itemId of FURY_STOCK)',
    'for (const [bossId, entries] of Object.entries(HEROIC_BOSS_LOOT))',
    'export function itemSourceLevel(itemId: string): number | undefined {',
    'export function itemFromRaid(itemId: string): boolean {',
    'export function itemLevel(item: ItemDef): number | undefined {',
  ]) {
    invariant(source.includes(needle), 'item-level catalog source drifted: ' + needle);
  }

  const extracted = sourceExtract();
  invariant(Array.isArray(extracted.items) && extracted.items.length > 0,
    'item-level source extractor returned no items');
  const ids = new Set();
  for (const item of extracted.items) {
    invariant(typeof item.id === 'string' && item.id.length > 0 && !ids.has(item.id),
      'item-level source extractor produced duplicate or invalid item id');
    ids.add(item.id);
    invariant(item.source_level === null || (Number.isInteger(item.source_level) && item.source_level > 0),
      'item-level source extractor produced invalid source level for ' + item.id);
    invariant(typeof item.from_raid === 'boolean',
      'item-level source extractor produced invalid raid flag for ' + item.id);
    invariant(item.item_level === null || (Number.isInteger(item.item_level) && item.item_level > 0),
      'item-level source extractor produced invalid item level for ' + item.id);
  }
  invariant(extracted.items.every((item, index) => index === 0 || extracted.items[index - 1].id < item.id),
    'item-level source extractor must sort item ids');

  const sourceItems = extracted.items.filter((item) => item.source_level !== null);
  const itemLevelItems = extracted.items.filter((item) => item.item_level !== null);
  const raidItems = extracted.items.filter((item) => item.from_raid);
  invariant(sourceItems.length > 0 && itemLevelItems.length > 0 && raidItems.length > 0,
    'item-level source extractor lost a required source/item-level/raid partition');
  const fixtureSource = sourceItems[0];
  const fixtureNoSource = extracted.items.find((item) => item.source_level === null);
  const fixtureRaid = raidItems[0];
  const fixtureItemLevel = itemLevelItems[0];
  invariant(fixtureNoSource, 'item-level source extractor lost the no-source partition');
  const fixtureRareSource = extracted.rare_source_fixture;
  invariant(
    fixtureRareSource && typeof fixtureRareSource.id === 'string' &&
    Number.isInteger(fixtureRareSource.source_level) && fixtureRareSource.source_level > 0 &&
    sourceItems.some((item) => item.id === fixtureRareSource.id &&
      item.source_level === fixtureRareSource.source_level),
    'item-level source extractor lost the sourced rare fixture',
  );

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/item_level_catalog_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    source_evaluation: 'typescript_git_loader executes the pinned item_level.ts graph against pinned content',
    item_count: extracted.items.length,
    source_item_count: sourceItems.length,
    item_level_count: itemLevelItems.length,
    raid_item_count: raidItems.length,
    item_entries_sha256: sha256(JSON.stringify(extracted.items)),
    fixtures: {
      source_item_id: fixtureSource.id,
      source_item_level: fixtureSource.source_level,
      no_source_item_id: fixtureNoSource.id,
      raid_item_id: fixtureRaid.id,
      raid_item_source_level: fixtureRaid.source_level,
      item_level_item_id: fixtureItemLevel.id,
      item_level: fixtureItemLevel.item_level,
      rare_source_item_id: fixtureRareSource.id,
      rare_source_level: fixtureRareSource.source_level,
    },
    items: extracted.items,
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'item-level catalog JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'item-level catalog Zr contract');
  process.stdout.write(
    (checkOnly ? 'checked' : 'generated') + ' item-level catalog: ' +
    document.item_count + ' items, ' + document.source_item_count + ' sources, ' +
    document.item_level_count + ' levels, ' + document.raid_item_count + ' raid items\n',
  );
}

function sourceExtract() {
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    env: {
      ...process.env,
      WOC_GIT_ROOT: sourceRoot,
      WOC_GIT_COMMIT: SOURCE_COMMIT,
    },
  });
  invariant(child.status === 0, child.stderr || 'item-level source extractor exited ' + child.status);
  return JSON.parse(child.stdout);
}

function renderZr(document) {
  const sourceItems = document.items.filter((item) => item.source_level !== null);
  const itemLevelItems = document.items.filter((item) => item.item_level !== null);
  const raidItems = document.items.filter((item) => item.from_raid);
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  appendIntegerLookup(lines, 'itemSourceLevel', sourceItems, 'source_level');
  appendBooleanLookup(lines, 'itemFromRaid', raidItems);
  appendIntegerLookup(lines, 'itemLevel', itemLevelItems, 'item_level');
  lines.push('pub fixtureSourceItemId(): string { return ' + JSON.stringify(document.fixtures.source_item_id) + '; }\n');
  lines.push('pub fixtureSourceItemLevel(): int { return ' + document.fixtures.source_item_level + '; }\n');
  lines.push('pub fixtureNoSourceItemId(): string { return ' + JSON.stringify(document.fixtures.no_source_item_id) + '; }\n');
  lines.push('pub fixtureRaidItemId(): string { return ' + JSON.stringify(document.fixtures.raid_item_id) + '; }\n');
  lines.push('pub fixtureRaidItemSourceLevel(): int { return ' + document.fixtures.raid_item_source_level + '; }\n');
  lines.push('pub fixtureItemLevelItemId(): string { return ' + JSON.stringify(document.fixtures.item_level_item_id) + '; }\n');
  lines.push('pub fixtureItemLevel(): int { return ' + document.fixtures.item_level + '; }\n');
  lines.push('pub fixtureRareSourceItemId(): string { return ' + JSON.stringify(document.fixtures.rare_source_item_id) + '; }\n');
  lines.push('pub fixtureRareSourceLevel(): int { return ' + document.fixtures.rare_source_level + '; }\n');
  return lines.join('');
}

function appendIntegerLookup(lines, name, entries, field) {
  lines.push('pub ' + name + '(itemId: string): int {\n');
  for (const entry of entries) {
    lines.push('    if (itemId == ' + JSON.stringify(entry.id) + ') return ' + entry[field] + ';\n');
  }
  lines.push('    return 0;\n}\n');
}

function appendBooleanLookup(lines, name, entries) {
  lines.push('pub ' + name + '(itemId: string): bool {\n');
  for (const entry of entries) {
    lines.push('    if (itemId == ' + JSON.stringify(entry.id) + ') return true;\n');
  }
  lines.push('    return false;\n}\n');
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
