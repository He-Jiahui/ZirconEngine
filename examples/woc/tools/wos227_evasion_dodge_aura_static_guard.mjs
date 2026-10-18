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

requireText(
  source('src/sim/content/classes.ts'),
  /evasion:[\s\S]*?class: 'rogue',[\s\S]*?learnLevel: 8,[\s\S]*?cost: 0,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 300,[\s\S]*?range: 0,[\s\S]*?school: 'physical',[\s\S]*?requiresTarget: false,[\s\S]*?offGcd: true,[\s\S]*?type: 'selfBuff', kind: 'buff_dodge', value: 0.5, duration: 15/,
  'source Evasion profile drifted',
);

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(generator, /'evasion'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Evasion is absent from the M4 projection');
requireText(zrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Evasion');

const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const evasion = m4.entries.find((entry) => entry.id === 'evasion');
if (m4.entries.length !== 117 || !evasion || evasion.index !== 99 ||
    evasion.definition.class !== 'rogue' || evasion.definition.learnLevel !== 8 ||
    evasion.definition.cost !== 0 || evasion.definition.castTime !== 0 ||
    evasion.definition.cooldown !== 300 || evasion.definition.range !== 0 ||
    evasion.definition.school !== 'physical' || evasion.definition.requiresTarget ||
    !evasion.definition.offGcd || evasion.definition.effects?.[0]?.type !== 'selfBuff' ||
    evasion.definition.effects?.[0]?.kind !== 'buff_dodge' ||
    evasion.definition.effects?.[0]?.value !== 0.5 ||
    evasion.definition.effects?.[0]?.duration !== 15) {
  throw new Error('Evasion source projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(
  world,
  /evasionAbilityCode[\s\S]*?evasionPayloadAbilityIsExact[\s\S]*?evasionProfileIsValid[\s\S]*?retainedDodgeAuraProfileIsValid[\s\S]*?retainedDodgeAuraBonus[\s\S]*?startOfflineEvasionCast/,
  'Evasion identity, retained dodge-aura boundary, or reducer is missing',
);
requireText(
  world,
  /retainedDodgeAuraProfileIsValid[\s\S]*?primalReflexesAbilityCode[\s\S]*?evasionAbilityCode[\s\S]*?6\.0[\s\S]*?15\.0/,
  'retained dodge-aura invariant must preserve Primal Reflexes and add Evasion only',
);
requireText(
  world,
  /startOfflineEvasionCast[\s\S]*?catalogAdmission[\s\S]*?evasionProfileIsValid[\s\S]*?setAbilityCooldownExpiration[\s\S]*?applyOfflineMotionAuraWithDetails/,
  'Evasion must use normal off-GCD admission, independent cooldown, and motion aura',
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?evasionAbilityCode\(\)[\s\S]*?startOfflineEvasionCast[\s\S]*?applySupportedCastCommand[\s\S]*?evasionPayloadAbilityIsExact[\s\S]*?startOfflineEvasionCast/,
  'Evasion slot and typed routes are missing',
);
requireText(
  world,
  /pub evasionRuntimeStateTest[\s\S]*?appendCastSlotCommand[\s\S]*?ageMotionAurasForEntity[\s\S]*?appendTypedCastCommandForTest[\s\S]*?entityLevels\[0\] = 7/,
  'Evasion state coverage is missing',
);

const fixture = read('scripts', 'woc_game', 'woc_m4_evasion_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/evasion_runtime_test_main.zr"')) {
  throw new Error('Evasion ZrVM project fixture is missing');
}

process.stdout.write(`WOS227 Evasion static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
