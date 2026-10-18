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

const classesSource = source('src/sim/content/classes.ts');
const frostArmorStart = classesSource.indexOf('  frost_armor: {');
const frostArmorEnd = classesSource.indexOf('\n  arcane_intellect:', frostArmorStart);
if (frostArmorStart < 0 || frostArmorEnd < 0) {
  throw new Error('source Frost Armor definition is missing');
}
const frostArmorSource = classesSource.slice(frostArmorStart, frostArmorEnd);
for (const [pattern, message] of [
  [/class: 'mage'/, 'class'],
  [/learnLevel: 1/, 'learn level'],
  [/excludeSpecs: \['fire', 'arcane'\]/, 'excluded specs'],
  [/cost: 20/, 'rank one cost'],
  [/castTime: 0/, 'cast time'],
  [/cooldown: 0/, 'cooldown'],
  [/range: 0/, 'range'],
  [/school: 'frost'/, 'school'],
  [/requiresTarget: false/, 'target mode'],
  [/type: 'selfBuff', kind: 'buff_armor', value: 30, duration: 1800/, 'rank one effect'],
  [/rank: 2,\s*level: 10,\s*cost: 30,\s*effects: \[\{ type: 'selfBuff', kind: 'buff_armor', value: 50, duration: 1800 \}\]/, 'rank two'],
  [/rank: 3,\s*level: 18,\s*cost: 45,\s*effects: \[\{ type: 'selfBuff', kind: 'buff_armor', value: 70, duration: 1800 \}\]/, 'rank three'],
]) {
  requireText(frostArmorSource, pattern, `source Frost Armor ${message} drifted`);
}

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(generator, /'frost_armor'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Frost Armor is absent from the M4 projection');
requireText(zrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Frost Armor');

const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const frostArmor = m4.entries.find((entry) => entry.id === 'frost_armor');
if (m4.entries.length !== 117 || !frostArmor || frostArmor.index !== 100 ||
    frostArmor.definition.class !== 'mage' || frostArmor.definition.learnLevel !== 1 ||
    frostArmor.definition.cost !== 20 || frostArmor.definition.castTime !== 0 ||
    frostArmor.definition.cooldown !== 0 || frostArmor.definition.range !== 0 ||
    frostArmor.definition.school !== 'frost' || frostArmor.definition.requiresTarget ||
    frostArmor.definition.effects?.[0]?.type !== 'selfBuff' ||
    frostArmor.definition.effects?.[0]?.kind !== 'buff_armor' ||
    frostArmor.definition.effects?.[0]?.value !== 30 ||
    frostArmor.definition.effects?.[0]?.duration !== 1800 ||
    frostArmor.definition.ranks?.[0]?.rank !== 2 ||
    frostArmor.definition.ranks?.[0]?.level !== 10 ||
    frostArmor.definition.ranks?.[0]?.cost !== 30 ||
    frostArmor.definition.ranks?.[0]?.effects?.[0]?.value !== 50 ||
    frostArmor.definition.ranks?.[1]?.rank !== 3 ||
    frostArmor.definition.ranks?.[1]?.level !== 18 ||
    frostArmor.definition.ranks?.[1]?.cost !== 45 ||
    frostArmor.definition.ranks?.[1]?.effects?.[0]?.value !== 70) {
  throw new Error('Frost Armor source projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(
  world,
  /frostArmorAbilityCode[\s\S]*?frostArmorPayloadAbilityIsExact[\s\S]*?frostArmorProfileIsValid[\s\S]*?retainedArmorAuraProfileIsValid[\s\S]*?retainedArmorAuraBonus[\s\S]*?startOfflineFrostArmorCast/,
  'Frost Armor identity, retained armor-aura boundary, or reducer is missing',
);
requireText(
  world,
  /retainedArmorAuraProfileIsValid[\s\S]*?demonSkinAbilityCode[\s\S]*?1800\.0[\s\S]*?80\.0[\s\S]*?barkskinAbilityCode[\s\S]*?15\.0[\s\S]*?150\.0[\s\S]*?frostArmorAbilityCode[\s\S]*?1800\.0[\s\S]*?70\.0/,
  'retained armor-aura invariant must retain Demon Skin, Barkskin, and Frost Armor only',
);
requireText(
  world,
  /startOfflineFrostArmorCast[\s\S]*?catalogAdmission[\s\S]*?frostArmorProfileIsValid[\s\S]*?ghostWolfGlobalCooldownSeconds[\s\S]*?spendOfflineAbilityResource[\s\S]*?applyOfflineMotionAuraWithDetails/,
  'Frost Armor must use normal-GCD admission, resource spending, and a motion aura',
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?frostArmorAbilityCode\(\)[\s\S]*?startOfflineFrostArmorCast[\s\S]*?applySupportedCastCommand[\s\S]*?frostArmorPayloadAbilityIsExact[\s\S]*?startOfflineFrostArmorCast/,
  'Frost Armor slot and typed routes are missing',
);
requireText(
  world,
  /pub frostArmorRuntimeStateTest[\s\S]*?talentSelectionCatalog\.specCode\("mage", "frost"\)[\s\S]*?"fire"[\s\S]*?"arcane"[\s\S]*?appendCastSlotCommand[\s\S]*?ageMotionAurasForEntity[\s\S]*?appendTypedCastCommandForTest[\s\S]*?appendCancelAuraCommandForTest/,
  'Frost Armor rank, spec, snapshot, expiry, and route coverage is missing',
);

const fixture = read('scripts', 'woc_game', 'woc_m4_frost_armor_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/frost_armor_runtime_test_main.zr"')) {
  throw new Error('Frost Armor ZrVM project fixture is missing');
}

process.stdout.write(`WOS228 Frost Armor static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
