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
const start = classes.indexOf('  insect_swarm: {');
const end = classes.indexOf('  tigers_fury: {', start);
if (start < 0 || end < start) throw new Error('source Insect Swarm block is missing');
const insectSwarm = classes.slice(start, end);
for (const needle of [
  "name: 'Stinging Swarm'", "class: 'druid'", 'learnLevel: 20', 'cost: 45',
  'castTime: 0', 'cooldown: 0', 'range: 30', "school: 'nature'",
  'requiresTarget: true', "type: 'dot', total: 48, duration: 12, interval: 3",
]) {
  if (!insectSwarm.includes(needle)) throw new Error(`source Insect Swarm drifted: ${needle}`);
}
requireText(dispatch, /case 'dot':[\s\S]*?const hybrid[\s\S]*?dotTickBonus[\s\S]*?ctx\.applyAura/,
  'source pure-DoT snapshot dispatch drifted');

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/pounce',[\s\S]*?'insect_swarm'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Insect Swarm projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'insect_swarm',
);
if (!entry || entry.index !== 74 || entry.definition.cost !== 45 ||
    entry.definition.castTime !== 0 || entry.definition.range !== 30 ||
    entry.definition.effects?.[0]?.type !== 'dot' || entry.definition.effects[0].total !== 48 ||
    entry.definition.effects[0].duration !== 12 || entry.definition.effects[0].interval !== 3) {
  throw new Error('M4 Insect Swarm projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /insectSwarmAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("insect_swarm"\)/,
  'Insect Swarm catalog identity is missing');
requireText(world, /pureDotAbilityIndex[\s\S]*?insectSwarmAbilityCode\(\)[\s\S]*?m4AbilityCatalog\.indexOf\("insect_swarm"\)/,
  'Insect Swarm pure-DoT identity mapping is missing');
requireText(world, /pureDotRankLevel[\s\S]*?insectSwarmAbilityCode\(\)[\s\S]*?rank == <uint>1[\s\S]*?20/,
  'Insect Swarm rank/learn-level mapping is missing');
requireText(world, /startOfflineInsectSwarmCast[\s\S]*?startOfflinePureDotCast[\s\S]*?insectSwarmAbilityCode/,
  'Insect Swarm instant pure-DoT reducer is missing');
requireText(world, /offlineProjectileStateIsValid[\s\S]*?insectSwarmAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_NATURE[\s\S]*?pureDotProjectileProfileIsValid/,
  'Insect Swarm nature projectile validation is missing');
requireText(world, /landOfflineInsectSwarmProjectile[\s\S]*?landOfflinePureDotProjectile[\s\S]*?insectSwarmAbilityCode/,
  'Insect Swarm landing projection is missing');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?insectSwarmAbilityCode\(\)[\s\S]*?startOfflineInsectSwarmCast/,
  'Insect Swarm action-slot routing is missing');
requireText(world, /applySupportedCastCommand[\s\S]*?insectSwarmPayloadAbilityIsExact[\s\S]*?startOfflineInsectSwarmCast/,
  'Insect Swarm typed routing is missing');
requireText(world, /pub insectSwarmCommandStateTest\(\): int[\s\S]*?appendTypedCastTargetCommandForTest[\s\S]*?offlineDotDurations[\s\S]*?appendCastSlotCommand/,
  'Insect Swarm state regression coverage is missing');
requireText(world, /if \(insectSwarmCommandStateTest\(\) != 1\) \{[\s\S]*?return -128;/,
  'world selfTest must execute Insect Swarm');

process.stdout.write(`WOS134 Insect Swarm static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
