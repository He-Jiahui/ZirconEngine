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
  /exorcism:[\s\S]*?class: 'paladin',[\s\S]*?learnLevel: 5,[\s\S]*?cost: 55,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 15,[\s\S]*?range: 30,[\s\S]*?school: 'holy',[\s\S]*?requiresTarget: true,[\s\S]*?type: 'directDamage', min: 46, max: 56/,
  'source Exorcism profile drifted',
);

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(generator, /'exorcism'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Exorcism is absent from the M4 projection');
requireText(zrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Exorcism');

const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const exorcism = m4.entries.find((entry) => entry.id === 'exorcism');
if (m4.entries.length !== 117 || !exorcism || exorcism.index !== 98 ||
    exorcism.definition.class !== 'paladin' || exorcism.definition.learnLevel !== 5 ||
    exorcism.definition.cost !== 55 || exorcism.definition.castTime !== 0 ||
    exorcism.definition.cooldown !== 15 || exorcism.definition.range !== 30 ||
    exorcism.definition.school !== 'holy' || !exorcism.definition.requiresTarget ||
    exorcism.definition.effects?.[0]?.type !== 'directDamage' ||
    exorcism.definition.effects?.[0]?.min !== 46 ||
    exorcism.definition.effects?.[0]?.max !== 56) {
  throw new Error('Exorcism source projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(
  world,
  /exorcismAbilityCode[\s\S]*?exorcismPayloadAbilityIsExact[\s\S]*?exorcismTargetIndex[\s\S]*?exorcismProjectileProfileIsValid[\s\S]*?startOfflineExorcismCast[\s\S]*?landOfflineExorcismProjectile/,
  'Exorcism identity, target reducer, or impact is missing',
);
requireText(
  world,
  /startOfflineExorcismCast[\s\S]*?catalogAdmission[\s\S]*?spendOfflineAbilityResource[\s\S]*?setAbilityCooldownExpiration[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_HOLY/,
  'Exorcism must use normal admission, cost, independent cooldown, and Holy projectile',
);
requireText(
  world,
  /landOfflineExorcismProjectile[\s\S]*?spell\.school = "holy"[\s\S]*?resolveOfflineFixedSpellImpactDamage/,
  'Exorcism must reuse the shared Holy spell-impact resolver',
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?exorcismAbilityCode\(\)[\s\S]*?startOfflineExorcismCast[\s\S]*?applySupportedCastCommand[\s\S]*?exorcismPayloadAbilityIsExact[\s\S]*?startOfflineExorcismCast/,
  'Exorcism slot and typed routes are missing',
);
requireText(
  world,
  /offlineProjectileStateIsValid[\s\S]*?exorcismAbilityCode\(\)[\s\S]*?landOfflineExorcismProjectile[\s\S]*?pub exorcismRuntimeStateTest[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?entityLevels\[0\] = 4/,
  'Exorcism projection, impact dispatch, or focused state coverage is missing',
);

const fixture = read('scripts', 'woc_game', 'woc_m4_exorcism_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/exorcism_runtime_test_main.zr"')) {
  throw new Error('Exorcism ZrVM project fixture is missing');
}

process.stdout.write(`WOS226 Exorcism static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
