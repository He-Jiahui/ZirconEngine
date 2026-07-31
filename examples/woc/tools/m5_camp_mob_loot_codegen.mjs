import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_MOB_COUNT = 47;
const EXPECTED_FIRST_ID = 'forest_wolf';
const EXPECTED_DUMMY_INDEX = 26;
const EXPECTED_LAST_ID = 'grix_the_tunnelking';
const EXPECTED_TOTALS = {
  mob_count: 47,
  loot_entry_count: 177,
  item_entry_count: 131,
  copper_entry_count: 46,
  quest_entry_count: 23,
  roll_group_entry_count: 24,
  component_tag_count: 29,
};
const FLAG_FIELDS = ['has_item_id', 'has_copper', 'has_quest_id', 'has_roll_group'];
const TEXT_FIELDS = ['item_id', 'quest_id', 'roll_group'];
const METRIC_FIELDS = ['copper', 'chance'];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm5_camp_mob_loot_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm5_camp_mob_loot.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm5_camp_mob_loot.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');

  const extracted = extractContent();
  validateExtracted(extracted);
  const source = sourceTexts();
  validateSourceStructure(source);
  const totals = calculateTotals(extracted.mobs);
  assert(JSON.stringify(totals) === JSON.stringify(EXPECTED_TOTALS),
    `camp mob loot totals drifted: ${JSON.stringify(totals)}`);
  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m5_camp_mob_loot_codegen.mjs',
    source_sha256: Object.fromEntries(
      Object.entries(source).map(([path, text]) => [path, sha256(text)]),
    ),
    totals,
    mobs: extracted.mobs,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({ mobs: catalog.mobs }));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(catalog));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} WOC camp mob loot: ` +
    `${totals.mob_count} mobs, ${totals.loot_entry_count} entries ` +
    `(${catalog.catalog_sha256.slice(0, 15)})\n`,
  );
}

function extractContent() {
  const child = spawnSync(process.execPath, [
    '--no-warnings', '--experimental-loader', loaderUrl, extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `camp mob loot extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function validateExtracted(extracted) {
  assert(Array.isArray(extracted.mobs), 'camp mob loot extractor returned no mob rows');
  assert(extracted.mobs.length === EXPECTED_MOB_COUNT, 'camp mob loot template count drifted');
  assert(
    extracted.mobs[0].id === EXPECTED_FIRST_ID &&
      extracted.mobs[EXPECTED_DUMMY_INDEX].id === 'training_dummy' &&
      extracted.mobs[EXPECTED_DUMMY_INDEX].loot_entries.length === 0 &&
      extracted.mobs.at(-1).id === EXPECTED_LAST_ID,
    'camp mob loot identity or dummy boundary drifted',
  );
  const ids = new Set();
  for (const mob of extracted.mobs) {
    assert(typeof mob.id === 'string' && mob.id.length > 0 && !ids.has(mob.id),
      'camp mob loot id is missing or duplicated');
    ids.add(mob.id);
    assert(Array.isArray(mob.loot_entries) && Array.isArray(mob.component_tags),
      `camp mob loot collections are missing for ${mob.id}`);
    for (const tag of mob.component_tags) {
      assert(typeof tag === 'string' && tag.length > 0, `invalid component tag for ${mob.id}`);
    }
    for (const entry of mob.loot_entries) {
      assert(Number.isFinite(entry.chance) && entry.chance >= 0 && entry.chance <= 1,
        `invalid loot chance for ${mob.id}`);
      assert(FLAG_FIELDS.every((field) => typeof entry[field] === 'boolean'),
        `invalid loot presence flag for ${mob.id}`);
      assert(TEXT_FIELDS.every((field) => typeof entry[field] === 'string'),
        `invalid loot text field for ${mob.id}`);
      assert(Number.isFinite(entry.copper) && entry.copper >= 0,
        `invalid loot copper for ${mob.id}`);
      assert(entry.has_item_id || entry.item_id.length === 0,
        `absent item entry retained a value for ${mob.id}`);
      assert(entry.has_quest_id || entry.quest_id.length === 0,
        `absent quest entry retained a value for ${mob.id}`);
      assert(entry.has_roll_group || entry.roll_group.length === 0,
        `absent roll-group entry retained a value for ${mob.id}`);
      assert(
        entry.has_copper || entry.copper === 0,
        `absent copper entry retained a value for ${mob.id}`,
      );
    }
  }
}

