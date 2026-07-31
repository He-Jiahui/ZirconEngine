import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_CLASS_IDS = [
  'warrior', 'mage', 'rogue', 'paladin', 'hunter', 'priest', 'shaman', 'warlock', 'druid',
];
const LEVEL_CAP = 20;
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const bootstrapCatalogPath = join(projectRoot, 'contracts', 'm8_offline_bootstrap_content.json');
const freshPlayerCatalogPath = join(projectRoot, 'contracts', 'm8_fresh_player_stats.json');
const outputPath = join(projectRoot, 'contracts', 'm5_class_baseline_stats.json');
const zrOutputPath = join(
  projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm5_class_baseline_stats.zr',
);
const extractorPath = join(scriptDirectory, 'm5_class_baseline_stats_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  const bootstrapCatalog = JSON.parse(readFileSync(bootstrapCatalogPath, 'utf8'));
  const freshPlayerCatalog = JSON.parse(readFileSync(freshPlayerCatalogPath, 'utf8'));
  invariant(sourceManifest.source_commit === SOURCE_COMMIT, 'reference source commit drifted');
  invariant(bootstrapCatalog.source_commit === SOURCE_COMMIT, 'bootstrap catalog source commit drifted');
  invariant(freshPlayerCatalog.source_commit === SOURCE_COMMIT, 'fresh player catalog source commit drifted');

  const child = spawnSync(process.execPath, [
    '--no-warnings', '--experimental-loader', loaderUrl, extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `M5 class baseline extractor exited ${child.status}`);
  const extracted = JSON.parse(child.stdout);
  validateExtracted(extracted, bootstrapCatalog, freshPlayerCatalog);

  const catalog = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m5_class_baseline_stats_codegen.mjs',
    basis: {
      equipment: 'each class startWeapon and startChest only',
      talents: 'none',
      aura: 'none',
      hp: 'full maxHp after recalcPlayerStats',
      resource: 'mana=maxResource, energy=100, rage=0 after recalcPlayerStats',
    },
    source_identities: sourceIdentities(),
    max_level: extracted.max_level,
    classes: extracted.classes,
  };
  catalog.catalog_sha256 = catalogHash(catalog);
  writeOrCheck(outputPath, `${JSON.stringify(catalog, null, 2)}\n`);
  writeOrCheck(zrOutputPath, renderZr(catalog));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} M5 class baseline stats: ` +
    `${catalog.classes.length} classes x ${catalog.max_level} levels ` +
    `(${catalog.catalog_sha256.slice(0, 15)})\n`,
  );
}

function validateExtracted(extracted, bootstrapCatalog, freshPlayerCatalog) {
  invariant(extracted && typeof extracted === 'object', 'baseline extraction is not an object');
  invariant(extracted.max_level === LEVEL_CAP, `target level cap drifted: ${extracted.max_level}`);
  invariant(Array.isArray(extracted.classes), 'baseline class list is missing');
  invariant(JSON.stringify(extracted.classes.map((entry) => entry.class_id)) ===
    JSON.stringify(EXPECTED_CLASS_IDS), 'baseline class order drifted');
  invariant(Array.isArray(bootstrapCatalog.classes), 'bootstrap classes are missing');
  invariant(Array.isArray(freshPlayerCatalog.players), 'fresh player facts are missing');

  for (const [classIndex, entry] of extracted.classes.entries()) {
    const bootstrap = bootstrapCatalog.classes[classIndex];
    const fresh = freshPlayerCatalog.players[classIndex];
    invariant(bootstrap?.id === entry.class_id, `bootstrap class mismatch: ${entry.class_id}`);
    invariant(fresh?.class_id === entry.class_id, `fresh class mismatch: ${entry.class_id}`);
    invariant(entry.resource_type === bootstrap.resource_type && entry.resource_type === fresh.resource_type,
      `resource type mismatch: ${entry.class_id}`);
    invariant(entry.equipment?.mainhand === bootstrap.start_weapon &&
      entry.equipment?.chest === bootstrap.start_chest,
    `starting equipment mismatch: ${entry.class_id}`);
    validateEquipmentContribution(entry.class_id, entry.equipment_contributions?.mainhand, true);
    validateEquipmentContribution(entry.class_id, entry.equipment_contributions?.chest, false);
    invariant(JSON.stringify(entry.start_items) === JSON.stringify(bootstrap.start_items),
      `starter inventory mismatch: ${entry.class_id}`);
    invariant(Array.isArray(entry.levels) && entry.levels.length === LEVEL_CAP,
      `level count mismatch: ${entry.class_id}`);
    for (const [offset, profile] of entry.levels.entries()) {
      invariant(profile.level === offset + 1,
        `level sequence mismatch: ${entry.class_id} ${profile.level}`);
      validateProfile(entry.class_id, entry.resource_type, profile);
    }
    validateLevelOne(entry.class_id, entry.levels[0], fresh);
  }
}

function validateEquipmentContribution(classId, contribution, requiresWeapon) {
  invariant(contribution && typeof contribution.item_id === 'string' && contribution.item_id.length > 0,
    `starting equipment contribution is missing: ${classId}`);
  for (const value of [
    contribution.stats?.str, contribution.stats?.agi, contribution.stats?.sta,
    contribution.stats?.int, contribution.stats?.armor, contribution.spell_power,
    contribution.crit_rating, contribution.haste_rating, contribution.hit_rating,
    contribution.weapon?.min, contribution.weapon?.max, contribution.weapon?.speed,
  ]) {
    invariant(Number.isFinite(value), `non-finite starting equipment contribution: ${classId}`);
  }
  invariant(Number.isInteger(contribution.stats.str) && Number.isInteger(contribution.stats.agi) &&
    Number.isInteger(contribution.stats.sta) && Number.isInteger(contribution.stats.int) &&
    Number.isInteger(contribution.stats.armor) && Number.isInteger(contribution.spell_power) &&
    Number.isInteger(contribution.crit_rating) && Number.isInteger(contribution.haste_rating) &&
    Number.isInteger(contribution.hit_rating),
  `non-integral starting equipment contribution: ${classId}`);
  if (requiresWeapon) {
    invariant(contribution.weapon.min > 0 && contribution.weapon.max >= contribution.weapon.min &&
      contribution.weapon.speed > 0, `invalid starting mainhand contribution: ${classId}`);
  } else {
    invariant(contribution.weapon.min === 0 && contribution.weapon.max === 0 &&
      contribution.weapon.speed === 0, `starting non-weapon contribution drifted: ${classId}`);
  }
}

function validateProfile(classId, resourceType, profile) {
  invariant(profile.hp === profile.max_hp && profile.max_hp > 0,
    `baseline hp mismatch: ${classId} level ${profile.level}`);
  const expectedResource = resourceType === 'mana'
    ? profile.max_resource
    : resourceType === 'energy' ? 100 : 0;
  invariant(profile.resource === expectedResource && profile.max_resource > 0,
    `baseline resource mismatch: ${classId} level ${profile.level}`);
  for (const value of Object.values(profile.stats ?? {})) {
    invariant(Number.isFinite(value), `non-finite stat: ${classId} level ${profile.level}`);
  }
  const input = profile.pre_form;
  invariant(input && Number.isFinite(input.strength) && Number.isFinite(input.agility) &&
    Number.isFinite(input.stamina) && Number.isFinite(input.intellect) &&
    Number.isFinite(input.armor_before_agility) &&
    Number.isFinite(input.bonus_attack_power) && Number.isFinite(input.bonus_spell_power) &&
    Number.isFinite(input.base_hp) && Number.isFinite(input.hp_per_level),
  `pre-form input is invalid: ${classId} level ${profile.level}`);
  invariant(input.strength === profile.stats.str && input.agility === profile.stats.agi &&
    input.stamina === profile.stats.sta && input.intellect === profile.stats.int &&
    input.armor_before_agility + input.agility * 2 === profile.stats.armor,
  `pre-form primary-stat ordering drifted: ${classId} level ${profile.level}`);
  invariant(input.base_hp + input.hp_per_level * (profile.level - 1) +
    hpFromStamina(input.stamina) === profile.max_hp,
  `pre-form max HP ordering drifted: ${classId} level ${profile.level}`);
  for (const value of [
    profile.weapon?.min, profile.weapon?.max, profile.weapon?.speed, profile.attack_power,
    profile.ranged_power, profile.spell_power, profile.crit_chance, profile.dodge_chance,
    profile.move_speed,
  ]) {
    invariant(Number.isFinite(value), `non-finite derived value: ${classId} level ${profile.level}`);
  }
  invariant(profile.weapon.min > 0 && profile.weapon.max >= profile.weapon.min &&
    profile.weapon.speed > 0 && profile.move_speed > 0,
  `invalid weapon or movement: ${classId} level ${profile.level}`);
}

function validateLevelOne(classId, profile, fresh) {
  const values = {
    level: profile.level,
    stats: profile.stats,
    weapon: profile.weapon,
    max_hp: profile.max_hp,
    hp: profile.hp,
    max_resource: profile.max_resource,
    resource: profile.resource,
    attack_power: profile.attack_power,
    ranged_power: profile.ranged_power,
    spell_power: profile.spell_power,
    crit_chance: profile.crit_chance,
    dodge_chance: profile.dodge_chance,
    move_speed: profile.move_speed,
  };
  const expected = {
    level: fresh.level,
    stats: fresh.stats,
    weapon: fresh.weapon,
    max_hp: fresh.max_hp,
    hp: fresh.hp,
    max_resource: fresh.max_resource,
    resource: fresh.resource,
    attack_power: fresh.attack_power,
    ranged_power: fresh.ranged_power,
    spell_power: fresh.spell_power,
    crit_chance: fresh.crit_chance,
    dodge_chance: fresh.dodge_chance,
    move_speed: fresh.move_speed,
  };
  invariant(JSON.stringify(values) === JSON.stringify(expected),
    `level one diverged from M8 fresh player facts: ${classId}`);
}

function sourceIdentities() {
  const paths = [
    'src/sim/entity.ts',
    'src/sim/sim.ts',
    'src/sim/types.ts',
    'src/sim/data.ts',
    'src/sim/item_level_req.ts',
    'src/sim/pvp/index.ts',
    'src/sim/content/classes.ts',
    'src/sim/content/items.ts',
    'src/sim/content/weapon_skin_rules.ts',
  ];
  return {
    representation: 'git_blob_lf',
    files: paths.map((path) => textIdentity(path, gitShow(path))),
  };
}

function renderZr(catalog) {
  const lines = [
    '// Generated by examples/woc/tools/m5_class_baseline_stats_codegen.mjs.',
    `// Source ${catalog.source_commit}; start gear, no talents/aura baseline only; do not edit.`,
    '',
    'pub catalogSha(): string {',
    `    return ${zrString(catalog.catalog_sha256)};`,
    '}',
    '',
    'pub classCount(): int {',
    `    return ${catalog.classes.length};`,
    '}',
    '',
    'pub levelCap(): int {',
    `    return ${catalog.max_level};`,
    '}',
    '',
    'pub classId(index: int): string {',
  ];
  catalog.classes.forEach((entry, index) => lines.push(
    `    if (index == ${index}) { return ${zrString(entry.class_id)}; }`,
  ));
  lines.push('    throw "unknown WOC baseline class index";', '}', '',
    'pub classIndex(id: string): int {');
  catalog.classes.forEach((entry, index) => lines.push(
    `    if (id == ${zrString(entry.class_id)}) { return ${index}; }`,
  ));
  lines.push('    return -1;', '}', '',
    '// Class ids join M8 raw bootstrap content; this module owns derived baseline values only.', '');
  renderStartingEquipment(lines, catalog.classes);
  lines.push('');
  renderClassInteger(lines, catalog.classes);
  lines.push('');
  renderClassDecimal(lines, catalog.classes);
  lines.push('', ...renderContractTest(catalog));
  return `${lines.join('\n')}\n`;
}

