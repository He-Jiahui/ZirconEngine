import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import ts from 'typescript';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const M5_SCENARIOS = new Set([
  'g1b_xp_prestige',
  'talents_progression',
  'inventory_vendor',
  'l1_loot_distribution',
  'party_loot',
  'bank_round_trip',
  'market_round_trip',
  'player_trade',
  'quest_collect_turnin',
  'quest_kill_credit',
  'quest_link_abandon',
]);
const EXPECTED = {
  item_ids: [
    'baked_bread',
    'bandit_bandana',
    'cryptbone_helm',
    'elixir_of_the_bear',
    'gnarled_staff',
    'greyjaw_hide_boots',
    'minor_healing_potion',
    'roadwardens_helm',
    'spring_water',
    'wolf_fang',
    'worn_sword',
  ],
  quest_ids: ['q_boars', 'q_wolves'],
  mob_ids: ['forest_wolf'],
  npc_ids: ['the_merchant', 'trader_wilkes'],
  talent_option_ids: [
    'war_row_die_by_the_sword',
    'war_row_double_charge',
    'war_row_victory_rush',
  ],
  ability_ids: ['mortal_strike', 'overpower'],
  spec_ids: ['arms', 'fury'],
};
const EXPECTED_CLASS_STARTING_EQUIPMENT_ITEM_IDS = [
  'apprentice_robe',
  'eastbrook_buckler',
  'footpad_jerkin',
  'gnarled_staff',
  'recruit_tunic',
  'rusty_dagger',
  'rusty_hatchet',
  'training_mace',
  'worn_sword',
];
const EXPECTED_VENDOR_ITEM_IDS = [
  'baked_bread',
  'bronze_sickle',
  'copper_mining_pick',
  'felling_axe',
  'gathering_sickle',
  'handaxe',
  'iron_mining_pick',
  'ironbark_axe',
  'linen_pouch',
  'minor_healing_potion',
  'minor_mana_potion',
  'mithril_mining_pick',
  'roasted_boar',
  'silverleaf_sickle',
  'spring_water',
  'tough_jerky',
  'travelers_knapsack',
];
const FINAL_EXPECTED_ITEM_IDS = [...new Set([
  ...EXPECTED.item_ids,
  ...EXPECTED_VENDOR_ITEM_IDS,
  'apprentice_robe',
  'boar_hide',
  'eastbrook_buckler',
  'footpad_jerkin',
  'milepost_boots',
  'recruit_tunic',
  'rusty_dagger',
  'rusty_hatchet',
  'training_mace',
  'wolfhide_satchel',
])].sort();
// Item indexes are stored as WOS43/WOS44 inventory and equipment codes. Keep
// the original 14-entry projection as an immutable prefix and append newly
// required source identities so previously committed bytes retain their item.
const STABLE_ITEM_ID_ORDER = [
  'baked_bread',
  'bandit_bandana',
  'boar_hide',
  'cryptbone_helm',
  'elixir_of_the_bear',
  'gnarled_staff',
  'greyjaw_hide_boots',
  'milepost_boots',
  'minor_healing_potion',
  'roadwardens_helm',
  'spring_water',
  'wolf_fang',
  'wolfhide_satchel',
  'worn_sword',
  'apprentice_robe',
  'eastbrook_buckler',
  'footpad_jerkin',
  'recruit_tunic',
  'rusty_dagger',
  'rusty_hatchet',
  'training_mace',
  'roasted_boar',
  'tough_jerky',
  'minor_mana_potion',
  'linen_pouch',
  'travelers_knapsack',
  'copper_mining_pick',
  'iron_mining_pick',
  'mithril_mining_pick',
  'handaxe',
  'felling_axe',
  'ironbark_axe',
  'gathering_sickle',
  'bronze_sickle',
  'silverleaf_sickle',
];
const FINAL_EXPECTED_NPC_IDS = [
  'bursar_aldous_crane',
  'bursar_fernando',
  'bursar_petra_vell',
  'marshal_redbrook',
  'the_merchant',
  'trader_wilkes',
];
const FINAL_EXPECTED_ABILITY_IDS = ['bloodthirst', 'mortal_strike', 'overpower'];

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const parityCatalogPath = join(projectRoot, 'reference', 'current-head', 'parity_scenarios.json');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const scenariosSourcePath = 'tests/parity/scenarios.ts';
const marketSourcePath = 'src/sim/market.ts';
const outputPath = join(projectRoot, 'contracts', 'm5_content.json');
const extractorPath = join(scriptDirectory, 'm5_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const parity = JSON.parse(readFileSync(parityCatalogPath, 'utf8'));
  invariant(parity.source_commit === SOURCE_COMMIT, 'parity catalog source commit drifted');
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  invariant(sourceManifest.source_commit === SOURCE_COMMIT, 'source manifest commit drifted');

  const scenarios = parity.entries.filter((entry) => M5_SCENARIOS.has(entry.name));
  invariant(scenarios.length === M5_SCENARIOS.size, 'M5 parity scenario catalog is incomplete');
  const factories = new Map(scenarios.map((entry) => [entry.factory, entry.name]));
  const scenarioText = gitShow(scenariosSourcePath);
  const scenarioIdentity = gitBlobScenarioIdentity(scenarioText);
  verifyScenarioIdentity(sourceManifest, scenarioIdentity);
  const sourceFile = ts.createSourceFile(
    scenariosSourcePath,
    scenarioText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const uses = newUses();
  for (const statement of sourceFile.statements) {
    if (!ts.isFunctionDeclaration(statement) || !statement.name ||
        !factories.has(statement.name.text)) continue;
    extractScenarioUses(statement, sourceFile, factories.get(statement.name.text), uses);
  }
  for (const [kind, expected] of Object.entries(EXPECTED)) {
    const actual = [...uses[kind].keys()].sort();
    invariant(
      JSON.stringify(actual) === JSON.stringify(expected),
      `M5 ${kind} drifted: ${JSON.stringify(actual)}`,
    );
  }

  const scope = Object.fromEntries(
    Object.keys(EXPECTED).map((kind) => [kind, [...uses[kind].keys()].sort()]),
  );
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
    JSON.stringify(scope),
  ], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `M5 content extractor exited ${child.status}`);
  const extracted = JSON.parse(child.stdout);
  addDerivedSpecAbilityUses(uses, extracted);
  addDerivedItemUses(uses, extracted);
  addDerivedMobUses(uses, extracted);
  addDerivedMobItemUses(uses, extracted);
  addDerivedNpcUses(uses, extracted);
  addDerivedClassStartingEquipmentUses(uses, extracted);
  addDerivedVendorItemUses(uses, extracted);
  invariant(
    JSON.stringify([...uses.item_ids.keys()].sort()) === JSON.stringify(FINAL_EXPECTED_ITEM_IDS),
    `final M5 item scope drifted: ${JSON.stringify([...uses.item_ids.keys()].sort())}`,
  );
  invariant(
    JSON.stringify([...uses.npc_ids.keys()].sort()) === JSON.stringify(FINAL_EXPECTED_NPC_IDS),
    `final M5 NPC scope drifted: ${JSON.stringify([...uses.npc_ids.keys()].sort())}`,
  );
  invariant(
    JSON.stringify([...uses.ability_ids.keys()].sort()) ===
      JSON.stringify(FINAL_EXPECTED_ABILITY_IDS),
    `final M5 ability scope drifted: ${JSON.stringify([...uses.ability_ids.keys()].sort())}`,
  );

  const catalog = {
    schema_version: 2,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m5_content_codegen.mjs',
    scenarios: scenarios.map((entry) => entry.name).sort(),
    source_identities: sourceIdentities(scenarioText),
    reference_manifest_identity: {
      representation: 'git_blob_lf',
      ...scenarioIdentity,
    },
    market_cut: extractNumberConstant(gitShow(marketSourcePath), marketSourcePath, 'MARKET_CUT'),
    constants: extracted.constants,
    specs: entriesFromUses(
      uses.spec_ids,
      extracted.specs,
      'src/sim/content/talents_warrior.ts#warriorTalents.specs',
    ),
    items: entriesFromUses(
      uses.item_ids, extracted.items, 'src/sim/data.ts#ITEMS', STABLE_ITEM_ID_ORDER,
    ),
    quests: entriesFromUses(uses.quest_ids, extracted.quests, 'src/sim/data.ts#QUESTS'),
    mobs: entriesFromUses(uses.mob_ids, extracted.mobs, 'src/sim/data.ts#MOBS'),
    npcs: entriesFromUses(uses.npc_ids, extracted.npcs, 'src/sim/data.ts#NPCS'),
    talent_options: entriesFromUses(
      uses.talent_option_ids,
      extracted.talent_options,
      'src/sim/content/talent_rows.ts#ROW_TREES.warrior',
    ),
    abilities: entriesFromUses(
      uses.ability_ids,
      extracted.abilities,
      'src/sim/content/classes.ts#ABILITIES',
    ),
  };
  catalog.catalog_sha256 = hashText(JSON.stringify({
    scenarios: catalog.scenarios,
    market_cut: catalog.market_cut,
    constants: catalog.constants,
    specs: catalog.specs,
    items: catalog.items,
    quests: catalog.quests,
    mobs: catalog.mobs,
    npcs: catalog.npcs,
    talent_options: catalog.talent_options,
    abilities: catalog.abilities,
  }));

  const content = `${JSON.stringify(catalog, null, 2)}\n`;
  if (checkOnly) {
    invariant(existsSync(outputPath), 'm5_content.json is missing; run npm run generate:m5');
    invariant(readFileSync(outputPath, 'utf8') === content,
      'm5_content.json is stale; run npm run generate:m5');
  } else {
    writeFileSync(outputPath, content, 'utf8');
  }
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} M5 content: ` +
      `${catalog.items.length} items, ${catalog.quests.length} quests, ` +
      `${catalog.npcs.length} NPCs (${catalog.catalog_sha256.slice(0, 15)})\n`,
  );
}

function newUses() {
  return Object.fromEntries(Object.keys(EXPECTED).map((kind) => [kind, new Map()]));
}

function extractScenarioUses(declaration, sourceFile, scenario, uses) {
  const itemMethods = new Map([
    ['addItem', 0], ['equipItem', 0], ['useItem', 0], ['discardItem', 0],
    ['sellItem', 0], ['buyBackItem', 0], ['marketList', 0], ['buyItem', 1],
  ]);
  const questMethods = new Set(['acceptQuest', 'turnInQuest']);
  visit(declaration.body, (node) => {
    if (ts.isPropertyAccessExpression(node) && ts.isIdentifier(node.expression)) {
      if (node.expression.text === 'MOBS') addUse(uses.mob_ids, node.name.text, scenario);
      if (node.expression.text === 'QUESTS') addUse(uses.quest_ids, node.name.text, scenario);
    }
    if (ts.isPropertyAssignment(node) && propertyName(node.name) === 'itemId' &&
        ts.isStringLiteralLike(node.initializer)) {
      addUse(uses.item_ids, node.initializer.text, scenario);
    }
    if (ts.isPropertyAssignment(node) && propertyName(node.name) === 'spec' &&
        ts.isStringLiteralLike(node.initializer)) {
      addUse(uses.spec_ids, node.initializer.text, scenario);
    }
    if (ts.isPropertyAssignment(node) && propertyName(node.name) === 'rows' &&
        ts.isObjectLiteralExpression(node.initializer)) {
      for (const row of node.initializer.properties) {
        if (ts.isPropertyAssignment(row) && ts.isStringLiteralLike(row.initializer)) {
          addUse(uses.talent_option_ids, row.initializer.text, scenario);
        }
      }
    }
    if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name) &&
        /questId$/u.test(node.name.text) && node.initializer &&
        ts.isStringLiteralLike(node.initializer)) {
      addUse(uses.quest_ids, node.initializer.text, scenario);
    }
    if (ts.isBinaryExpression(node) && node.operatorToken.kind === ts.SyntaxKind.EqualsEqualsEqualsToken) {
      const left = node.left.getText(sourceFile);
      if (left.endsWith('.templateId') && ts.isStringLiteralLike(node.right)) {
        addUse(uses.npc_ids, node.right.text, scenario);
      }
    }
    if (!ts.isCallExpression(node) || !ts.isPropertyAccessExpression(node.expression)) return;
    const method = node.expression.name.text;
    if (itemMethods.has(method)) {
      const argument = node.arguments[itemMethods.get(method)];
      if (argument && ts.isStringLiteralLike(argument)) {
        addUse(uses.item_ids, argument.text, scenario);
      }
    }
    if (questMethods.has(method) && node.arguments[0] && ts.isStringLiteralLike(node.arguments[0])) {
      addUse(uses.quest_ids, node.arguments[0].text, scenario);
    }
    if (method === 'setSpec' && node.arguments[0] && ts.isStringLiteralLike(node.arguments[0])) {
      addUse(uses.spec_ids, node.arguments[0].text, scenario);
    }
    if (method === 'saveLoadout' && node.arguments[1] &&
        ts.isArrayLiteralExpression(node.arguments[1])) {
      for (const ability of node.arguments[1].elements) {
        if (ts.isStringLiteralLike(ability)) addUse(uses.ability_ids, ability.text, scenario);
      }
    }
  });
}

function addDerivedNpcUses(uses, extracted) {
  for (const [questId, quest] of Object.entries(extracted.quests)) {
    for (const scenario of uses.quest_ids.get(questId) ?? []) {
      addUse(uses.npc_ids, quest.giverNpcId, scenario);
    }
  }
  for (const bankerId of extracted.banker_ids) {
    addUse(uses.npc_ids, bankerId, 'bank_round_trip');
  }
}

function addDerivedSpecAbilityUses(uses, extracted) {
  for (const [specId, spec] of Object.entries(extracted.specs)) {
    invariant(typeof spec.signature === 'string', `M5 spec ${specId} has no signature ability`);
    for (const scenario of uses.spec_ids.get(specId) ?? []) {
      addUse(uses.ability_ids, spec.signature, scenario);
    }
  }
}

function addDerivedItemUses(uses, extracted) {
  for (const [questId, quest] of Object.entries(extracted.quests)) {
    for (const objective of quest.objectives ?? []) {
      if (typeof objective.itemId !== 'string') continue;
      for (const scenario of uses.quest_ids.get(questId) ?? []) {
        addUse(uses.item_ids, objective.itemId, scenario);
      }
    }
  }
}

function addDerivedMobUses(uses, extracted) {
  for (const [questId, quest] of Object.entries(extracted.quests)) {
    for (const objective of quest.objectives ?? []) {
      if (typeof objective.targetMobId !== 'string') continue;
      for (const scenario of uses.quest_ids.get(questId) ?? []) {
        addUse(uses.mob_ids, objective.targetMobId, scenario);
      }
    }
  }
}

function addDerivedMobItemUses(uses, extracted) {
  for (const [mobId, mob] of Object.entries(extracted.mobs)) {
    for (const loot of mob.loot ?? []) {
      if (typeof loot.itemId !== 'string') continue;
      for (const scenario of uses.mob_ids.get(mobId) ?? []) {
        addUse(uses.item_ids, loot.itemId, scenario);
      }
    }
  }
}

function addDerivedClassStartingEquipmentUses(uses, extracted) {
  invariant(
    JSON.stringify(extracted.class_starting_equipment_item_ids) ===
      JSON.stringify(EXPECTED_CLASS_STARTING_EQUIPMENT_ITEM_IDS),
    `class starting equipment scope drifted: ${JSON.stringify(
      extracted.class_starting_equipment_item_ids,
    )}`,
  );
  for (const itemId of extracted.class_starting_equipment_item_ids) {
    addUse(uses.item_ids, itemId, 'class_starting_equipment');
  }
}

function addDerivedVendorItemUses(uses, extracted) {
  invariant(
    JSON.stringify(extracted.vendor_item_ids) === JSON.stringify(EXPECTED_VENDOR_ITEM_IDS),
    `vendor item scope drifted: ${JSON.stringify(extracted.vendor_item_ids)}`,
  );
  for (const itemId of extracted.vendor_item_ids) {
    addUse(uses.item_ids, itemId, 'inventory_vendor');
  }
}

function entriesFromUses(useMap, definitions, sourceOwner, stableOrder) {
  const ids = stableOrder ?? [...useMap.keys()].sort();
  if (stableOrder) {
    invariant(
      JSON.stringify([...stableOrder].sort()) === JSON.stringify([...useMap.keys()].sort()),
      'M5 stable item order does not cover the extracted scope',
    );
  }
  return ids.map((id) => {
    invariant(id in definitions, `missing extracted M5 definition: ${id}`);
    const entry = { id, scenarios: [...useMap.get(id)].sort() };
    if (sourceOwner) entry.source_owner = `${sourceOwner}.${id}`;
    entry.definition = definitions[id];
    return entry;
  });
}

function addUse(map, id, scenario) {
  const scenarios = map.get(id) ?? new Set();
  scenarios.add(scenario);
  map.set(id, scenarios);
}

function sourceIdentities(scenarioText) {
  const paths = [
    scenariosSourcePath,
    'src/sim/data.ts',
    'src/sim/content/classes.ts',
    'src/sim/content/items.ts',
    'src/sim/content/talents.ts',
    'src/sim/content/talent_rows.ts',
    'src/sim/content/talents_warrior.ts',
    'src/sim/content/warrior_rows.ts',
    'src/sim/content/zone1.ts',
    'src/sim/content/zone2.ts',
    'src/sim/content/zone3.ts',
    'src/sim/bank.ts',
    marketSourcePath,
    'src/sim/types.ts',
  ];
  return {
    representation: 'git_blob_lf',
    files: paths.map((path) => textIdentity(
      path,
      path === scenariosSourcePath ? scenarioText : gitShow(path),
    )),
  };
}

function verifyScenarioIdentity(sourceManifest, scenarioIdentity) {
  const recorded = sourceManifest.identities.parity_sources.files.find(
    (entry) => entry.path === scenariosSourcePath,
  );
  invariant(recorded, 'source manifest is missing the parity scenario identity');
  invariant(recorded.bytes === scenarioIdentity.bytes &&
    recorded.sha256 === scenarioIdentity.sha256,
  'source manifest parity scenario is not the pinned Git blob');
}

function gitBlobScenarioIdentity(scenarioText) {
  return textIdentity(scenariosSourcePath, scenarioText);
}

function extractNumberConstant(sourceText, sourcePath, name) {
  const sourceFile = ts.createSourceFile(
    sourcePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name &&
          declaration.initializer && ts.isNumericLiteral(declaration.initializer)) {
        return Number(declaration.initializer.text);
      }
    }
  }
  throw new Error(`missing numeric constant ${sourcePath}#${name}`);
}

function propertyName(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
  return name.getText();
}

function visit(node, callback) {
  if (!node) return;
  callback(node);
  ts.forEachChild(node, (child) => visit(child, callback));
}

function hashText(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function textIdentity(sourcePath, value) {
  return {
    path: sourcePath,
    bytes: Buffer.byteLength(value, 'utf8'),
    sha256: hashText(value),
  };
}

function gitShow(sourcePath) {
  return execFileSync(
    'git',
    ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`],
    { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 },
  );
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
