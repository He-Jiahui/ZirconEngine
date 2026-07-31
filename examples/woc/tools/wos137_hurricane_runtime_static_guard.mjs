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
const casting = source('src/sim/combat/casting_lifecycle.ts');
const start = classes.indexOf('  hurricane: {');
const end = classes.indexOf('  earthquake: {', start);
if (start < 0 || end < start) throw new Error('source Hurricane block is missing');
const hurricane = classes.slice(start, end);
for (const needle of [
  "name: 'Galeheart'", "class: 'druid'", 'learnLevel: 18', 'cost: 90',
  'castTime: 0', 'cooldown: 12', 'range: 30', "school: 'nature'",
  'requiresTarget: false', "targetMode: 'position'", 'channel: { duration: 6, ticks: 6 }',
  "type: 'aoeDamage', min: 12, max: 16, radius: 8",
]) {
  if (!hurricane.includes(needle)) throw new Error(`source Hurricane drifted: ${needle}`);
}
requireText(casting, /Ground-targeted channels[\s\S]*?targetMode === 'position'[\s\S]*?channelTickBonus[\s\S]*?Math\.round\(dmg\)/,
  'source positioned-channel pulse ordering drifted');

const generator = read('tools', 'm4_ability_codegen.mjs');
const zrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
if (!/rip',[\s\S]*?'hurricane'/.test(generator) ||
    !generator.includes('EXPECTED_ABILITY_COUNT = 79') ||
    !zrGenerator.includes('document.entries.length === 79')) {
  throw new Error('M4 Hurricane projection scope is missing');
}
const entry = JSON.parse(read('contracts', 'm4_abilities.json')).entries.find(
  (value) => value.id === 'hurricane',
);
if (!entry || entry.index !== 77 || entry.definition.cost !== 90 ||
    entry.definition.cooldown !== 12 || entry.definition.targetMode !== 'position' ||
    entry.definition.channel?.duration !== 6 || entry.definition.channel?.ticks !== 6 ||
    entry.definition.effects?.[0]?.type !== 'aoeDamage' ||
    entry.definition.effects[0].min !== 12 || entry.definition.effects[0].max !== 16 ||
    entry.definition.effects[0].radius !== 8) {
  throw new Error('M4 Hurricane projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /hurricaneAbilityCode\([\s\S]*?knownAbilityCatalog\.abilityCode\("hurricane"\)/,
  'Hurricane catalog identity is missing');
requireText(world, /hurricaneProfileIsValid[\s\S]*?nature[\s\S]*?position[\s\S]*?90\.0[\s\S]*?12\.0[\s\S]*?6\.0[\s\S]*?8\.0/,
  'Hurricane source profile is missing');
requireText(world, /startOfflineHurricaneCast[\s\S]*?math\.sqrt[\s\S]*?range \/ distance[\s\S]*?armChannel[\s\S]*?setAbilityCooldownExpiration[\s\S]*?entityCastAimPresent/,
  'Hurricane must clamp and snapshot its source aim');
requireText(world, /launchOfflineHurricaneChannelTick[\s\S]*?channelTickBonus[\s\S]*?resolveOfflineGroundAoEPulse/,
  'Hurricane channel tick reducer is missing');
requireText(world, /applySupportedCastAtCommand[\s\S]*?hurricanePayloadAbilityIsExact[\s\S]*?startOfflineHurricaneCast/,
  'Hurricane castAt routing is missing');
requireText(world, /stepRetainedCasting[\s\S]*?hurricaneAbilityCode\(\)[\s\S]*?launchOfflineHurricaneChannelTick[\s\S]*?clearHurricaneCastAim/,
  'Hurricane channel lifecycle cleanup is missing');
requireText(world, /pub hurricaneCommandStateTest\(\): int[\s\S]*?appendTypedCastAtCommandForTest[\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?rngDraws == <uint>6/,
  'Hurricane state regression coverage is missing');
requireText(world, /if \(hurricaneCommandStateTest\(\) != 1\) \{[\s\S]*?return -131;/,
  'world selfTest must execute Hurricane');

process.stdout.write(`WOS137 Hurricane static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
