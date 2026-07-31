import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = path.resolve(root, '..', '..');
const sourceRoot = path.resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), 'utf8');
const source = (file) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${file}`], { encoding: 'utf8' },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const classes = source('src/sim/content/classes.ts');
const dispatch = source('src/sim/combat/effect_dispatch.ts');
const sim = source('src/sim/sim.ts');
const types = source('src/sim/types.ts');
const start = classes.indexOf('  faerie_fire: {');
const end = classes.indexOf('  hibernate: {', start);
if (start < 0 || end < start) throw new Error('source Faerie Fire block is missing');
const faerieFire = classes.slice(start, end);
for (const needle of [
  "name: 'Witchlight'", "class: 'druid'", 'learnLevel: 18', 'cost: 30',
  'castTime: 0', 'cooldown: 0', 'range: 30', "school: 'nature'",
  'requiresTarget: true', "type: 'faerieFire', duration: 40",
]) {
  if (!faerieFire.includes(needle)) throw new Error(`source Faerie Fire drifted: ${needle}`);
}
requireText(dispatch, /case 'faerieFire':[\s\S]*?kind: 'faerie_fire',[\s\S]*?remaining: eff\.duration,[\s\S]*?value: 0,[\s\S]*?ctx\.enterCombat\(p, target\)/,
  'source Faerie Fire aura application drifted');
if (!types.includes('FAERIE_FIRE_ARMOR_PCT = 0.1') ||
    !sim.includes("else if (a.kind === 'faerie_fire')") ||
    !sim.includes('reductionPct = Math.max(reductionPct, FAERIE_FIRE_ARMOR_PCT)')) {
  throw new Error('source Faerie Fire max-combined armor reduction drifted');
}

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/bash',[\s\S]*?'faerie_fire'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Faerie Fire projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'faerie_fire',
);
if (!entry || entry.index !== 70 || entry.definition.cost !== 30 ||
    entry.definition.cooldown !== 0 || entry.definition.range !== 30 ||
    entry.definition.effects?.[0]?.type !== 'faerieFire' ||
    entry.definition.effects[0].duration !== 40) {
  throw new Error('M4 Faerie Fire projection drifted');
}

const cc = JSON.parse(read('reference', 'current-head', 'cc_contract.json'));
if (cc.motion_kind_codes?.faerie_fire !== 11) {
  throw new Error('Faerie Fire motion aura kind is missing');
}
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /faerieFireAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("faerie_fire"\)/,
  'Faerie Fire catalog identity is missing');
requireText(world, /faerieFireTargetIndex[\s\S]*?m4AbilityCatalog\.metric[\s\S]*?range[\s\S]*?targetIndex : -1/,
  'Faerie Fire source range gate is missing');
requireText(world, /startOfflineFaerieFireCast[\s\S]*?entityCastGcdRemaining[\s\S]*?faerieFireTargetIndex[\s\S]*?applyOfflineFaerieFire[\s\S]*?entityInCombat/,
  'Faerie Fire targeted aura reducer is missing');
requireText(world, /applyOfflineFaerieFire[\s\S]*?faerieFireAbilityCode\(\)[\s\S]*?motionAuraKindCode\("faerie_fire"\)/,
  'Faerie Fire aura projection is missing');
requireText(world, /effectiveOfflineArmor[\s\S]*?sunderArmorReduction[\s\S]*?faerieFireArmorReduction[\s\S]*?faerieFireReduction > reduction/,
  'Faerie Fire/Sunder maximum reduction rule is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?faerieFireAbilityCode\(\)[\s\S]*?startOfflineFaerieFireCast/,
  'Faerie Fire action-slot routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?faerieFirePayloadAbilityIsExact[\s\S]*?startOfflineFaerieFireCast/,
  'Faerie Fire typed routing is missing');
requireText(world, /pub faerieFireCommandStateTest\(\): int[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?effectiveOfflineArmor[\s\S]*?decodeState/,
  'Faerie Fire state regression coverage is missing');
requireText(world, /if \(faerieFireCommandStateTest\(\) != 1\) \{[\s\S]*?return -124;/,
  'world selfTest must execute Faerie Fire');

process.stdout.write(`WOS130 Faerie Fire static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
