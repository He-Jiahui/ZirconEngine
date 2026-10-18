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
assert.match(classes, /icy_veins:[\s\S]*?specs: \['frost'\][\s\S]*?cooldown: 180[\s\S]*?buff_spellhaste', value: 0\.3, duration: 10[\s\S]*?cast_shield', value: 1, duration: 10/);
const spellCombat = source('src/sim/combat/spell_combat.ts');
assert.match(spellCombat, /aura\.kind === 'buff_spellhaste'[\s\S]*?bonus \+= aura\.value/);
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
assert.match(lifecycle, /pushbackCast\(p: Entity\): void \{[\s\S]*?if \(hasCastShield\(p\)\) return/);
const dispatcher = source('src/sim/combat/effect_dispatch.ts');
assert.match(dispatcher, /isPrimarySelfBuff[\s\S]*?isPrimarySelfBuff \? ability\.id : `\$\{ability\.id\}_\$\{eff\.kind\}`/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityIcyVeinsSpellHasteRemaining[\s\S]*?entityIcyVeinsCastShieldRemaining/);
assert.match(world, /schemaVersion != <uint>90[\s\S]*?schemaVersion < <uint>90 && !icyVeinsStateIsLegacyCompatible[\s\S]*?schemaVersion >= <uint>90[\s\S]*?entityIcyVeinsCastShieldRemaining/);
assert.match(world, /icyVeinsPayloadAbilityIsExact[\s\S]*?abilityLength == <uint>9/);
assert.match(world, /icyVeinsCastShieldAuraPayloadIsExact[\s\S]*?abilityLength == <uint>21/);
assert.match(world, /startOfflineIcyVeinsCast[\s\S]*?catalogAdmission[\s\S]*?preBuffGcd[\s\S]*?entityIcyVeinsSpellHasteRemaining\[casterIndex\] = 10\.0[\s\S]*?180000000/);
assert.match(world, /mageSpellHasteMultiplier[\s\S]*?icyVeinsSpellHasteBonus/);
assert.match(world, /loadCastingState[\s\S]*?icyVeinsCastShieldActive[\s\S]*?storeCastingState[\s\S]*?!icyVeinsCastShieldActive/);
assert.match(world, /applyOfflineCancelAuraCommand[\s\S]*?icyVeinsPayloadAbilityIsExact[\s\S]*?entityIcyVeinsSpellHasteRemaining\[actorIndex\] = 0\.0[\s\S]*?icyVeinsCastShieldAuraPayloadIsExact[\s\S]*?entityIcyVeinsCastShieldRemaining\[actorIndex\] = 0\.0/);
assert.match(world, /ageOfflineIcyVeins[\s\S]*?entityIcyVeinsSpellHasteRemaining[\s\S]*?entityIcyVeinsCastShieldRemaining/);
assert.match(world, /pub icyVeinsRuntimeStateTest\([\s\S]*?frostSpec[\s\S]*?appendCancelAuraCommandForTest[\s\S]*?castBefore \+ 0\.5/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'icy_veins_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_icy_veins_runtime_tests.zrp');
assert.match(testMain, /state\.icyVeinsRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/icy_veins_runtime_test_main\.zr"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '181-icy-veins-runtime.md');
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS181 Icy Veins runtime static guards passed\n');
