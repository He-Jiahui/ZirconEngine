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
requireText(
  classes,
  /berserker_rage:[\s\S]*?class: 'warrior',[\s\S]*?learnLevel: 10,[\s\S]*?cost: 0,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 30,[\s\S]*?range: 0,[\s\S]*?school: 'physical',[\s\S]*?requiresTarget: false,[\s\S]*?offGcd: true,[\s\S]*?type: 'gainResource', amount: 20/,
  'source Berserker Rage profile drifted',
);

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(generator, /'berserker_rage'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Berserker Rage is absent from the M4 projection');
requireText(zrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Berserker Rage');

const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const berserkerRage = m4.entries.find((entry) => entry.id === 'berserker_rage');
if (m4.entries.length !== 117 || !berserkerRage || berserkerRage.index !== 97 ||
    berserkerRage.definition.class !== 'warrior' || berserkerRage.definition.learnLevel !== 10 ||
    berserkerRage.definition.cost !== 0 || berserkerRage.definition.castTime !== 0 ||
    berserkerRage.definition.cooldown !== 30 || berserkerRage.definition.range !== 0 ||
    berserkerRage.definition.school !== 'physical' || berserkerRage.definition.requiresTarget ||
    !berserkerRage.definition.offGcd ||
    berserkerRage.definition.effects?.[0]?.type !== 'gainResource' ||
    berserkerRage.definition.effects?.[0]?.amount !== 20) {
  throw new Error('Berserker Rage source projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(
  world,
  /berserkerRageAbilityCode[\s\S]*?berserkerRagePayloadAbilityIsExact[\s\S]*?berserkerRageProfileIsValid[\s\S]*?startOfflineBerserkerRageCast/,
  'Berserker Rage identity or reducer is missing',
);
requireText(
  world,
  /startOfflineBerserkerRageCast[\s\S]*?catalogAdmission[\s\S]*?berserkerRageProfileIsValid[\s\S]*?gainResourceTalentScaling\.resolveGainResourceAmount[\s\S]*?offlineCappedResourceGain[\s\S]*?setAbilityCooldownExpiration/,
  'Berserker Rage must use normal off-GCD admission and exact capped resource gain',
);
requireText(
  world,
  /offlineCappedResourceGain[\s\S]*?resourcePrecisionState[\s\S]*?precision\.cappedAdd/,
  'Berserker Rage resource bridge must remain exact and capped',
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?berserkerRageAbilityCode\(\)[\s\S]*?startOfflineBerserkerRageCast[\s\S]*?applySupportedCastCommand[\s\S]*?berserkerRagePayloadAbilityIsExact[\s\S]*?startOfflineBerserkerRageCast/,
  'Berserker Rage slot and typed routes are missing',
);
requireText(
  world,
  /pub berserkerRageRuntimeStateTest\([\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastCommandForTest[\s\S]*?entityLevels\[0\] = 9[\s\S]*?offlinePlayerClass = <uint>rogue[\s\S]*?rngDraws == <uint>0/,
  'Berserker Rage runtime coverage is missing',
);

const fixture = read('scripts', 'woc_game', 'woc_m4_berserker_rage_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/berserker_rage_runtime_test_main.zr"')) {
  throw new Error('Berserker Rage ZrVM project fixture is missing');
}

process.stdout.write(`WOS225 Berserker Rage static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
