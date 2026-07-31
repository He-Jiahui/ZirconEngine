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
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
const dispatch = source('src/sim/combat/effect_dispatch.ts');
const entity = source('src/sim/entity.ts');
const start = classes.indexOf('  tigers_fury: {');
const end = classes.indexOf('  rip: {', start);
if (start < 0 || end < start) throw new Error("source Tiger's Fury block is missing");
const tigersFury = classes.slice(start, end);
for (const needle of [
  "name: 'Wolfsblood'", "class: 'druid'", 'learnLevel: 20', 'cost: 30',
  'castTime: 0', 'cooldown: 30', 'range: 0', "school: 'physical'",
  'requiresTarget: false', "requiresForm: 'cat'",
  "type: 'selfBuff', kind: 'buff_ap', value: 40, duration: 6",
]) {
  if (!tigersFury.includes(needle)) throw new Error(`source Tiger's Fury drifted: ${needle}`);
}
requireText(lifecycle, /!ability\.offGcd && p\.gcdRemaining > 0/,
  "source Tiger's Fury global-cooldown gate drifted");
requireText(dispatch, /case 'selfBuff':[\s\S]*?ctx\.applyAura/,
  "source self-buff aura dispatch drifted");
requireText(entity, /a\.kind === 'buff_ap'\) bonusAp \+= a\.value/,
  "source effective attack-power aura projection drifted");

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
const ccGenerator = read('tools', 'cc_contract_codegen.mjs');
if (!/insect_swarm',[\s\S]*?'tigers_fury'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error("M4 Tiger's Fury projection scope is missing");
}
if (!ccGenerator.includes("id: 'tigers_fury'") ||
    !ccGenerator.includes('buff_ap: 12')) {
  throw new Error("Tiger's Fury CC aura projection is missing");
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'tigers_fury',
);
if (!entry || entry.index !== 75 || entry.definition.cost !== 30 ||
    entry.definition.cooldown !== 30 || entry.definition.requiresForm !== 'cat' ||
    entry.definition.effects?.[0]?.type !== 'selfBuff' ||
    entry.definition.effects[0].kind !== 'buff_ap' ||
    entry.definition.effects[0].value !== 40 || entry.definition.effects[0].duration !== 6) {
  throw new Error("M4 Tiger's Fury projection drifted");
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /tigersFuryAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("tigers_fury"\)/,
  "Tiger's Fury catalog identity is missing");
requireText(world, /tigersFuryProfileIsValid[\s\S]*?buff_ap[\s\S]*?40\.0[\s\S]*?6\.0/,
  "Tiger's Fury source profile is missing");
requireText(world, /startOfflineTigersFuryCast[\s\S]*?entityCastGcdRemaining[\s\S]*?abilityCooldownExpiresAt[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?setAbilityCooldownExpiration[\s\S]*?motionAuraKindCode\("buff_ap"\)/,
  "Tiger's Fury Cat-form aura reducer is missing");
requireText(world, /tigersFuryAttackPowerBonus[\s\S]*?tigersFuryAbilityCode[\s\S]*?value != 40\.0/,
  "Tiger's Fury effective attack-power profile is missing");
requireText(world, /effectiveOfflineAttackPower[\s\S]*?tigersFuryAttackPowerBonus/,
  "Tiger's Fury attack-power bonus is not connected to combat math");
requireText(world, /prepareOfflineAutoActor[\s\S]*?effectiveOfflineAttackPower/,
  "Tiger's Fury bonus is not connected to retained auto attacks");
requireText(world, /applySupportedCastSlotCommand[\s\S]*?tigersFuryAbilityCode\(\)[\s\S]*?startOfflineTigersFuryCast/,
  "Tiger's Fury action-slot routing is missing");
requireText(world, /applySupportedCastCommand[\s\S]*?tigersFuryPayloadAbilityIsExact[\s\S]*?startOfflineTigersFuryCast/,
  "Tiger's Fury typed routing is missing");
requireText(world, /pub tigersFuryCommandStateTest\(\): int[\s\S]*?form_cat[\s\S]*?appendTypedCastCommandForTest[\s\S]*?effectiveOfflineAttackPower[\s\S]*?appendCastSlotCommand/,
  "Tiger's Fury state regression coverage is missing");
requireText(world, /if \(tigersFuryCommandStateTest\(\) != 1\) \{[\s\S]*?return -129;/,
  "world selfTest must execute Tiger's Fury");

process.stdout.write(`WOS135 Tiger's Fury static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
