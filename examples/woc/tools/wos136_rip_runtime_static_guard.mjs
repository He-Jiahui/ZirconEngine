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
const start = classes.indexOf('  rip: {');
const end = classes.indexOf('\n\n  // ============== TALENT-GRANTED', start);
if (start < 0 || end < start) throw new Error('source Rip block is missing');
const rip = classes.slice(start, end);
for (const needle of [
  "name: 'Rip'", "class: 'druid'", 'learnLevel: 14', 'cost: 30',
  'castTime: 0', 'cooldown: 0', 'range: 0', "school: 'physical'",
  'requiresTarget: true', 'spendsCombo: true', "requiresForm: 'cat'",
  "type: 'dot', total: 60, duration: 12, interval: 2",
]) {
  if (!rip.includes(needle)) throw new Error(`source Rip drifted: ${needle}`);
}
requireText(lifecycle, /ability\.spendsCombo && p\.comboPoints <= 0/,
  'source Rip combo admission drifted');
requireText(dispatch, /case 'dot':[\s\S]*?Physical bleeds[\s\S]*?ctx\.applyAura[\s\S]*?if \(ability\.spendsCombo && spentCombo > 0\)[\s\S]*?p\.comboPoints = 0/,
  'source Rip physical-DoT/combo ordering drifted');

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/tigers_fury',[\s\S]*?'rip'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Rip projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'rip',
);
if (!entry || entry.index !== 76 || entry.definition.cost !== 30 ||
    entry.definition.spendsCombo !== true || entry.definition.requiresForm !== 'cat' ||
    entry.definition.effects?.[0]?.type !== 'dot' || entry.definition.effects[0].total !== 60 ||
    entry.definition.effects[0].duration !== 12 || entry.definition.effects[0].interval !== 2) {
  throw new Error('M4 Rip projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /ripAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("rip"\)/,
  'Rip catalog identity is missing');
requireText(world, /ripProfileIsValid[\s\S]*?spendsCombo[\s\S]*?physical[\s\S]*?60\.0[\s\S]*?12\.0[\s\S]*?2\.0/,
  'Rip source profile is missing');
requireText(world, /ripTargetIndex[\s\S]*?range <= 0\.0[\s\S]*?5\.0/,
  'Rip zero-range melee target admission is missing');
requireText(world, /startOfflineRipCast[\s\S]*?entityComboPoints[\s\S]*?forms\.formKindForAbilityCode[\s\S]*?ripTargetIndex[\s\S]*?resolvePureDotProfile[\s\S]*?effectiveOfflineAttackPower[\s\S]*?appendOfflinePureDot[\s\S]*?entityComboPoints\[casterIndex\] = 0/,
  'Rip immediate Cat finisher/physical-DoT reducer is missing');
requireText(world, /pureDotAbilityIndex[\s\S]*?ripAbilityCode\(\)[\s\S]*?m4AbilityCatalog\.indexOf\("rip"\)/,
  'Rip durable DoT identity mapping is missing');
requireText(world, /pureDotRankLevel[\s\S]*?ripAbilityCode\(\)[\s\S]*?rank == <uint>1[\s\S]*?14/,
  'Rip rank/learn-level mapping is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?ripAbilityCode\(\)[\s\S]*?startOfflineRipCast/,
  'Rip action-slot routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?ripPayloadAbilityIsExact[\s\S]*?startOfflineRipCast/,
  'Rip typed routing is missing');
requireText(world, /pub ripCommandStateTest\(\): int[\s\S]*?form_cat[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?offlineDotSnapshotPowers[\s\S]*?appendCastSlotCommand/,
  'Rip state regression coverage is missing');
requireText(world, /if \(ripCommandStateTest\(\) != 1\) \{[\s\S]*?return -130;/,
  'world selfTest must execute Rip');

process.stdout.write(`WOS136 Rip static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