function sourceTexts() {
  const paths = [
    'src/sim/types.ts',
    'src/sim/data.ts',
    'src/sim/content/zone1.ts',
    'src/sim/content/zone2.ts',
    'src/sim/content/zone3.ts',
    'src/sim/loot/loot_roll.ts',
  ];
  return Object.fromEntries(paths.map((sourcePath) => [sourcePath, gitShow(sourcePath)]));
}

function validateSourceStructure(source) {
  assert(
    source['src/sim/types.ts'].includes('export interface LootEntry') &&
      source['src/sim/types.ts'].includes('rollGroup?: string;') &&
      source['src/sim/data.ts'].includes('export const MOBS: Record<string, MobTemplate> = {') &&
      source['src/sim/content/zone3.ts'].includes('training_dummy: {') &&
      source['src/sim/loot/loot_roll.ts'].includes('const rolledGroups = new Set<string>();') &&
      source['src/sim/loot/loot_roll.ts'].includes('const roll = ctx.rng.next();') &&
      source['src/sim/loot/loot_roll.ts'].includes('needsQuestDrop(ctx, entry, m)'),
    'camp mob loot source structure drifted',
  );
}

function calculateTotals(mobs) {
  const lootEntries = mobs.flatMap((mob) => mob.loot_entries);
  return {
    mob_count: mobs.length,
    loot_entry_count: lootEntries.length,
    item_entry_count: lootEntries.filter((entry) => entry.has_item_id).length,
    copper_entry_count: lootEntries.filter((entry) => entry.has_copper).length,
    quest_entry_count: lootEntries.filter((entry) => entry.has_quest_id).length,
    roll_group_entry_count: lootEntries.filter((entry) => entry.has_roll_group).length,
    component_tag_count: mobs.reduce((total, mob) => total + mob.component_tags.length, 0),
  };
}

function gitShow(sourcePath) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`], {
    encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  });
}

function renderZr(catalog) {
  const lines = [
    '// Generated by tools/m5_camp_mob_loot_codegen.mjs from pinned WOC source.',
    '// Data projection only: M5 owns the source-order loot-roll and inventory transaction.',
    '',
    'mobIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${catalog.mobs.length};`,
    '}',
    '',
    'pub catalogSha(): string {',
    `    return ${JSON.stringify(catalog.catalog_sha256)};`,
    '}',
    '',
    'pub mobCount(required: bool): int {',
    '    if (!required) { throw "woc camp mob loot count is required"; }',
    `    return ${catalog.mobs.length};`,
    '}',
    '',
    renderMobId(catalog.mobs), '',
    renderMobIndexOf(catalog.mobs), '',
    renderLootEntryCount(catalog.mobs), '',
    renderComponentTagCount(catalog.mobs), '',
    renderComponentTag(catalog.mobs), '',
    renderEntryFlag(catalog.mobs), '',
    renderEntryText(catalog.mobs), '',
    renderEntryMetric(catalog.mobs), '',
    renderContractTest(catalog), '',
  ];
  return lines.join('\n');
}

function renderMobId(mobs) {
  const lines = [
    'pub mobId(index: int, required: bool): string {',
    '    if (!required || !mobIndexIsValid(index)) { throw "woc camp mob loot index is invalid"; }',
  ];
  mobs.forEach((mob, index) => lines.push(`    if (index == ${index}) { return ${JSON.stringify(mob.id)}; }`));
  lines.push('    throw "woc camp mob loot index is invalid";', '}');
  return lines.join('\n');
}

