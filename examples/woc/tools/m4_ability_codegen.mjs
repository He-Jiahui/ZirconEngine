import { spawnSync, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import ts from 'typescript';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const M4_SCENARIOS = new Set([
  'solo_warrior',
  'solo_mage',
  'solo_rogue',
  'affix_mob',
  'mob_swing_affixes',
  'hit_rating_heroic_geared',
  'hit_rating_heroic_ungeared',
  'multi_class_frenzy',
  'multi_class_heal',
  'paladin_consecration',
  'drowned_litany',
  'nythraxis_full_pull',
  'c3_aura_runner',
  'c4a_casting_lifecycle',
  'c4b_effect_dispatch',
  'c5_auto_attack',
]);
// Retained by the WOC offline replay, although no pinned parity scenario
// invokes it directly.
const WOC_RETAINED_ABILITY_IDS = [
  'backstab',
  'gouge',
  // WOS84 keeps the two current-head pure DoTs even though the pinned M4
  // parity scenarios do not cast them. Their projectile landing snapshots
  // need the same generated source contract as the scenario-owned abilities.
  'serpent_sting',
  'shadow_word_pain',
  'renew',
  'power_word_shield',
  'smite',
  'mind_blast',
  'heal',
  'flash_heal',
  'mind_flay',
  'lightning_bolt',
  'healing_wave',
  'earth_shock',
  'frost_shock',
  'flame_shock',
  'flametongue_weapon',
  'frostbrand_weapon',
  'ghost_wolf',
  'stormstrike',
  'shadow_bolt',
  'immolate',
  'corruption',
  'life_tap',
  'curse_of_agony',
  'searing_pain',
  'shadowburn',
  'demon_skin',
  // WOS109 closes the source-positioned Warlock channel through the existing
  // castAt payload ABI and retained channel lifecycle.
  'rain_of_fire',
  // WOS110 retains the source-owned Immolate consumption and delayed Fire
  // projectile resolution through the existing hostile-spell envelope.
  'conflagrate',
  // WOS111 retains the source periodic Shadow leech through the existing
  // pure-DoT snapshot and Eastbrook healing-threat paths.
  'siphon_life',
  // WOS112 consumes the oldest matching friendly HoT before resolving its
  // direct-heal range and critical path.
  'swiftmend',
  // WOS113 retains the Druid's four-rank Nature projectile through the
  // existing hard-cast and delayed hostile-spell lifecycle.
  'wrath',
  // WOS114 retains the Druid's four-rank friendly hard cast through the
  // existing direct-heal and effective-healing-threat lifecycle.
  'healing_touch',
  // WOS115 retains the Druid's one-rank Arcane hard cast through the existing
  // delayed hostile-spell projectile lifecycle.
  'starfire',
  // WOS116 retains the Druid's two-rank root and periodic Nature spell through
  // the existing hostile cast, motion-aura and periodic damage lifecycles.
  'entangling_roots',
  // WOS117 retains the Bear-form queued melee strike through the existing
  // on-next-swing resource, damage, threat and cooldown lifecycle.
  'maul',
  // WOS118 retains the Bear-form taunt through the existing forced-target and
  // threat settlement once source form admission has succeeded.
  'growl',
  // WOS119 retains the Bear-form hostile AP reduction with one source aura
  // refresh row per Eastbrook target and the existing mob-swing AP read.
  'demoralizing_roar',
  // WOS120 retains the Cat-form, out-of-combat stealth toggle through the
  // source self-buff envelope and the offline idle-aggro projection.
  'prowl',
  // WOS121 retains the Cat-form stealth weapon-strike and bleed opener.
  'rake',
  // WOS122 retains the Cat-form baseline weapon-strike builder.
  'claw',
  // WOS123 retains the Cat-form combo finisher through the existing physical
  // finisher, Prowl reveal and combo-consumption lifecycles.
  'ferocious_bite',
  // WOS124 retains the Bear-form physical area strike and its source threat
  // multiplier through the existing deterministic combat transaction.
  'swipe',
  // WOS125 retains Regrowth's direct-heal plus zero-extra-scaling HoT order.
  'regrowth',
  // WOS126 retains the off-GCD, form-usable self armor cooldown.
  'barkskin',
  // WOS127 retains the off-GCD, form-usable self dodge cooldown.
  'primal_reflexes',
  // WOS128 retains the Bear-only off-GCD Rage generator.
  'enrage',
  // WOS129 retains the Bear-only targeted stun.
  'bash',
  // WOS130 retains the target armor-debuff aura and its max-combined Sunder rule.
  'faerie_fire',
  // WOS131 retains the Nature hard-cast and damage-breakable incapacitate.
  'hibernate',
  // WOS132 retains the Cat-only, off-GCD movement-speed cooldown.
  'dash',
  // WOS133 retains the Cat stealth stun opener's source execution behavior.
  'pounce',
  // WOS134 retains the Druid's instant Nature pure-DoT projectile.
  'insect_swarm',
  // WOS135 retains the Cat-only, short-lived attack-power self buff.
  'tigers_fury',
  // WOS136 retains the Cat-form, combo-consuming physical pure DoT.
  'rip',
  // WOS137 retains the Druid's positioned Nature damage channel.
  'hurricane',
  // WOS138 retains the Druid's target interrupt and school lockout.
  'skull_bash',
];
const EXPECTED_ABILITY_COUNT = 79;

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const parityCatalogPath = join(projectRoot, 'reference', 'current-head', 'parity_scenarios.json');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const scenariosSourcePath = 'tests/parity/scenarios.ts';
const abilitiesSourcePath = 'src/sim/content/classes.ts';
const outputPath = join(projectRoot, 'contracts', 'm4_abilities.json');
const extractorPath = join(scriptDirectory, 'm4_ability_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);

  const parity = JSON.parse(readFileSync(parityCatalogPath, 'utf8'));
  invariant(parity.source_commit === SOURCE_COMMIT, 'parity catalog source commit drifted');
  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  invariant(sourceManifest.source_commit === SOURCE_COMMIT, 'source manifest commit drifted');
  const scenarios = parity.entries.filter((entry) => M4_SCENARIOS.has(entry.name));
  invariant(scenarios.length === M4_SCENARIOS.size, 'M4 parity scenario catalog is incomplete');
  const factories = new Map();
  for (const scenario of scenarios) {
    const names = factories.get(scenario.factory) ?? [];
    names.push(scenario.name);
    factories.set(scenario.factory, names);
  }

  const sourceText = gitShow(scenariosSourcePath);
  const scenarioGitIdentity = textIdentity(scenariosSourcePath, sourceText);
  const scenarioWindowsIdentity = textIdentity(
    scenariosSourcePath,
    sourceText.replace(/\r?\n/gu, '\r\n'),
  );
  const recordedScenarioIdentity = sourceManifest.identities.parity_sources.files.find(
    (entry) => entry.path === scenariosSourcePath,
  );
  invariant(recordedScenarioIdentity, 'source manifest is missing the parity scenario identity');
  const matchingScenarioRepresentation = [
    { name: 'git_blob_lf', identity: scenarioGitIdentity },
    { name: 'git_blob_crlf', identity: scenarioWindowsIdentity },
  ].find(
    ({ identity }) =>
      recordedScenarioIdentity.bytes === identity.bytes &&
      recordedScenarioIdentity.sha256 === identity.sha256,
  );
  invariant(
    matchingScenarioRepresentation,
    'source manifest parity scenario is not a known newline representation of the pinned Git blob',
  );
  const sourceFile = ts.createSourceFile(
    scenariosSourcePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const uses = new Map();
  for (const statement of sourceFile.statements) {
    if (!ts.isFunctionDeclaration(statement) || !statement.name ||
        !factories.has(statement.name.text)) continue;
    const abilityIds = extractCastAbilityIds(statement, sourceFile);
    for (const id of abilityIds) {
      const owners = uses.get(id) ?? new Set();
      for (const scenario of factories.get(statement.name.text)) owners.add(scenario);
      uses.set(id, owners);
    }
  }
  for (const id of WOC_RETAINED_ABILITY_IDS) {
    if (!uses.has(id)) uses.set(id, new Set());
  }
  invariant(uses.size === EXPECTED_ABILITY_COUNT,
    `M4 ability scope contains ${uses.size} entries, expected ${EXPECTED_ABILITY_COUNT}`);

  const ids = [...uses.keys()];
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
    `wocgit:///${abilitiesSourcePath}`,
    ...ids,
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  invariant(child.status === 0, child.stderr || `ability extractor exited ${child.status}`);
  const definitions = JSON.parse(child.stdout);
  invariant(definitions.length === ids.length, 'ability extractor returned the wrong row count');

  const entries = ids.map((id, index) => {
    const definition = definitions[index];
    invariant(definition.id === id, `ability extractor order drifted at ${id}`);
    return {
      index,
      id,
      scenarios: [...uses.get(id)].sort(),
      source_owner: `src/sim/content/classes.ts#ABILITIES.${id}`,
      definition,
    };
  });
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m4_ability_codegen.mjs',
    source_identities: {
      representation: 'git_blob_lf',
      scenarios: scenarioGitIdentity,
      abilities: textIdentity(abilitiesSourcePath, gitShow(abilitiesSourcePath)),
    },
    reference_manifest_identity: {
      representation: matchingScenarioRepresentation.name,
      ...recordedScenarioIdentity,
    },
    catalog_sha256: hashText(JSON.stringify(entries)),
    entries,
  };
  const content = `${JSON.stringify(document, null, 2)}\n`;
  if (checkOnly) {
    invariant(existsSync(outputPath), 'm4_abilities.json is missing; run npm run generate');
    invariant(readFileSync(outputPath, 'utf8') === content,
      'm4_abilities.json is stale; run npm run generate');
  } else {
    writeFileSync(outputPath, content, 'utf8');
  }
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} ${entries.length} WOC M4 abilities ` +
      `(${document.catalog_sha256.slice(0, 15)})\n`,
  );
}

function extractCastAbilityIds(declaration, sourceFile) {
  const stringArrays = new Map();
  visit(declaration.body, (node) => {
    if (!ts.isVariableDeclaration(node) || !ts.isIdentifier(node.name) ||
        !node.initializer || !ts.isArrayLiteralExpression(node.initializer)) return;
    const values = node.initializer.elements.filter(ts.isStringLiteralLike).map((entry) => entry.text);
    if (values.length === node.initializer.elements.length) stringArrays.set(node.name.text, values);
  });
  const result = [];
  visit(declaration.body, (node) => {
    if (!ts.isCallExpression(node) || !ts.isPropertyAccessExpression(node.expression) ||
        node.expression.name.text !== 'castAbility' || node.arguments.length === 0) return;
    const argument = node.arguments[0];
    if (ts.isStringLiteralLike(argument)) {
      result.push(argument.text);
      return;
    }
    if (ts.isElementAccessExpression(argument) && ts.isIdentifier(argument.expression) &&
        stringArrays.has(argument.expression.text)) {
      result.push(...stringArrays.get(argument.expression.text));
      return;
    }
    throw new Error(
      `${declaration.name?.text ?? '<anonymous>'} has a dynamic castAbility source: ` +
        argument.getText(sourceFile),
    );
  });
  return [...new Set(result)];
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
    { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  );
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
