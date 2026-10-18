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

const sim = source('src/sim/sim.ts');
assert.match(sim, /cancelAura\(auraId: string[\s\S]*?removeCancelableAura\(e\.auras, auraId\)/);
const dispatcher = source('src/sim/combat/effect_dispatch.ts');
assert.match(dispatcher, /isPrimarySelfBuff[\s\S]*?\$\{ability\.id\}_\$\{eff\.kind\}/);
assert.match(dispatcher, /case 'dot'[\s\S]*?value: dotBase \+ dotSp/);
const auras = source('src/sim/combat/auras.ts');
assert.match(auras, /a\.kind === 'dot'[\s\S]*?let tickDamage = a\.value/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityArcanePowerSpellHasteRemaining/);
assert.match(world, /arcanePowerSpellHasteAuraPayloadIsExact[\s\S]*?abilityLength == <uint>28/);
assert.match(world, /applyOfflineCancelAuraCommand[\s\S]*?arcanePowerPayloadAbilityIsExact[\s\S]*?entitySpellDamageMultipliers\[actorIndex\] = 1\.0[\s\S]*?arcanePowerSpellHasteAuraPayloadIsExact[\s\S]*?entityArcanePowerSpellHasteRemaining\[actorIndex\] = 0\.0/);
assert.match(world, /schemaVersion != <uint>89[\s\S]*?schemaVersion >= <uint>89[\s\S]*?entityArcanePowerSpellHasteRemaining/);
assert.match(world, /pub arcanePowerAuraCancellationStateTest\([\s\S]*?appendCancelAuraCommandForTest[\s\S]*?entityArcanePowerSpellHasteRemaining[\s\S]*?90000000/);
assert.match(world, /completeOfflineGlacialFrontCast[\s\S]*?entitySpellDamageMultipliers\[casterIndex\][\s\S]*?entityCritDamageSpellBonus\[casterIndex\]/);
assert.match(world, /startOfflineArcaneExplosionCast[\s\S]*?projection\.addSource\([\s\S]*?entitySpellDamageMultipliers\[casterIndex\][\s\S]*?arcaneExplosionCommandStateTest[\s\S]*?entitySpellDamageMultipliers\[0\] = 1\.2[\s\S]*?entityHp\[1\] > 69/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'spell_aura_convergence_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_spell_aura_convergence_tests.zrp');
assert.match(testMain, /state\.arcanePowerAuraCancellationStateTest\(\)[\s\S]*?state\.arcaneExplosionCommandStateTest\(\)/);
assert.match(testPackage, /"entry": "world\/spell_aura_convergence_test_main"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '180-spell-aura-convergence.md');
assert.match(plan, /later[\s\S]*?periodic ticks do not reapply `buff_spelldmg`/);
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS180 spell aura convergence static guards passed\n');
