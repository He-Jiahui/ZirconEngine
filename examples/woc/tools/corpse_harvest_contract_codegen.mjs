import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_TAGS = ['hide', 'fang', 'claw', 'tusk', 'venomSac', 'silk', 'gills', 'horn'];
const EXPECTED_MAX_COMPONENTS_PER_MOB = 3;
const EXPECTED_MAPPED_TAGS = ['hide', 'fang', 'silk', 'venomSac'];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const mobContractPath = join(projectRoot, 'contracts', 'm5_camp_mob_loot.json');
const contractPath = join(projectRoot, 'reference', 'current-head', 'corpse_harvest_contract.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'corpse_harvest_contract.zr');
const rustPath = join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'corpse_harvest_contract.rs');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');

  const source = Object.fromEntries([
    'src/sim/content/professions.ts',
    'src/sim/professions/gathering.ts',
    'src/sim/interaction.ts',
  ].map((path) => [path, gitShow(path)]));
  const componentItems = extractComponentItems(source['src/sim/content/professions.ts']);
  const mobs = extractMobs();
  const tags = [...new Set(mobs.flatMap((mob) => mob.component_tags))];
  const maxComponentsPerMob = Math.max(...mobs.map((mob) => mob.component_tags.length));
  validate(source, componentItems, mobs, tags, maxComponentsPerMob);

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/corpse_harvest_contract_codegen.mjs',
    source_sha256: Object.fromEntries(Object.entries(source).map(([path, text]) => [path, sha256(text)])),
    tags,
    component_items: componentItems,
    max_components_per_mob: maxComponentsPerMob,
    mobs: mobs.map((mob) => ({ id: mob.id, component_tags: mob.component_tags })),
    tier_weights: [40, 30, 15, 10, 4, 1],
    rarity_baseline: 40,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({
    tags: catalog.tags,
    component_items: catalog.component_items,
    max_components_per_mob: catalog.max_components_per_mob,
    mobs: catalog.mobs,
    tier_weights: catalog.tier_weights,
    rarity_baseline: catalog.rarity_baseline,
  }));
  verifyOrWrite(contractPath, `${JSON.stringify(catalog, null, 2)}\n`);
  verifyOrWrite(zrPath, renderZr(catalog));
  verifyOrWrite(rustPath, renderRust(catalog));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} WOC corpse harvest: ` +
    `${tags.length} tags, ${mobs.length} mobs, ${catalog.catalog_sha256.slice(0, 15)}\n`,
  );
}

function extractMobs() {
  const extracted = JSON.parse(readFileSync(mobContractPath, 'utf8'));
  assert(extracted.source_commit === SOURCE_COMMIT, 'camp mob contract source commit drifted');
  assert(Array.isArray(extracted.mobs), 'mob extractor did not return mobs');
  return extracted.mobs;
}

function extractComponentItems(text) {
  const match = text.match(/HARVEST_COMPONENT_ITEMS[^=]*=\s*\{([\s\S]*?)\n\};/);
  assert(match, 'source harvest component item map is missing');
  const entries = [];
  for (const line of match[1].split('\n')) {
    const entry = line.match(/^\s*([A-Za-z][A-Za-z0-9_]*)\s*:\s*'([^']+)'\s*,?\s*$/);
    if (entry) entries.push([entry[1], entry[2]]);
  }
  assert(entries.length > 0, 'source harvest component item map is empty');
  return Object.fromEntries(entries);
}

function validate(source, componentItems, mobs, tags, maxComponentsPerMob) {
  assert(JSON.stringify(tags) === JSON.stringify(EXPECTED_TAGS), `corpse tag order drifted: ${JSON.stringify(tags)}`);
  assert(maxComponentsPerMob === EXPECTED_MAX_COMPONENTS_PER_MOB,
    `corpse tag maximum drifted: ${maxComponentsPerMob}`);
  assert(JSON.stringify(Object.keys(componentItems)) === JSON.stringify(EXPECTED_MAPPED_TAGS),
    `corpse mapped tags drifted: ${JSON.stringify(Object.keys(componentItems))}`);
  assert(mobs.length === 47, `camp mob count drifted: ${mobs.length}`);
  for (const mob of mobs) {
    assert(mob.component_tags.length <= EXPECTED_MAX_COMPONENTS_PER_MOB,
      `corpse tag count exceeds protocol bound for ${mob.id}`);
    for (const tag of mob.component_tags) {
      assert(tags.includes(tag), `unknown corpse tag ${tag} for ${mob.id}`);
    }
  }
  assert(
    source['src/sim/interaction.ts'].includes('if (wanted.length > 0 && !fitsAll') &&
      source['src/sim/interaction.ts'].includes('mob.harvestClaimedBy = claim.claimedBy;') &&
      source['src/sim/interaction.ts'].includes('ctx.addItemInstance(itemId, { signer: meta.name }') &&
      source['src/sim/professions/gathering.ts'].includes('const BASE_TIER_WEIGHTS: readonly number[] = [40, 30, 15, 10, 4, 1];') &&
      source['src/sim/professions/gathering.ts'].includes('export const CORPSE_HARVEST_RARITY_BASELINE = 40;'),
    'corpse harvest source authority structure drifted',
  );
}

function renderZr(catalog) {
  const lines = [
    '// Generated by tools/corpse_harvest_contract_codegen.mjs from pinned WOC source.',
    '// Source facts only; world/state.zr owns command gates, RNG and transactions.',
    '',
    'pub catalogSha(): string {', `    return ${JSON.stringify(catalog.catalog_sha256)};`, '}', '',
    'pub componentTagCount(required: bool): int {',
    '    if (!required) { throw "woc corpse-harvest tag count is required"; }',
    `    return ${catalog.tags.length};`, '}', '',
    'pub maxComponentsPerMob(required: bool): int {',
    '    if (!required) { throw "woc corpse-harvest maximum is required"; }',
    `    return ${catalog.max_components_per_mob};`, '}', '',
    'pub componentCode(tag: string): uint {',
  ];
  catalog.tags.forEach((tag, index) => lines.push(`    if (tag == ${JSON.stringify(tag)}) { return <uint>${index + 1}; }`));
  lines.push('    return <uint>0;', '}', '', 'pub componentTag(code: uint, required: bool): string {',
    '    if (!required || code == <uint>0) { throw "woc corpse-harvest component code is invalid"; }');
  catalog.tags.forEach((tag, index) => lines.push(`    if (code == <uint>${index + 1}) { return ${JSON.stringify(tag)}; }`));
  lines.push('    throw "woc corpse-harvest component code is invalid";', '}', '',
    'pub itemIdForComponentCode(code: uint): string {');
  catalog.tags.forEach((tag, index) => {
    const itemId = catalog.component_items[tag] ?? '';
    lines.push(`    if (code == <uint>${index + 1}) { return ${JSON.stringify(itemId)}; }`);
  });
  lines.push('    return "";', '}', '', 'pub mobComponentTagCount(mobIndex: int, required: bool): int {',
    '    if (!required || mobIndex < 0) { throw "woc corpse-harvest mob index is invalid"; }');
  catalog.mobs.forEach((mob, index) => lines.push(`    if (mobIndex == ${index}) { return ${mob.component_tags.length}; }`));
  lines.push('    throw "woc corpse-harvest mob index is invalid";', '}', '',
    'pub mobComponentCode(mobIndex: int, componentIndex: int, required: bool): uint {',
    '    if (!required || mobIndex < 0 || componentIndex < 0) { throw "woc corpse-harvest mob component is invalid"; }');
  catalog.mobs.forEach((mob, mobIndex) => {
    lines.push(`    if (mobIndex == ${mobIndex}) {`);
    mob.component_tags.forEach((tag, componentIndex) => {
      lines.push(`        if (componentIndex == ${componentIndex}) { return <uint>${catalog.tags.indexOf(tag) + 1}; }`);
    });
    lines.push('        throw "woc corpse-harvest mob component is invalid";', '    }');
  });
  lines.push('    throw "woc corpse-harvest mob component is invalid";', '}', '',
    'pub tierWeight(tierIndex: int, required: bool): int {',
    '    if (!required || tierIndex < 0) { throw "woc corpse-harvest tier is invalid"; }');
  catalog.tier_weights.forEach((weight, index) => lines.push(`    if (tierIndex == ${index}) { return ${weight}; }`));
  lines.push('    throw "woc corpse-harvest tier is invalid";', '}', '',
    'pub rarityBaseline(required: bool): int {',
    '    if (!required) { throw "woc corpse-harvest rarity baseline is required"; }',
    `    return ${catalog.rarity_baseline};`, '}', '',
    'pub contractTest(): int {',
    `    if (catalogSha() != ${JSON.stringify(catalog.catalog_sha256)} || componentTagCount(true) != ${catalog.tags.length} || maxComponentsPerMob(true) != ${catalog.max_components_per_mob}) { return -1; }`,
  );
  catalog.tags.forEach((tag, index) => lines.push(
    `    if (componentCode(${JSON.stringify(tag)}) != <uint>${index + 1} || componentTag(<uint>${index + 1}, true) != ${JSON.stringify(tag)}) { return -${index + 2}; }`,
  ));
  lines.push('    return 1;', '}', '');
  return lines.join('\n');
}

function renderRust(catalog) {
  const tags = catalog.tags.map((tag) => JSON.stringify(tag)).join(', ');
  const itemIds = catalog.tags
    .map((tag) => `    ${JSON.stringify(catalog.component_items[tag] ?? '')}`)
    .join(',\n');
  return [
    '// Generated by tools/corpse_harvest_contract_codegen.mjs from pinned WOC source.',
    `pub const CORPSE_HARVEST_COMPONENT_TAG_COUNT: usize = ${catalog.tags.length};`,
    `pub const CORPSE_HARVEST_MAX_COMPONENTS_PER_MOB: usize = ${catalog.max_components_per_mob};`,
    'pub const CORPSE_HARVEST_COMPONENT_TAGS: [&str; CORPSE_HARVEST_COMPONENT_TAG_COUNT] = [',
    `    ${tags},`,
    '];',
    'pub const CORPSE_HARVEST_ITEM_IDS_BY_COMPONENT_CODE: [&str; CORPSE_HARVEST_COMPONENT_TAG_COUNT] = [',
    `${itemIds},`,
    '];',
    '',
    'pub fn is_valid_corpse_harvest_component_code(code: u8) -> bool {',
    '    code <= CORPSE_HARVEST_COMPONENT_TAG_COUNT as u8',
    '}',
    '',
    'pub fn corpse_harvest_component_code(tag: &str) -> u8 {',
    '    CORPSE_HARVEST_COMPONENT_TAGS',
    '        .iter()',
    '        .position(|candidate| *candidate == tag)',
    '        .map(|index| index as u8 + 1)',
    '        .unwrap_or(0)',
    '}',
    '',
  ].join('\n');
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8', maxBuffer: 16 * 1024 * 1024,
  });
}

function verifyOrWrite(path, content) {
  if (checkOnly) {
    assert(existsSync(path), `missing generated output ${path}`);
    assert(readFileSync(path, 'utf8') === content, `generated output drifted: ${path}`);
  } else {
    writeFileSync(path, content);
  }
}

function sha256(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
