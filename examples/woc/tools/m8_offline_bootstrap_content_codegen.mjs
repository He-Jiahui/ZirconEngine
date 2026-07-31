import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_CLASS_IDS = [
  'warrior', 'mage', 'rogue', 'paladin', 'hunter', 'priest', 'shaman', 'warlock', 'druid',
];
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const outputPath = join(projectRoot, 'contracts', 'm8_offline_bootstrap_content.json');
const zrOutputPath = join(
  projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm8_offline_bootstrap_content.zr',
);
const extractorPath = join(scriptDirectory, 'm8_offline_bootstrap_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  invariant(sourceManifest.source_commit === SOURCE_COMMIT, 'reference source commit drifted');
  const child = spawnSync(process.execPath, [
    '--no-warnings', '--experimental-loader', loaderUrl, extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `bootstrap extractor exited ${child.status}`);
  const extracted = JSON.parse(child.stdout);
  validateExtracted(extracted);
  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m8_offline_bootstrap_content_codegen.mjs',
    source_identities: sourceIdentities(),
    player_start: extracted.player_start,
    classes: extracted.classes,
    starter_items: extracted.starter_items,
  };
  catalog.catalog_sha256 = catalogHash(catalog);
  const json = `${JSON.stringify(catalog, null, 2)}\n`;
  writeOrCheck(outputPath, json);
  writeOrCheck(zrOutputPath, renderZr(catalog));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} M8 offline bootstrap content: ` +
    `${catalog.classes.length} classes, ${catalog.starter_items.length} starter items ` +
    `(${catalog.catalog_sha256.slice(0, 15)})\n`,
  );
}

function validateExtracted(extracted) {
  invariant(extracted && typeof extracted === 'object', 'bootstrap extraction is not an object');
  invariant(Number.isFinite(extracted.player_start?.x) && Number.isFinite(extracted.player_start?.z),
    'bootstrap player start is invalid');
  invariant(Array.isArray(extracted.classes), 'bootstrap classes are missing');
  invariant(JSON.stringify(extracted.classes.map((entry) => entry.id)) === JSON.stringify(EXPECTED_CLASS_IDS),
    `bootstrap class order drifted: ${JSON.stringify(extracted.classes.map((entry) => entry.id))}`);
  const starterIds = new Set();
  for (const entry of extracted.classes) {
    invariant(typeof entry.name === 'string' && entry.name.length > 0, `class name missing: ${entry.id}`);
    invariant(typeof entry.resource_type === 'string', `class resource missing: ${entry.id}`);
    invariant(Number.isFinite(entry.base_hp) && Number.isFinite(entry.base_mana),
      `class base pool missing: ${entry.id}`);
    for (const itemId of [entry.start_weapon, entry.start_chest]) {
      if (typeof itemId === 'string' && itemId.length > 0) starterIds.add(itemId);
    }
    for (const item of entry.start_items) {
      invariant(typeof item.item_id === 'string' && Number.isInteger(item.count) && item.count > 0,
        `class ration is invalid: ${entry.id}`);
      starterIds.add(item.item_id);
    }
  }
  invariant(Array.isArray(extracted.starter_items), 'starter items are missing');
  invariant(JSON.stringify(extracted.starter_items.map((entry) => entry.id)) ===
    JSON.stringify([...starterIds].sort()), 'starter item catalog drifted');
  for (const item of extracted.starter_items) {
    invariant(typeof item.kind === 'string' && item.kind.length > 0, `starter item kind missing: ${item.id}`);
    for (const metric of Object.values(item.stats)) invariant(Number.isFinite(metric), `invalid starter stat: ${item.id}`);
  }
}

function sourceIdentities() {
  const paths = [
    'src/sim/sim.ts',
    'src/sim/entity.ts',
    'src/sim/data.ts',
    'src/sim/content/classes.ts',
    'src/sim/content/items.ts',
    'src/sim/types.ts',
  ];
  return {
    representation: 'git_blob_lf',
    files: paths.map((path) => textIdentity(path, gitShow(path))),
  };
}

function renderZr(catalog) {
  const lines = [
    '// Generated by examples/woc/tools/m8_offline_bootstrap_content_codegen.mjs.',
    `// Source ${catalog.source_commit}; raw Sim/addPlayer bootstrap inputs only; do not edit.`,
    '',
    'pub catalogSha(): string {',
    `    return ${zrString(catalog.catalog_sha256)};`,
    '}',
    '',
    'pub playerStartX(): float {',
    `    return ${zrNumber(catalog.player_start.x)};`,
    '}',
    '',
    'pub playerStartZ(): float {',
    `    return ${zrNumber(catalog.player_start.z)};`,
    '}',
    '',
    'pub classCount(): int {',
    `    return ${catalog.classes.length};`,
    '}',
    '',
    'pub classId(index: int): string {',
  ];
  catalog.classes.forEach((entry, index) => lines.push(
    `    if (index == ${index}) { return ${zrString(entry.id)}; }`,
  ));
  lines.push('    throw "unknown WOC bootstrap class index";', '}', '',
    'pub classIndex(id: string): int {');
  catalog.classes.forEach((entry, index) => lines.push(
    `    if (id == ${zrString(entry.id)}) { return ${index}; }`,
  ));
  lines.push('    return -1;', '}', '');
  renderClassText(lines, catalog.classes);
  lines.push('');
  renderClassMetric(lines, catalog.classes);
  lines.push('');
  renderClassRations(lines, catalog.classes);
  lines.push('');
  renderClassAbilities(lines, catalog.classes);
  lines.push('');
  renderStarterItems(lines, catalog.starter_items);
  lines.push('', ...renderContractTest(catalog));
  return `${lines.join('\n')}\n`;
}

