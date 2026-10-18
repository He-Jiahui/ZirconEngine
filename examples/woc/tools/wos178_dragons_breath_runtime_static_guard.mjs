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
  /dragons_breath:[\s\S]*?learnLevel: 14,[\s\S]*?cost: 90,[\s\S]*?castTime: 2\.4,[\s\S]*?cooldown: 20,[\s\S]*?guaranteedCritLevel: 4,[\s\S]*?hotStreakOnce: true/,
);
const effects = source('src/sim/combat/effect_dispatch.ts');
assert.match(effects, /case 'empoweredCone'[\s\S]*?ctx\.rng\.chance[\s\S]*?ctx\.rng\.range[\s\S]*?if \(m\.dead\) continue;[\s\S]*?hotStreakOnce/);

const contract = read('scripts', 'woc_game', 'src', 'generated', 'empowered_cone_contract.zr');
assert.match(contract, /pub resourceCost[\s\S]*?<uint>86.*90\.0/);
assert.match(contract, /pub cooldown[\s\S]*?<uint>86.*20\.0/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /dragonsBreathProfileIsValid[\s\S]*?resourceCost[\s\S]*?guaranteedCritLevel/);
assert.match(world, /startOfflineDragonsBreathCast[\s\S]*?armEmpowered[\s\S]*?stageCount/);
assert.match(world, /completeOfflineDragonsBreathCast[\s\S]*?EmpoweredConeBatch[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?fireGuaranteedCrit[\s\S]*?applyOfflineFireMageSpellHit/);
assert.match(world, /completeOfflineDragonsBreathCast[\s\S]*?offlineFearDuration[\s\S]*?entityMotionAuraSchoolCodes/);
assert.match(world, /applyOfflineReleaseEmpoweredCommand[\s\S]*?dragonsBreathPayloadAbilityIsExact[\s\S]*?completeOfflineDragonsBreathCast/);
assert.match(world, /applySupportedCastSlotCommand[\s\S]*?dragonsBreathAbilityCode[\s\S]*?startOfflineDragonsBreathCast/);
assert.match(world, /stepRetainedCasting[\s\S]*?completedAbility == dragonsBreathAbilityCode[\s\S]*?completeOfflineDragonsBreathCast/);
assert.match(world, /clearOfflineBreakableIncapacitateOnDamage[\s\S]*?dragonsBreathCode/);
assert.match(world, /offlineAbilitySchoolCode[\s\S]*?abilityCode == dragonsBreathAbilityCode\(\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FIRE/);
assert.match(world, /pub dragonsBreathRuntimeStateTest\([\s\S]*?appendTypedCastCommandForTest[\s\S]*?offlineCastSchoolLockoutAllows\(state, 0, abilityCode\)[\s\S]*?appendReleaseEmpoweredCommandForTest[\s\S]*?entityLastResolvedEmpowerLevel[\s\S]*?entityHeatingUpRemaining/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'dragons_breath_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_dragons_breath_runtime_tests.zrp');
assert.match(testMain, /state\.dragonsBreathRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/dragons_breath_runtime_test_main\.zr"/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '178-dragons-breath-runtime.md');
assert.match(plan, /does not advance it or consume a crit\/damage draw/);
assert.match(plan, /damage happens before damage-breakable incapacitation/i);

process.stdout.write('WOS178 Dragon\'s Breath runtime static guards passed\n');