function renderMobIndexOf(mobs) {
  const lines = [
    'pub mobIndexOf(id: string, required: bool): int {',
    '    if (!required) { throw "woc camp mob loot id is invalid"; }',
  ];
  mobs.forEach((mob, index) => lines.push(`    if (id == ${JSON.stringify(mob.id)}) { return ${index}; }`));
  lines.push('    return -1;', '}');
  return lines.join('\n');
}

function renderLootEntryCount(mobs) {
  const lines = [
    'pub lootEntryCount(mobIndex: int, required: bool): int {',
    '    if (!required || !mobIndexIsValid(mobIndex)) { throw "woc camp mob loot index is invalid"; }',
  ];
  mobs.forEach((mob, index) => lines.push(`    if (mobIndex == ${index}) { return ${mob.loot_entries.length}; }`));
  lines.push('    throw "woc camp mob loot index is invalid";', '}');
  return lines.join('\n');
}

function renderComponentTagCount(mobs) {
  const lines = [
    'pub componentTagCount(mobIndex: int, required: bool): int {',
    '    if (!required || !mobIndexIsValid(mobIndex)) { throw "woc camp mob loot index is invalid"; }',
  ];
  mobs.forEach((mob, index) => lines.push(`    if (mobIndex == ${index}) { return ${mob.component_tags.length}; }`));
  lines.push('    throw "woc camp mob loot index is invalid";', '}');
  return lines.join('\n');
}

function renderComponentTag(mobs) {
  const lines = [
    'pub componentTag(mobIndex: int, tagIndex: int, required: bool): string {',
    '    if (!required || !mobIndexIsValid(mobIndex)) { throw "woc camp mob tag query is invalid"; }',
  ];
  mobs.forEach((mob, mobIndex) => {
    lines.push(`    if (mobIndex == ${mobIndex}) {`);
    mob.component_tags.forEach((tag, tagIndex) => {
      lines.push(`        if (tagIndex == ${tagIndex}) { return ${JSON.stringify(tag)}; }`);
    });
    lines.push('        throw "woc camp mob tag index is invalid";', '    }');
  });
  lines.push('    throw "woc camp mob tag query is invalid";', '}');
  return lines.join('\n');
}

function renderEntryFlag(mobs) {
  const lines = [
    'pub entryFlag(mobIndex: int, entryIndex: int, field: string, required: bool): bool {',
    '    if (!required || !mobIndexIsValid(mobIndex)) { throw "woc camp mob loot query is invalid"; }',
  ];
  mobs.forEach((mob, mobIndex) => {
    lines.push(`    if (mobIndex == ${mobIndex}) {`);
    mob.loot_entries.forEach((entry, entryIndex) => {
      lines.push(`        if (entryIndex == ${entryIndex}) {`);
      FLAG_FIELDS.forEach((field) => {
        lines.push(`            if (field == ${JSON.stringify(field)}) { return ${entry[field] ? 'true' : 'false'}; }`);
      });
      lines.push('            return false;', '        }');
    });
    lines.push('        throw "woc camp mob loot entry index is invalid";', '    }');
  });
  lines.push('    throw "woc camp mob loot query is invalid";', '}');
  return lines.join('\n');
}

function renderEntryText(mobs) {
  const lines = [
    'pub entryText(mobIndex: int, entryIndex: int, field: string, required: bool): string {',
    '    if (!required || !mobIndexIsValid(mobIndex)) { throw "woc camp mob loot query is invalid"; }',
  ];
  mobs.forEach((mob, mobIndex) => {
    lines.push(`    if (mobIndex == ${mobIndex}) {`);
    mob.loot_entries.forEach((entry, entryIndex) => {
      lines.push(`        if (entryIndex == ${entryIndex}) {`);
      TEXT_FIELDS.forEach((field) => {
        lines.push(`            if (field == ${JSON.stringify(field)}) { return ${JSON.stringify(entry[field])}; }`);
      });
      lines.push('            return "";', '        }');
    });
    lines.push('        throw "woc camp mob loot entry index is invalid";', '    }');
  });
  lines.push('    throw "woc camp mob loot query is invalid";', '}');
  return lines.join('\n');
}

