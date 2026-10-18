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

const choices = source('src/sim/content/choice_rows_classic.ts');
assert.match(
  choices,
  /id: 'mag_r8_warded'[\s\S]*?global: \{ barrierDrPct: 0\.15 \}[\s\S]*?id: 'mag_warded'[\s\S]*?trigger: \{ on: 'shieldConsumed', ability: 'personal_barrier' \}[\s\S]*?kind: 'heal', amount: 39/,
);
const damage = source('src/sim/combat/damage.ts');
assert.match(
  damage,
  /Warded[\s\S]*?PERSONAL_BARRIER_IDS\.includes\(a\.id\)[\s\S]*?barrierDrPct[\s\S]*?Math\.round\(amount \* Math\.max\(0, 1 - wardedCut\)\)[\s\S]*?kind !== 'absorb'/,
);
const procs = source('src/sim/combat/talent_procs.ts');
assert.match(procs, /onShieldConsumed[\s\S]*?PERSONAL_BARRIER_IDS\.includes\(shieldAbilityId\)[\s\S]*?fire\(ctx, player, def, owner\)/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /offlineTalentModifiersForEntity[\s\S]*?talentModifiers\.recompute/);
assert.match(world, /offlineWardedPersonalBarrierIsActive[\s\S]*?iceBarrierAbilityCode\(\)/);
assert.match(world, /offlineWardedIncomingDamage[\s\S]*?barrierDrPct[\s\S]*?damage \* multiplier \+ 0\.5/);
assert.match(world, /wardedTalentProcIndex[\s\S]*?mag_warded/);
assert.match(world, /applyOfflineWardedHeal[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?healState\.applyHeal[\s\S]*?mobs\.healerThreat/);
assert.match(world, /applyOfflineWardedShieldConsumed[\s\S]*?catalogEntryMatchesSelection[\s\S]*?onShieldConsumed[\s\S]*?planResponse/);
assert.match(world, /applyOfflineAbsorbDamage[\s\S]*?offlineWardedIncomingDamage[\s\S]*?removeOfflineAbsorbAt[\s\S]*?applyOfflineWardedShieldConsumed/);
assert.match(world, /pub wardedIceBarrierRuntimeStateTest\([\s\S]*?mag_r8_warded[\s\S]*?applyOfflineAbsorbDamage\(state, 0, 200\) != 40[\s\S]*?entityHp\[0\] != 59[\s\S]*?rngDraws != <uint>1[\s\S]*?29\.5[\s\S]*?applyOfflineAbsorbDamage\(unwarded, 0, 200\) != 70[\s\S]*?decodeState\(encodeState\(snapshot\)\)/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'warded_ice_barrier_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_warded_ice_barrier_runtime_tests.zrp');
assert.match(testMain, /state\.wardedIceBarrierRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/warded_ice_barrier_runtime_test_main\.zr"/);
const plan = read('..', '..', 'docs', 'plans', 'woc', '185-warded-ice-barrier-runtime.md');
assert.match(plan, /`zr_vm:project`/);

process.stdout.write('WOS185 Warded Ice Barrier runtime static guards passed\n');
