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
  /flamestrike:[\s\S]*?cost: 80,[\s\S]*?castTime: 2,[\s\S]*?cooldown: 12,[\s\S]*?range: 30,[\s\S]*?targetMode: 'position',[\s\S]*?min: 34, max: 44, radius: 7, canCrit: true/,
);
const effects = source('src/sim/combat/effect_dispatch.ts');
assert.match(effects, /canCrit[\s\S]*?aoeTargets\.length > 0[\s\S]*?ctx\.rng\.chance[\s\S]*?fireGuaranteedCrit/);
assert.match(effects, /for \(const m of aoeTargets\)[\s\S]*?ctx\.rng\.range/);
assert.match(effects, /noteSpellHit\(ctx, p, aoeCrit, ability\.id\)/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /flamestrikeProfileIsValid[\s\S]*?targetMode"\) == "position"[\s\S]*?canCrit/);
assert.match(world, /flamestrikeAimX[\s\S]*?distance > range/);
assert.match(world, /startOfflineFlamestrikeCast[\s\S]*?instantAtStart[\s\S]*?consumeNextCastModifierForAbility/);
assert.match(world, /completeOfflineFlamestrikeCast[\s\S]*?freeAtCompletion[\s\S]*?setAbilityCooldownExpiration/);
assert.match(world, /resolveOfflineFlamestrikeBurst[\s\S]*?targetCount == 0\) \{ return; \}[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?fireGuaranteedCrit/);
assert.match(world, /resolveOfflineFlamestrikeBurst[\s\S]*?nextAuthoritativeRandomUnit\(state\) \*[\s\S]*?applyOfflineFireMageSpellHit/);
assert.match(world, /resolveOfflineFlamestrikeBurst[\s\S]*?entitySpellDamageMultipliers[\s\S]*?entityCritDamageSpellBonus/);
assert.match(world, /schemaVersion >= <uint>87[\s\S]*?entityCritDamageSpellBonus[\s\S]*?entitySpellDamageMultipliers/);
assert.match(world, /applyOfflineFireMageSpellHit[\s\S]*?flamestrikeAbilityCode\(\) \? "flamestrike"/);
assert.match(world, /applySupportedCastAtCommand[\s\S]*?isFlamestrike[\s\S]*?startOfflineFlamestrikeCast/);
assert.match(world, /completedAbility == flamestrikeAbilityCode\(\)[\s\S]*?completeOfflineFlamestrikeCast/);
assert.match(world, /pub flamestrikePositionRuntimeStateTest\(/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'flamestrike_position_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_flamestrike_position_runtime_tests.zrp');
assert.match(testMain, /state\.flamestrikePositionRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/flamestrike_position_runtime_test_main\.zr"/);
assert.match(world, /pub flamestrikePositionRuntimeStateTest\([\s\S]*?encodeState\(state\)[\s\S]*?entityCritDamageSpellBonus[\s\S]*?entitySpellDamageMultipliers/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '177-flamestrike-position-runtime.md');
assert.match(plan, /one shared crit unit/);
assert.match(plan, /does not create a persistent ground effect/);

process.stdout.write('WOS177 Flamestrike position runtime static guards passed\n');