function renderStartingEquipment(lines, classes) {
  lines.push('pub startingEquipmentText(classIndex: int, slot: string): string {');
  for (const [classIndex, entry] of classes.entries()) {
    lines.push(`    if (classIndex == ${classIndex}) {`);
    for (const slot of ['mainhand', 'chest']) {
      lines.push(`        if (slot == ${zrString(slot)}) { return ${zrString(entry.equipment_contributions[slot].item_id)}; }`);
    }
    lines.push('        return "";', '    }');
  }
  lines.push('    throw "unknown WOC baseline class index";', '}', '',
    'pub startingEquipmentInteger(classIndex: int, slot: string, field: string): int {');
  for (const [classIndex, entry] of classes.entries()) {
    lines.push(`    if (classIndex == ${classIndex}) {`);
    for (const slot of ['mainhand', 'chest']) {
      lines.push(`        if (slot == ${zrString(slot)}) {`);
      for (const [field, value] of Object.entries(equipmentIntegerValues(entry.equipment_contributions[slot]))) {
        lines.push(`            if (field == ${zrString(field)}) { return ${zrInt(value)}; }`);
      }
      lines.push('            return 0;', '        }');
    }
    lines.push('        return 0;', '    }');
  }
  lines.push('    throw "unknown WOC baseline class index";', '}', '',
    'pub startingEquipmentDecimal(classIndex: int, slot: string, field: string): float {');
  for (const [classIndex, entry] of classes.entries()) {
    lines.push(`    if (classIndex == ${classIndex}) {`);
    for (const slot of ['mainhand', 'chest']) {
      lines.push(`        if (slot == ${zrString(slot)}) {`);
      lines.push(`            if (field == "weaponSpeed") { return ${zrNumber(entry.equipment_contributions[slot].weapon.speed)}; }`);
      lines.push('            return 0.0;', '        }');
    }
    lines.push('        return 0.0;', '    }');
  }
  lines.push('    throw "unknown WOC baseline class index";', '}');
}

