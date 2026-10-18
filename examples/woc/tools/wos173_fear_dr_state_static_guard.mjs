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
assert.match(sim, /PVP_FEAR_DR_RESET = 60/);
assert.match(sim, /PVP_FEAR_DR_DURATIONS = \[8, 4, 2, 1\]/);
assert.match(
  sim,
  /source\.kind !== 'player' \|\| target\.kind !== 'player' \|\| !this\.isHostileTo\(source, target\)[\s\S]*?return duration[\s\S]*?category === 'fear'[\s\S]*?stage: stage \+ 1, resetAt: this\.time \+ reset[\s\S]*?PVP_FEAR_DR_DURATIONS\[Math\.min\(stage, PVP_FEAR_DR_DURATIONS\.length - 1\)\]/,
);

const fear = read('scripts', 'woc_game', 'src', 'combat', 'fear_dr_state.zr');
assert.match(fear, /appliesToPair[\s\S]*?sourceIsPlayer && targetIsPlayer && hostile/);
assert.match(fear, /storedResetAt > now \? storedStage : 0/);
assert.match(fear, /var resolvedDuration = 1\.0/);
assert.match(fear, /stage == 0[\s\S]*?resolvedDuration = 8\.0/);
assert.match(fear, /stage == 1[\s\S]*?resolvedDuration = 4\.0/);
assert.match(fear, /stage == 2[\s\S]*?resolvedDuration = 2\.0/);
assert.match(fear, /now \+ fearResetSeconds\(\)/);
assert.match(fear, /var npc = resolve[\s\S]*?var later = resolve[\s\S]*?var expired = resolve/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityFearDrStages[\s\S]*?entityFearDrResetAt/);
assert.match(world, /entityHeatingUpRemaining[\s\S]*?entityHotStreakRemaining/);
assert.match(world, /appendDefaultFireMageProcColumns\(this\)/);
const currentSchema = world.match(
  /encodeState\(state: WorldState\)[\s\S]*?encodeStateVersion\(state, <uint>(\d+)\)/,
);
assert.ok(currentSchema && Number(currentSchema[1]) >= 84, 'current schema must retain WOS173 state');
assert.match(world, /schemaVersion >= <uint>84[\s\S]*?entityHeatingUpRemaining\.add\(reader\.fixed6[\s\S]*?entityHotStreakRemaining\.add\(reader\.fixed6/);
assert.match(world, /historicalFireProcIndex[\s\S]*?entityHeatingUpRemaining\.add\(0\.0\)[\s\S]*?entityHotStreakRemaining\.add\(0\.0\)/);
assert.match(world, /ageOfflineFireMageProcs[\s\S]*?entityHeatingUpRemaining[\s\S]*?entityHotStreakRemaining/);
assert.match(world, /pub fireMageProcPersistenceStateTest\([\s\S]*?decodeState\(encodeState\(state\)\)[\s\S]*?ageOfflineFireMageProcs/);

const testMain = read('scripts', 'woc_game', 'src', 'combat', 'fear_dr_state_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_fear_dr_state_tests.zrp');
assert.match(testMain, /fearDr\.contractTest\(\)/);
assert.match(testPackage, /"entry": "combat\/fear_dr_state_test_main"/);
const fireProcTestMain = read(
  'scripts', 'woc_game', 'src', 'world', 'fire_mage_proc_state_test_main.zr',
);
const fireProcTestPackage = read(
  'scripts', 'woc_game', 'woc_m4_fire_mage_proc_world_state_tests.zrp',
);
assert.match(fireProcTestMain, /state\.fireMageProcPersistenceStateTest\(\)/);
assert.match(
  fireProcTestPackage,
  /entry\s*=\s*"src\/world\/fire_mage_proc_state_test_main\.zr"/,
);

const plan = read('..', '..', 'docs', 'plans', 'woc', '173-dragons-breath-state-foundations.md');
assert.match(plan, /`8\/4\/2\/1` fixed-duration ladder/);
assert.match(plan, /`entityFearDrStages` and[\s\S]*?`entityFearDrResetAt`/);

process.stdout.write('WOS173 fear DR static guards passed\n');
