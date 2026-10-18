import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = resolve(projectRoot, '..', '..');
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const source = (path) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], { encoding: 'utf8' },
);

const classes = source('src/sim/content/classes.ts');
const arcanePowerStart = classes.indexOf('arcane_power:');
const arcanePowerEnd = classes.indexOf('\n  combustion:', arcanePowerStart);
assert.ok(arcanePowerStart >= 0 && arcanePowerEnd > arcanePowerStart);
const arcanePower = classes.slice(arcanePowerStart, arcanePowerEnd);
for (const field of [
  'learnLevel: 5',
  'cost: 0',
  'castTime: 0',
  'cooldown: 90',
  "school: 'arcane'",
  "kind: 'buff_spelldmg', value: 0.2, duration: 10",
  "kind: 'buff_spellhaste', value: 0.1, duration: 10",
]) assert.ok(arcanePower.includes(field), `Arcane Power source field missing: ${field}`);
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
assert.match(lifecycle, /const gcd = Math\.max[\s\S]*?if \(!ability\.offGcd\) p\.gcdRemaining[\s\S]*?applyAbility/);
const spellCombat = source('src/sim/combat/spell_combat.ts');
assert.match(spellCombat, /buff_spelldmg[\s\S]*?return 1 \+ bonus/);
assert.match(spellCombat, /buff_spellhaste[\s\S]*?return 1 \+ Math\.max\(0, bonus\)/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /arcanePowerAbilityCode[\s\S]*?abilityClassId[\s\S]*?"mage"/);
assert.match(world, /startOfflineArcanePowerCast[\s\S]*?preBuffGcd[\s\S]*?entityArcanePowerRemaining[\s\S]*?entitySpellDamageMultipliers[\s\S]*?90000000/);
assert.match(world, /mageSpellHasteMultiplier[\s\S]*?arcanePowerSpellHasteBonus/);
assert.match(world, /configureOfflineSpellDamageModifiers[\s\S]*?critDamageBonus[\s\S]*?damageMultiplier/);
assert.equal((world.match(/configureOfflineSpellDamageModifiers\(state, casterIndex, spell\)/g) ?? []).length, 16);
assert.match(world, /ageOfflineArcanePower[\s\S]*?entityArcanePowerRemaining[\s\S]*?entitySpellDamageMultipliers\[index\] = 1\.0/);
assert.match(world, /offlineAbilitySchoolCode[\s\S]*?arcanePowerAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_ARCANE/);
assert.match(world, /schemaVersion != <uint>113[\s\S]*?encodeState\(state[\s\S]*?<uint>113/);
assert.match(world, /pub arcanePowerRuntimeStateTest\([\s\S]*?appendTypedCastCommandForTest[\s\S]*?entityArcanePowerRemaining[\s\S]*?encodeStateVersion\(state, <uint>87\)/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'arcane_power_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_arcane_power_runtime_tests.zrp');
assert.match(testMain, /state\.arcanePowerRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/arcane_power_runtime_test_main\.zr"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '179-arcane-power-runtime.md');
assert.match(plan, /pre-cast `spellHasteMult`/);
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS179 Arcane Power runtime static guards passed\n');