function equipmentIntegerValues(contribution) {
  return {
    statStr: contribution.stats.str,
    statAgi: contribution.stats.agi,
    statSta: contribution.stats.sta,
    statInt: contribution.stats.int,
    statArmor: contribution.stats.armor,
    spellPower: contribution.spell_power,
    critRating: contribution.crit_rating,
    hasteRating: contribution.haste_rating,
    hitRating: contribution.hit_rating,
    weaponMin: contribution.weapon.min,
    weaponMax: contribution.weapon.max,
  };
}

function renderClassInteger(lines, classes) {
  lines.push('pub classInteger(index: int, level: int, field: string): int {');
  for (const [classIndex, entry] of classes.entries()) {
    lines.push(`    if (index == ${classIndex}) {`);
    for (const profile of entry.levels) {
      lines.push(`        if (level == ${profile.level}) {`);
      const values = integerValues(profile);
      for (const [field, value] of Object.entries(values)) {
        lines.push(`            if (field == ${zrString(field)}) { return ${zrInt(value)}; }`);
      }
      lines.push('            return 0;', '        }');
    }
    lines.push('        throw "unknown WOC baseline level";', '    }');
  }
  lines.push('    throw "unknown WOC baseline class index";', '}');
}

function renderClassDecimal(lines, classes) {
  lines.push('pub classDecimal(index: int, level: int, field: string): float {');
  for (const [classIndex, entry] of classes.entries()) {
    lines.push(`    if (index == ${classIndex}) {`);
    for (const profile of entry.levels) {
      lines.push(`        if (level == ${profile.level}) {`);
      const values = decimalValues(profile);
      for (const [field, value] of Object.entries(values)) {
        lines.push(`            if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
      }
      lines.push('            return 0.0;', '        }');
    }
    lines.push('        throw "unknown WOC baseline level";', '    }');
  }
  lines.push('    throw "unknown WOC baseline class index";', '}');
}

function integerValues(profile) {
  return {
    maxHp: profile.max_hp,
    hp: profile.hp,
    maxResource: profile.max_resource,
    resource: profile.resource,
    statStr: profile.stats.str,
    statAgi: profile.stats.agi,
    statSta: profile.stats.sta,
    statInt: profile.stats.int,
    statSpi: profile.stats.spi,
    statArmor: profile.stats.armor,
    inputArmorBeforeAgility: profile.pre_form.armor_before_agility,
    inputBonusAttackPower: profile.pre_form.bonus_attack_power,
    inputBonusSpellPower: profile.pre_form.bonus_spell_power,
    inputBaseHp: profile.pre_form.base_hp,
    inputHpPerLevel: profile.pre_form.hp_per_level,
    weaponMin: profile.weapon.min,
    weaponMax: profile.weapon.max,
    attackPower: profile.attack_power,
    rangedPower: profile.ranged_power,
    spellPower: profile.spell_power,
  };
}

function decimalValues(profile) {
  return {
    weaponSpeed: profile.weapon.speed,
    critChance: profile.crit_chance,
    dodgeChance: profile.dodge_chance,
    moveSpeed: profile.move_speed,
    pvpOffense: profile.stats.pvp_offense,
    pvpDefense: profile.stats.pvp_defense,
  };
}

function renderContractTest(catalog) {
  const warrior = catalog.classes.find((entry) => entry.class_id === 'warrior');
  const mage = catalog.classes.find((entry) => entry.class_id === 'mage');
  const hunter = catalog.classes.find((entry) => entry.class_id === 'hunter');
  const warriorLevelOne = warrior.levels[0];
  const warriorLevelCap = warrior.levels.at(-1);
  const mageLevelOne = mage.levels[0];
  const mageLevelCap = mage.levels.at(-1);
  const hunterLevelCap = hunter.levels.at(-1);
  const warriorMainhand = warrior.equipment_contributions.mainhand;
  const mageMainhand = mage.equipment_contributions.mainhand;
  return [
    'pub contractTest(): int {',
    `    if (catalogSha() != ${zrString(catalog.catalog_sha256)} || classCount() != 9 || levelCap() != ${catalog.max_level}) { return -1; }`,
    `    if (classIndex("warrior") != 0 || classIndex("druid") != 8 || classIndex("missing") != -1 ||`,
    `        classInteger(0, 1, "maxHp") != ${zrInt(warriorLevelOne.max_hp)} ||`,
    `        classInteger(0, levelCap(), "maxHp") != ${zrInt(warriorLevelCap.max_hp)} ||`,
    `        classInteger(0, levelCap(), "attackPower") != ${zrInt(warriorLevelCap.attack_power)}) { return -2; }`,
    '    var mageIndex = classIndex("mage");',
    `    if (mageIndex != 1 ||`,
    `        classInteger(mageIndex, 1, "maxResource") != ${zrInt(mageLevelOne.max_resource)} ||`,
    `        classInteger(mageIndex, levelCap(), "maxResource") != ${zrInt(mageLevelCap.max_resource)} ||`,
    `        classInteger(mageIndex, levelCap(), "spellPower") != ${zrInt(mageLevelCap.spell_power)} ||`,
    `        classDecimal(mageIndex, levelCap(), "weaponSpeed") != ${zrNumber(mageLevelCap.weapon.speed)}) { return -3; }`,
    `    if (classInteger(4, levelCap(), "rangedPower") != ${zrInt(hunterLevelCap.ranged_power)} ||`,
    `        classDecimal(4, levelCap(), "moveSpeed") != ${zrNumber(hunterLevelCap.move_speed)}) { return -4; }`,
    `    if (startingEquipmentText(0, "mainhand") != ${zrString(warriorMainhand.item_id)} ||`,
    `        startingEquipmentInteger(0, "mainhand", "weaponMin") != ${zrInt(warriorMainhand.weapon.min)} ||`,
    `        startingEquipmentDecimal(0, "mainhand", "weaponSpeed") != ${zrNumber(warriorMainhand.weapon.speed)} ||`,
    `        startingEquipmentText(mageIndex, "mainhand") != ${zrString(mageMainhand.item_id)} ||`,
    `        startingEquipmentInteger(mageIndex, "mainhand", "statInt") != ${zrInt(mageMainhand.stats.int)}) { return -5; }`,
    '    var index = 0;',
    '    while (index < classCount()) {',
    '        if (classIndex(classId(index)) != index) { return -6; }',
    '        index = index + 1;',
    '    }',
    '    return 1;',
    '}',
  ];
}

function catalogHash(catalog) {
  return hashText(JSON.stringify({ max_level: catalog.max_level, classes: catalog.classes }));
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
    invariant(existsSync(path), `${path} is missing; run npm run generate:m5-class-baseline-stats`);
    invariant(readFileSync(path, 'utf8') === content,
      `${path} is stale; run npm run generate:m5-class-baseline-stats`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, 'utf8');
}

function zrString(value) {
  return JSON.stringify(value);
}

function zrInt(value) {
  invariant(Number.isSafeInteger(value), `non-integer Zr value ${value}`);
  return String(value);
}

function zrNumber(value) {
  invariant(Number.isFinite(value), `non-finite Zr number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function hashText(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function hpFromStamina(stamina) {
  const value = Math.max(0, stamina);
  return Math.min(20, value) + Math.max(0, value - 20) * 10;
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
