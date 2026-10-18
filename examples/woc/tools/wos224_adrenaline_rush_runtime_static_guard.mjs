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
  /adrenaline_rush:[\s\S]*?class: 'rogue',[\s\S]*?learnLevel: 20,[\s\S]*?cost: 0,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 180,[\s\S]*?range: 0,[\s\S]*?school: 'physical',[\s\S]*?requiresTarget: false,[\s\S]*?offGcd: true,[\s\S]*?type: 'gainResource', amount: 60/,
  'source Adrenaline Rush profile drifted',
);

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(generator, /'adrenaline_rush'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Adrenaline Rush is absent from the M4 projection');
requireText(zrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Adrenaline Rush');

const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const adrenalineRush = m4.entries.find((entry) => entry.id === 'adrenaline_rush');
if (m4.entries.length !== 117 || !adrenalineRush || adrenalineRush.index !== 96 ||
    adrenalineRush.definition.class !== 'rogue' || adrenalineRush.definition.learnLevel !== 20 ||
    adrenalineRush.definition.cost !== 0 || adrenalineRush.definition.castTime !== 0 ||
    adrenalineRush.definition.cooldown !== 180 || adrenalineRush.definition.range !== 0 ||
    adrenalineRush.definition.school !== 'physical' || adrenalineRush.definition.requiresTarget ||
    !adrenalineRush.definition.offGcd ||
    adrenalineRush.definition.effects?.[0]?.type !== 'gainResource' ||
    adrenalineRush.definition.effects?.[0]?.amount !== 60) {
  throw new Error('Adrenaline Rush source projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(
  world,
  /adrenalineRushAbilityCode[\s\S]*?adrenalineRushPayloadAbilityIsExact[\s\S]*?adrenalineRushProfileIsValid[\s\S]*?startOfflineAdrenalineRushCast/,
  'Adrenaline Rush identity or reducer is missing',
);
requireText(
  world,
  /startOfflineAdrenalineRushCast[\s\S]*?catalogAdmission[\s\S]*?adrenalineRushProfileIsValid[\s\S]*?gainResourceTalentScaling\.resolveGainResourceAmount[\s\S]*?offlineCappedResourceGain[\s\S]*?setAbilityCooldownExpiration/,
  'Adrenaline Rush must use normal off-GCD admission and exact capped resource gain',
);
requireText(
  world,
  /offlineCappedResourceGain[\s\S]*?resourcePrecisionState[\s\S]*?precision\.cappedAdd/,
  'Adrenaline Rush resource bridge must remain exact and capped',
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?adrenalineRushAbilityCode\(\)[\s\S]*?startOfflineAdrenalineRushCast[\s\S]*?applySupportedCastCommand[\s\S]*?adrenalineRushPayloadAbilityIsExact[\s\S]*?startOfflineAdrenalineRushCast/,
  'Adrenaline Rush slot and typed routes are missing',
);
requireText(
  world,
  /pub adrenalineRushRuntimeStateTest\([\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastCommandForTest[\s\S]*?rngDraws == <uint>0/,
  'Adrenaline Rush runtime coverage is missing',
);
requireText(
  world,
  /pub adrenalineRushRuntimeStateTest\([\s\S]*?entityLevels\[0\] = 19[\s\S]*?offlinePlayerClass = <uint>warrior[\s\S]*?energyResourceKind\(\)[\s\S]*?rageResourceKind\(\)/,
  'Adrenaline Rush rejection coverage is missing',
);

const fixture = read('scripts', 'woc_game', 'woc_m4_adrenaline_rush_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/adrenaline_rush_runtime_test_main.zr"')) {
  throw new Error('Adrenaline Rush ZrVM project fixture is missing');
}

process.stdout.write(`WOS224 Adrenaline Rush static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
