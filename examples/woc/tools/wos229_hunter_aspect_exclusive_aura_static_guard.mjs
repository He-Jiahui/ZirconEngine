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
for (const [id, kind, value, level, cost] of [
  ['aspect_of_the_hawk', 'buff_ap', 20, 4, 20],
  ['aspect_of_the_monkey', 'buff_dodge', 0.08, 5, 20],
  ['aspect_of_the_cheetah', 'buff_speed', 1.3, 14, 20],
]) {
  const start = classes.indexOf(`  ${id}: {`);
  const next = classes.slice(start + 1).search(/\n  [a-z_]+: \{/);
  const end = next < 0 ? classes.length : start + 1 + next;
  if (start < 0) throw new Error(`source ${id} definition is missing`);
  const block = classes.slice(start, end);
  requireText(block, new RegExp(`learnLevel: ${level}`), `${id} level drifted`);
  requireText(block, new RegExp(`cost: ${cost}`), `${id} cost drifted`);
  requireText(block, /castTime: 0,[\s\S]*?cooldown: 0,[\s\S]*?range: 0,[\s\S]*?school: 'nature',[\s\S]*?requiresTarget: false,[\s\S]*?exclusiveGroup: 'aspect'/,
    `${id} source profile drifted`);
  requireText(block, new RegExp(`kind: '${kind}', value: ${value}, duration: 1800`),
    `${id} effect drifted`);
}
const hawkSource = classes.slice(classes.indexOf('  aspect_of_the_hawk: {'), classes.indexOf('\n  serpent_sting:'));
requireText(hawkSource, /rank: 2,[\s\S]*?level: 12,[\s\S]*?cost: 30,[\s\S]*?value: 35/,
  'Hawk rank two drifted');
requireText(hawkSource, /rank: 3,[\s\S]*?level: 18,[\s\S]*?cost: 40,[\s\S]*?value: 50/,
  'Hawk rank three drifted');

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(generator, /'aspect_of_the_hawk'[\s\S]*?'aspect_of_the_monkey'[\s\S]*?'aspect_of_the_cheetah'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Hunter Aspects are absent from the M4 projection');
requireText(zrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Hunter Aspects');

const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const expected = new Map([
  ['aspect_of_the_hawk', [101, 'buff_ap', 20]],
  ['aspect_of_the_monkey', [102, 'buff_dodge', 0.08]],
  ['aspect_of_the_cheetah', [103, 'buff_speed', 1.3]],
]);
if (m4.entries.length !== 117) throw new Error('Hunter Aspect M4 cardinality drifted');
for (const [id, [index, kind, value]] of expected) {
  const entry = m4.entries.find((candidate) => candidate.id === id);
  if (!entry || entry.index !== index || entry.definition.class !== 'hunter' ||
      entry.definition.exclusiveGroup !== 'aspect' || entry.definition.effects?.[0]?.kind !== kind ||
      entry.definition.effects?.[0]?.value !== value || entry.definition.effects?.[0]?.duration !== 1800) {
    throw new Error(`Hunter Aspect projection drifted: ${id}`);
  }
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /aspectOfTheHawkAbilityCode[\s\S]*?aspectOfTheMonkeyAbilityCode[\s\S]*?aspectOfTheCheetahAbilityCode[\s\S]*?startOfflineHunterAspectCast/,
  'Hunter Aspect identities or reducer are missing');
requireText(world, /clearOfflineHunterAspectConflicts[\s\S]*?auraAbilityCode != newAbilityCode[\s\S]*?hunterAspectAbilityCodeIsKnown[\s\S]*?removeMotionAuraAt/,
  'Hunter Aspect exclusive-group removal is missing');
requireText(world, /retainedAttackPowerAuraProfileIsValid[\s\S]*?aspectOfTheHawkAbilityCode/,
  'Hawk attack-power aura boundary is missing');
requireText(world, /retainedDodgeAuraProfileIsValid[\s\S]*?aspectOfTheMonkeyAbilityCode/,
  'Monkey dodge aura boundary is missing');
requireText(world, /retainedSpeedAuraProfileIsValid[\s\S]*?aspectOfTheCheetahAbilityCode/,
  'Cheetah speed aura boundary is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?hunterAspectAbilityCodeIsKnown[\s\S]*?startOfflineHunterAspectCast[\s\S]*?applySupportedCastCommand[\s\S]*?hunterAspectAbilityCodeIsKnown[\s\S]*?startOfflineHunterAspectCast/,
  'Hunter Aspect slot or typed route is missing');
requireText(world, /pub hunterAspectRuntimeStateTest[\s\S]*?appendCastSlotCommand[\s\S]*?ageMotionAurasForEntity[\s\S]*?appendTypedCastCommandForTest[\s\S]*?appendCancelAuraCommandForTest/,
  'Hunter Aspect rank, exclusive, snapshot, expiry, typed, and cancel coverage is missing');

const fixture = read('scripts', 'woc_game', 'woc_m4_hunter_aspect_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/hunter_aspect_runtime_test_main.zr"')) {
  throw new Error('Hunter Aspect ZrVM project fixture is missing');
}

process.stdout.write(`WOS229 Hunter Aspect static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
