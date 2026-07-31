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
const start = classes.indexOf('  pounce: {');
const end = classes.indexOf('  insect_swarm: {', start);
if (start < 0 || end < start) throw new Error('source Pounce block is missing');
const pounce = classes.slice(start, end);
for (const needle of [
  "name: 'Slinkstrike'", "class: 'druid'", 'learnLevel: 18', 'cost: 50',
  'castTime: 0', 'cooldown: 0', 'range: 8', "school: 'physical'",
  'requiresTarget: true', 'awardsCombo: 1', "requiresForm: 'cat'",
  'requiresStealth: true', "type: 'stun', duration: 2",
]) {
  if (!pounce.includes(needle)) throw new Error(`source Pounce drifted: ${needle}`);
}
const stunStart = dispatch.indexOf("case 'stun':");
const stunEnd = dispatch.indexOf("case 'incapacitate':", stunStart);
if (stunStart < 0 || stunEnd < stunStart || !dispatch.slice(stunStart, stunEnd).includes("kind: 'stun'") ||
    dispatch.slice(stunStart, stunEnd).includes('awardCombo')) {
  throw new Error('source Pounce stun/combo execution boundary drifted');
}

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/dash',[\s\S]*?'pounce'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Pounce projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'pounce',
);
if (!entry || entry.index !== 73 || entry.definition.cost !== 50 ||
    entry.definition.awardsCombo !== 1 || entry.definition.requiresForm !== 'cat' ||
    entry.definition.requiresStealth !== true || entry.definition.effects?.[0]?.type !== 'stun' ||
    entry.definition.effects[0].duration !== 2) {
  throw new Error('M4 Pounce projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /pounceAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("pounce"\)/,
  'Pounce catalog identity is missing');
requireText(world, /pounceTargetIndex[\s\S]*?m4AbilityCatalog\.metric[\s\S]*?range[\s\S]*?targetIndex : -1/,
  'Pounce source range gate is missing');
requireText(world, /startOfflinePounceCast[\s\S]*?offlineProwlIsActive[\s\S]*?catalogAdmission[\s\S]*?clearOfflineProwl[\s\S]*?applyOfflinePounceStun[\s\S]*?entityInCombat/,
  'Pounce Cat/stealth stun reducer is missing');
requireText(world, /applyOfflinePounceStun[\s\S]*?pounceAbilityCode\(\)[\s\S]*?motionAuraKindCode\("stun"\)/,
  'Pounce stun aura projection is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?pounceAbilityCode\(\)[\s\S]*?startOfflinePounceCast/,
  'Pounce action-slot routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?pouncePayloadAbilityIsExact[\s\S]*?startOfflinePounceCast/,
  'Pounce typed routing is missing');
requireText(world, /pub pounceCommandStateTest\(\): int[\s\S]*?form_cat[\s\S]*?entityProwlRemaining[\s\S]*?entityComboPoints\[0\] != 0[\s\S]*?appendTypedCastTargetCommandForTest/,
  'Pounce state regression must retain source no-combo behavior');
requireText(world, /if \(pounceCommandStateTest\(\) != 1\) \{[\s\S]*?return -127;/,
  'world selfTest must execute Pounce');

process.stdout.write(`WOS133 Pounce static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