function renderClassText(lines, classes) {
  lines.push('pub classText(index: int, field: string): string {');
  for (const [index, entry] of classes.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const fields = {
      id: entry.id,
      name: entry.name,
      resourceType: entry.resource_type,
      startWeapon: entry.start_weapon,
      startChest: entry.start_chest,
      rangedSchool: entry.ranged?.school ?? '',
    };
    for (const [field, value] of Object.entries(fields)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrString(value)}; }`);
    }
    lines.push('        return "";', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap class index";', '}');
}

function renderClassMetric(lines, classes) {
  lines.push('pub classMetric(index: int, field: string): float {');
  for (const [index, entry] of classes.entries()) {
    lines.push(`    if (index == ${index}) {`);
    const values = {
      baseStr: entry.base_stats.str,
      baseAgi: entry.base_stats.agi,
      baseSta: entry.base_stats.sta,
      baseInt: entry.base_stats.int,
      baseSpi: entry.base_stats.spi,
      baseArmor: entry.base_stats.armor,
      growthStr: entry.stats_per_level.str,
      growthAgi: entry.stats_per_level.agi,
      growthSta: entry.stats_per_level.sta,
      growthInt: entry.stats_per_level.int,
      growthSpi: entry.stats_per_level.spi,
      growthArmor: entry.stats_per_level.armor,
      baseHp: entry.base_hp,
      hpPerLevel: entry.hp_per_level,
      baseMana: entry.base_mana,
      manaPerLevel: entry.mana_per_level,
      color: entry.color,
      rangedMin: entry.ranged?.min ?? 0,
      rangedMax: entry.ranged?.max ?? 0,
      rangedSpeed: entry.ranged?.speed ?? 0,
      rangedMaxRange: entry.ranged?.max_range ?? 0,
      rangedMinRange: entry.ranged?.min_range ?? 0,
      rangedWand: entry.ranged?.wand ? 1 : 0,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap class index";', '}');
}

function renderClassRations(lines, classes) {
  lines.push('pub classRationCount(index: int): int {');
  classes.forEach((entry, index) => lines.push(`    if (index == ${index}) { return ${entry.start_items.length}; }`));
  lines.push('    throw "unknown WOC bootstrap class index";', '}', '',
    'pub classRationId(index: int, rationIndex: int): string {');
  for (const [index, entry] of classes.entries()) {
    lines.push(`    if (index == ${index}) {`);
    entry.start_items.forEach((item, rationIndex) => lines.push(
      `        if (rationIndex == ${rationIndex}) { return ${zrString(item.item_id)}; }`,
    ));
    lines.push('        throw "unknown WOC bootstrap ration index";', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap class index";', '}', '',
    'pub classRationCountValue(index: int, rationIndex: int): int {');
  for (const [index, entry] of classes.entries()) {
    lines.push(`    if (index == ${index}) {`);
    entry.start_items.forEach((item, rationIndex) => lines.push(
      `        if (rationIndex == ${rationIndex}) { return ${item.count}; }`,
    ));
    lines.push('        throw "unknown WOC bootstrap ration index";', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap class index";', '}');
}

function renderClassAbilities(lines, classes) {
  lines.push('pub classAbilityCount(index: int): int {');
  classes.forEach((entry, index) => lines.push(`    if (index == ${index}) { return ${entry.abilities.length}; }`));
  lines.push('    throw "unknown WOC bootstrap class index";', '}', '',
    'pub classAbilityId(index: int, abilityIndex: int): string {');
  for (const [index, entry] of classes.entries()) {
    lines.push(`    if (index == ${index}) {`);
    entry.abilities.forEach((id, abilityIndex) => lines.push(
      `        if (abilityIndex == ${abilityIndex}) { return ${zrString(id)}; }`,
    ));
    lines.push('        throw "unknown WOC bootstrap ability index";', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap class index";', '}');
}

function renderStarterItems(lines, items) {
  lines.push('pub starterItemCount(): int {', `    return ${items.length};`, '}', '',
    'pub starterItemText(id: string, field: string): string {');
  for (const item of items) {
    lines.push(`    if (id == ${zrString(item.id)}) {`);
    for (const [field, value] of Object.entries({
      id: item.id, kind: item.kind, slot: item.slot ?? '', armorType: item.armor_type ?? '', quality: item.quality ?? '',
    })) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrString(value)}; }`);
    }
    lines.push('        return "";', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap starter item";', '}', '',
    'pub starterItemMetric(id: string, field: string): float {');
  for (const item of items) {
    lines.push(`    if (id == ${zrString(item.id)}) {`);
    const values = {
      statStr: item.stats.str,
      statAgi: item.stats.agi,
      statSta: item.stats.sta,
      statInt: item.stats.int,
      statSpi: item.stats.spi,
      statArmor: item.stats.armor,
      weaponMin: item.weapon?.min ?? 0,
      weaponMax: item.weapon?.max ?? 0,
      weaponSpeed: item.weapon?.speed ?? 0,
      foodHp: item.food_hp,
      drinkMana: item.drink_mana,
    };
    for (const [field, value] of Object.entries(values)) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC bootstrap starter item";', '}');
}

function renderContractTest(catalog) {
  const warrior = catalog.classes.find((entry) => entry.id === 'warrior');
  const mage = catalog.classes.find((entry) => entry.id === 'mage');
  const wornSword = catalog.starter_items.find((entry) => entry.id === 'worn_sword');
  const water = catalog.starter_items.find((entry) => entry.id === 'spring_water');
  return [
    'pub contractTest(): int {',
    `    if (catalogSha() != ${zrString(catalog.catalog_sha256)} || classCount() != 9 ||`,
    `        playerStartX() != ${zrNumber(catalog.player_start.x)} || playerStartZ() != ${zrNumber(catalog.player_start.z)}) { return -1; }`,
    `    if (classIndex("warrior") != 0 || classIndex("druid") != 8 || classIndex("missing") != -1 ||`,
    `        classText(0, "resourceType") != "rage" || classMetric(0, "baseSta") != ${zrNumber(warrior.base_stats.sta)} ||`,
    `        classText(0, "startWeapon") != "worn_sword") { return -2; }`,
    '    var mageIndex = classIndex("mage");',
    `    if (mageIndex != 1 || classText(mageIndex, "resourceType") != "mana" || classRationCount(mageIndex) != ${mage.start_items.length} ||`,
    `        classRationId(mageIndex, 1) != "spring_water" || classRationCountValue(mageIndex, 1) != 5) { return -3; }`,
    `    if (starterItemMetric("worn_sword", "weaponMin") != ${zrNumber(wornSword.weapon.min)} ||`,
    `        starterItemMetric("worn_sword", "weaponSpeed") != ${zrNumber(wornSword.weapon.speed)} ||`,
    `        starterItemMetric("spring_water", "drinkMana") != ${zrNumber(water.drink_mana)}) { return -4; }`,
    '    var index = 0;',
    '    while (index < classCount()) {',
    '        if (classIndex(classId(index)) != index) { return -5; }',
    '        index = index + 1;',
    '    }',
    '    return 1;',
    '}',
  ];
}

function catalogHash(catalog) {
  return hashText(JSON.stringify({
    player_start: catalog.player_start,
    classes: catalog.classes,
    starter_items: catalog.starter_items,
  }));
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
}

function textIdentity(path, text) {
  return { path, bytes: Buffer.byteLength(text, 'utf8'), sha256: hashText(text) };
}

function writeOrCheck(path, content) {
  if (checkOnly) {
    invariant(existsSync(path), `${path} is missing; run npm run generate:m8-offline-bootstrap`);
    invariant(readFileSync(path, 'utf8') === content, `${path} is stale; run npm run generate:m8-offline-bootstrap`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, 'utf8');
}

function zrString(value) {
  return JSON.stringify(value);
}

function zrNumber(value) {
  invariant(Number.isFinite(value), `non-finite Zr number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function hashText(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