function renderEntryMetric(mobs) {
  const lines = [
    'pub entryMetric(mobIndex: int, entryIndex: int, field: string, required: bool): float {',
    '    if (!required || !mobIndexIsValid(mobIndex)) { throw "woc camp mob loot query is invalid"; }',
  ];
  mobs.forEach((mob, mobIndex) => {
    lines.push(`    if (mobIndex == ${mobIndex}) {`);
    mob.loot_entries.forEach((entry, entryIndex) => {
      lines.push(`        if (entryIndex == ${entryIndex}) {`);
      METRIC_FIELDS.forEach((field) => {
        lines.push(`            if (field == ${JSON.stringify(field)}) { return ${formatNumber(entry[field])}; }`);
      });
      lines.push('            return 0.0;', '        }');
    });
    lines.push('        throw "woc camp mob loot entry index is invalid";', '    }');
  });
  lines.push('    throw "woc camp mob loot query is invalid";', '}');
  return lines.join('\n');
}

function renderContractTest(catalog) {
  const lines = [
    'pub contractTest(): int {',
    `    if (catalogSha() != ${JSON.stringify(catalog.catalog_sha256)} || mobCount(true) != ${catalog.mobs.length}) { return -1; }`,
  ];
  catalog.mobs.forEach((mob, mobIndex) => {
    lines.push(`    if (mobId(${mobIndex}, true) != ${JSON.stringify(mob.id)} || mobIndexOf(${JSON.stringify(mob.id)}, true) != ${mobIndex} ||`);
    lines.push(`        lootEntryCount(${mobIndex}, true) != ${mob.loot_entries.length} || componentTagCount(${mobIndex}, true) != ${mob.component_tags.length}) { return -${mobIndex + 2}; }`);
    mob.component_tags.forEach((tag, tagIndex) => {
      lines.push(`    if (componentTag(${mobIndex}, ${tagIndex}, true) != ${JSON.stringify(tag)}) { return -${100 + mobIndex}; }`);
    });
    mob.loot_entries.forEach((entry, entryIndex) => {
      const failure = 1000 + mobIndex * 20 + entryIndex;
      lines.push(`    if (entryFlag(${mobIndex}, ${entryIndex}, "has_item_id", true) != ${entry.has_item_id ? 'true' : 'false'} ||`);
      lines.push(`        entryFlag(${mobIndex}, ${entryIndex}, "has_copper", true) != ${entry.has_copper ? 'true' : 'false'} ||`);
      lines.push(`        entryFlag(${mobIndex}, ${entryIndex}, "has_quest_id", true) != ${entry.has_quest_id ? 'true' : 'false'} ||`);
      lines.push(`        entryFlag(${mobIndex}, ${entryIndex}, "has_roll_group", true) != ${entry.has_roll_group ? 'true' : 'false'} ||`);
      lines.push(`        entryText(${mobIndex}, ${entryIndex}, "item_id", true) != ${JSON.stringify(entry.item_id)} ||`);
      lines.push(`        entryText(${mobIndex}, ${entryIndex}, "quest_id", true) != ${JSON.stringify(entry.quest_id)} ||`);
      lines.push(`        entryText(${mobIndex}, ${entryIndex}, "roll_group", true) != ${JSON.stringify(entry.roll_group)} ||`);
      lines.push(`        entryMetric(${mobIndex}, ${entryIndex}, "copper", true) != ${formatNumber(entry.copper)} ||`);
      lines.push(`        entryMetric(${mobIndex}, ${entryIndex}, "chance", true) != ${formatNumber(entry.chance)}) { return -${failure}; }`);
    });
  });
  lines.push('    return 1;', '}');
  return lines.join('\n');
}

function verifyOrWrite(path, text) {
  if (checkOnly) {
    assert(existsSync(path), `generated output is missing: ${path}`);
    assert(readFileSync(path, 'utf8') === text, `generated output drifted: ${path}`);
    return;
  }
  writeFileSync(path, text);
}

function formatNumber(value) {
  assert(Number.isFinite(value), `cannot emit non-finite number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : value.toString();
}

function sha256(text) {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
