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
const start = classes.indexOf('  hibernate: {');
const end = classes.indexOf('  dash: {', start);
if (start < 0 || end < start) throw new Error('source Hibernate block is missing');
const hibernate = classes.slice(start, end);
for (const needle of [
  "name: 'Slumber'", "class: 'druid'", 'learnLevel: 18', 'cost: 50',
  'castTime: 1.5', 'cooldown: 0', 'range: 30', "school: 'nature'",
  'requiresTarget: true', "type: 'incapacitate', duration: 8",
]) {
  if (!hibernate.includes(needle)) throw new Error(`source Hibernate drifted: ${needle}`);
}
requireText(dispatch, /case 'incapacitate':[\s\S]*?kind: 'incapacitate',[\s\S]*?value: ability\.fearDr \? ctx\.rng\.range[\s\S]*?breaksOnDamage: true,[\s\S]*?ctx\.enterCombat\(p, target\)/,
  'source Hibernate incapacitate application drifted');

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/faerie_fire',[\s\S]*?'hibernate'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Hibernate projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'hibernate',
);
if (!entry || entry.index !== 71 || entry.definition.cost !== 50 ||
    entry.definition.castTime !== 1.5 || entry.definition.range !== 30 ||
    entry.definition.effects?.[0]?.type !== 'incapacitate' ||
    entry.definition.effects[0].duration !== 8) {
  throw new Error('M4 Hibernate projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /hibernateAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("hibernate"\)/,
  'Hibernate catalog identity is missing');
requireText(world, /hibernateTargetIndex[\s\S]*?m4AbilityCatalog\.metric[\s\S]*?range[\s\S]*?targetIndex : -1/,
  'Hibernate source range gate is missing');
requireText(world, /startOfflineHibernateCast[\s\S]*?entityCastGcdRemaining[\s\S]*?cast\.armTimed[\s\S]*?cast\.lockTargets[\s\S]*?entityCastTargetIds/,
  'Hibernate hard-cast admission is missing');
requireText(world, /completeOfflineHibernateCast[\s\S]*?hibernateCompletionTargetIndex[\s\S]*?appendOfflineAbilityProjectile[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE/,
  'Hibernate cast completion is missing');
requireText(world, /hibernateProjectileProfileIsValid[\s\S]*?incapacitate[\s\S]*?duration"\) == 8\.0/,
  'Hibernate projectile profile is missing');
requireText(world, /landOfflineHibernateProjectile[\s\S]*?spellResist\.resolve[\s\S]*?applyOfflineMotionAuraWithDetails[\s\S]*?motionAuraKindCode\("incapacitate"\)[\s\S]*?8\.0/,
  'Hibernate landing aura is missing');
requireText(world, /clearOfflineBreakableIncapacitateOnDamage[\s\S]*?hibernateAbilityCode\(\)[\s\S]*?removeMotionAuraAt/,
  'Hibernate must wake from any positive damage');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?hibernateAbilityCode\(\)[\s\S]*?startOfflineHibernateCast/,
  'Hibernate action-slot routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?hibernatePayloadAbilityIsExact[\s\S]*?startOfflineHibernateCast/,
  'Hibernate typed routing is missing');
requireText(world, /stepRetainedCasting[\s\S]*?hibernateAbilityCode\(\)[\s\S]*?completeOfflineHibernateCast/,
  'Hibernate completion dispatch is missing');
requireText(world, /stepOfflineEastbrookProjectiles[\s\S]*?hibernateAbilityCode\(\)[\s\S]*?landOfflineHibernateProjectile/,
  'Hibernate landing dispatch is missing');
requireText(world, /pub hibernateCommandStateTest\(\): int[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?clearOfflineBreakableIncapacitateOnDamage/,
  'Hibernate state regression coverage is missing');
requireText(world, /if \(hibernateCommandStateTest\(\) != 1\) \{[\s\S]*?return -125;/,
  'world selfTest must execute Hibernate');

process.stdout.write(`WOS131 Hibernate static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
