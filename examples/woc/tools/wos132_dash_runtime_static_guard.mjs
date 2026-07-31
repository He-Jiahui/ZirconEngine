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
const start = classes.indexOf('  dash: {');
const end = classes.indexOf('  pounce: {', start);
if (start < 0 || end < start) throw new Error('source Dash block is missing');
const dash = classes.slice(start, end);
for (const needle of [
  "name: 'Dash'", "class: 'druid'", 'learnLevel: 18', 'cost: 0',
  'castTime: 0', 'cooldown: 60', 'range: 0', "school: 'physical'",
  'requiresTarget: false', 'offGcd: true', "requiresForm: 'cat'",
  "type: 'selfBuff', kind: 'buff_speed', value: 1.5, duration: 15",
]) {
  if (!dash.includes(needle)) throw new Error(`source Dash drifted: ${needle}`);
}
if (!dispatch.includes("case 'selfBuff':") || !dispatch.includes("ability.id === 'ghost_wolf'")) {
  throw new Error('source self-buff/toggle boundary drifted');
}

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/hibernate',[\s\S]*?'dash'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Dash projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'dash',
);
if (!entry || entry.index !== 72 || entry.definition.cost !== 0 ||
    entry.definition.cooldown !== 60 || entry.definition.offGcd !== true ||
    entry.definition.requiresForm !== 'cat' ||
    entry.definition.effects?.[0]?.kind !== 'buff_speed' ||
    entry.definition.effects[0].value !== 1.5 || entry.definition.effects[0].duration !== 15) {
  throw new Error('M4 Dash projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /dashAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("dash"\)/,
  'Dash catalog identity is missing');
requireText(world, /dashProfileIsValid[\s\S]*?offGcd[\s\S]*?buff_speed[\s\S]*?1\.5[\s\S]*?15\.0/,
  'Dash source profile is missing');
requireText(world, /startOfflineDashCast[\s\S]*?entityCastingAbility[\s\S]*?abilityCooldownExpiresAt[\s\S]*?catalogAdmission[\s\S]*?setAbilityCooldownExpiration[\s\S]*?motionAuraKindCode\("buff_speed"\)/,
  'Dash Cat-only off-GCD reducer is missing');
requireText(world, /ghostWolfMotionStateIsValid[\s\S]*?dashAbilityCode\(\)[\s\S]*?entityMotionAuraRemaining[\s\S]*?15\.0[\s\S]*?1\.5/,
  'Dash speed-aura state validation is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?dashAbilityCode\(\)[\s\S]*?startOfflineDashCast/,
  'Dash action-slot routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?dashPayloadAbilityIsExact[\s\S]*?startOfflineDashCast/,
  'Dash typed routing is missing');
requireText(world, /pub dashCommandStateTest\(\): int[\s\S]*?form_cat[\s\S]*?appendTypedCastCommandForTest[\s\S]*?retainedPlayerMovementSpeedMultiplier[\s\S]*?appendCastSlotCommand/,
  'Dash state regression coverage is missing');
requireText(world, /if \(dashCommandStateTest\(\) != 1\) \{[\s\S]*?return -126;/,
  'world selfTest must execute Dash');

process.stdout.write(`WOS132 Dash static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
