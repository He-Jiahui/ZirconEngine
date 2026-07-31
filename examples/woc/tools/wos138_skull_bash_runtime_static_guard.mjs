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
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
const start = classes.indexOf('  skull_bash: {');
const end = classes.indexOf('  spell_lock: {', start);
if (start < 0 || end < start) throw new Error('source Skull Bash block is missing');
const skullBash = classes.slice(start, end);
for (const needle of [
  "name: 'Headbutt'", "class: 'druid'", 'learnLevel: 10', 'cost: 10',
  'castTime: 0', 'cooldown: 15', 'range: 8', "school: 'physical'",
  'requiresTarget: true', "type: 'interrupt', lockout: 4",
]) {
  if (!skullBash.includes(needle)) throw new Error(`source Skull Bash drifted: ${needle}`);
}
requireText(dispatch, /case 'interrupt':[\s\S]*?target\.castingAbility === null[\s\S]*?interruptedDef\?\.school === 'physical'[\s\S]*?ctx\.cancelCast\(target\)[\s\S]*?kind: 'lockout'[\s\S]*?school,/,
  'source interrupt/lockout semantics drifted');
requireText(lifecycle, /isLockedOut\(p, cast\.def\.school\)/,
  'source in-progress school-lockout cancellation drifted');

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
const ccGenerator = read('tools', 'cc_contract_codegen.mjs');
if (!/hurricane',[\s\S]*?'skull_bash'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Skull Bash projection scope is missing');
}
if (!ccGenerator.includes('lockout: 13')) throw new Error('lockout motion-aura code is missing');
if (!zrGenerator.includes('pub idUtf8Length(index: int): int') ||
    !zrGenerator.includes('pub idUtf8Byte(index: int, byteIndex: int): uint')) {
  throw new Error('M4 ability UTF-8 identity projection is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'skull_bash',
);
if (!entry || entry.index !== 78 || entry.definition.cost !== 10 ||
    entry.definition.cooldown !== 15 || entry.definition.range !== 8 ||
    entry.definition.effects?.[0]?.type !== 'interrupt' ||
    entry.definition.effects[0].lockout !== 4) {
  throw new Error('M4 Skull Bash projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /entityMotionAuraSchoolCodes[\s\S]*?entityMotionAuraKindCodes[\s\S]*?entityMotionAuraRemaining/,
  'motion-aura school persistence is missing');
requireText(world, /skullBashAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("skull_bash"\)/,
  'Skull Bash catalog identity is missing');
requireText(world, /skullBashProfileIsValid[\s\S]*?school[\s\S]*?physical[\s\S]*?interrupt[\s\S]*?4\.0/,
  'Skull Bash source profile is missing');
requireText(world, /startOfflineSkullBashCast[\s\S]*?skullBashTargetIndex[\s\S]*?entityCastingAbility[\s\S]*?cancelOfflineTargetCast[\s\S]*?motionAuraKindCode\("lockout"\)/,
  'Skull Bash interrupt and school-lockout reducer is missing');
requireText(world, /offlineSchoolLockoutIsActive[\s\S]*?entityMotionAuraSchoolCodes/,
  'school-specific lockout predicate is missing');
requireText(world, /m4AbilityCodeFromPayload[\s\S]*?idUtf8Length[\s\S]*?idUtf8Byte[\s\S]*?offlineCastSchoolLockoutAllows/,
  'typed spell-school lockout admission is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?skullBashAbilityCode\(\)[\s\S]*?startOfflineSkullBashCast/,
  'Skull Bash action-slot routing is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?offlineCastSchoolLockoutAllows/,
  'Skull Bash slot admission must respect school lockout');
requireText(world, /applySupportedCastCommand[\s\S]*?skullBashPayloadAbilityIsExact[\s\S]*?startOfflineSkullBashCast/,
  'Skull Bash typed routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?m4AbilityCodeFromPayload[\s\S]*?offlineCastSchoolLockoutAllows/,
  'Skull Bash typed admission must respect school lockout');
requireText(world, /stepRetainedCasting[\s\S]*?offlineSchoolLockoutIsActive[\s\S]*?casting\.cancelCast/,
  'school lockout must cancel an already-active matching cast');
requireText(world, /pub skullBashCommandStateTest\(\): int[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?entityMotionAuraSchoolCodes[\s\S]*?appendCastSlotCommand/,
  'Skull Bash state regression coverage is missing');
requireText(world, /if \(skullBashCommandStateTest\(\) != 1\) \{[\s\S]*?return -132;/,
  'world selfTest must execute Skull Bash');

process.stdout.write(`WOS138 Skull Bash static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
