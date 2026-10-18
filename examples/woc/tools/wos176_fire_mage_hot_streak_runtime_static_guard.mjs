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
assert.match(
  classes,
  /pyroblast:[\s\S]*?cost: 125,[\s\S]*?castTime: 6\.0,[\s\S]*?min: 170, max: 225[\s\S]*?total: 48, duration: 12, interval: 2/,
);
assert.match(
  classes,
  /combustion:[\s\S]*?cost: 100,[\s\S]*?cooldown: 120,[\s\S]*?usableWhileCasting: true,[\s\S]*?offGcd: true,[\s\S]*?kind: 'combustion', value: 0, duration: 10/,
);
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
assert.match(lifecycle, /hasScopedNextCastInstant[\s\S]*?consumeNextCastInstant/);
assert.match(lifecycle, /canCastFree[\s\S]*?consumeFreeCostFor/);

const fireContract = read('scripts', 'woc_game', 'src', 'generated', 'fire_mage_contract.zr');
assert.match(fireContract, /hotStreakFreeModifierIdentityCode\(\): uint \{ return <uint>1; \}/);
assert.match(fireContract, /hotStreakInstantModifierIdentityCode\(\): uint \{ return <uint>2; \}/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityCombustionRemaining/);
assert.match(world, /schemaVersion != <uint>85 &&[\s\S]*?schemaVersion != <uint>86/);
assert.match(world, /if \(schemaVersion >= <uint>86\)[\s\S]*?entityCombustionRemaining/);
assert.match(world, /historicalCombustionIndex[\s\S]*?entityCombustionRemaining\.add\(0\.0\)/);
assert.match(
  world,
  /historicalCombustionIndex[\s\S]*?entityCombustionRemaining\.add\(0\.0\)[\s\S]*?if \(!fireMageProcStateIsValid\(state\)\)/,
);
assert.match(world, /pyroblastAbilityCode[\s\S]*?Pyroblast is absent/);
assert.match(world, /pyroblastProjectileProfileIsValid[\s\S]*?directDamage[\s\S]*?dot/);
assert.match(world, /startOfflinePyroblastCast[\s\S]*?instantAtStart[\s\S]*?consumeNextCastModifierForAbility[\s\S]*?launchOfflinePyroblastProjectile/);
assert.match(world, /completeOfflinePyroblastCast[\s\S]*?freeAtCompletion[\s\S]*?consumeNextCastModifierForAbility/);
assert.match(world, /startOfflineCombustionCast[\s\S]*?entityCombustionRemaining[\s\S]*?120000000/);
assert.match(world, /applyOfflineFireMageSpellHit[\s\S]*?hotStreakOnSpellHit[\s\S]*?setNextCastModifier/);
assert.match(world, /landOfflineFireballProjectile[\s\S]*?fireGuaranteedCrit[\s\S]*?applyOfflineFireMageSpellHit/);
assert.match(world, /landOfflinePyroblastProjectile[\s\S]*?fireGuaranteedCrit[\s\S]*?applyOfflineFireMageSpellHit/);
assert.match(world, /landOfflinePyroblastProjectile[\s\S]*?spellResist\.resolve[\s\S]*?TimedSpellResult[\s\S]*?randomUnits\.add[\s\S]*?randomUnits\.add/);
assert.match(world, /pyroblastPayloadAbilityIsExact[\s\S]*?startOfflinePyroblastCast/);
assert.match(world, /combustionPayloadAbilityIsExact[\s\S]*?startOfflineCombustionCast/);
assert.match(world, /pub fireMageHotStreakRuntimeStateTest\(/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'fire_mage_hot_streak_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_fire_mage_hot_streak_runtime_tests.zrp');
assert.match(testMain, /state\.fireMageHotStreakRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/fire_mage_hot_streak_runtime_test_main\.zr"/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '176-fire-mage-hot-streak-runtime.md');
assert.match(plan, /Schema 86/);
assert.match(plan, /resist draw[\s\S]*?damage\/crit draw/);

process.stdout.write('WOS176 Fire Mage Hot Streak runtime static guards passed\n');
