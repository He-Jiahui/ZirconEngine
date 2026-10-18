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

const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
assert.match(lifecycle, /MAGE_DEFENSIVE_COOLDOWNS[\s\S]*?'blink'[\s\S]*?'ice_barrier'[\s\S]*?'blazing_barrier'[\s\S]*?'greater_invisibility'/);
assert.match(lifecycle, /OVERFLOW_CAP_SECONDS = 10[\s\S]*?OVERFLOW_CAP_WINDOW = 30/);
assert.match(lifecycle, /overflowingPowerCdr[\s\S]*?manaDefCdrPer10[\s\S]*?cost \/ p\.maxResource[\s\S]*?OVERFLOW_CAP_SECONDS - used/);
assert.match(lifecycle, /spendResource\(p, res\.cost\);[\s\S]*?overflowingPowerCdr\(ctx, p, meta, res\.cost\)/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityOverflowingPowerCapRemaining[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /overflowingPowerStateIsValid[\s\S]*?entityOverflowingPowerCapRemaining[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /spendOfflineAbilityResource[\s\S]*?applyOfflineOverflowingPowerCdr/);
const spendBridgeStart = world.indexOf('spendOfflineAbilityResource(state: WorldState');
const spendBridgeEnd = world.indexOf('\n}', spendBridgeStart);
assert.notEqual(spendBridgeStart, -1, 'the centralized resource bridge must remain present');
assert.notEqual(spendBridgeEnd, -1, 'the centralized resource bridge must remain closed');
const spendBridge = world.slice(spendBridgeStart, spendBridgeEnd + 2);
assert.match(spendBridge, /var precision = %import\("combat\/resource_precision_state"\);/);
assert.match(
  spendBridge,
  /new precision\.ExactResourceState\([\s\S]*?state\.entityResources\[casterIndex\][\s\S]*?state\.entityMaxResources\[casterIndex\]/,
);
assert.match(
  spendBridge,
  /precision\.canSpend\(exact, <float>cost\)[\s\S]*?precision\.spend\(exact, <float>cost\)\.current : 0\.0;/,
);
const directCasterResourceDebits = [
  ...world.matchAll(
    /state\.entityResources\[casterIndex\]\s*=\s*<int>state\.entityResources\[casterIndex\]\s*-\s*([^;]+);/g,
  ),
].map((match) => match[1].trim());
assert.deepEqual(
  directCasterResourceDebits,
  [],
  'resolver paths may not bypass the centralized clamping resource bridge',
);
assert.ok(
  (world.match(/spendOfflineAbilityResource\(\s*state,\s*casterIndex,/g) ?? []).length >= 84,
  'retained variable and fixed-cost ability debits must keep using the bridge',
);
assert.match(world, /applyOfflineOverflowingPowerCdr[\s\S]*?manaDefCdrPer10[\s\S]*?10\.0[\s\S]*?30\.0/);
assert.match(world, /applyOfflineOverflowingPowerCdr[\s\S]*?blink[\s\S]*?iceBarrier[\s\S]*?blazingBarrier[\s\S]*?greaterInvisibility/);
assert.match(world, /ageOfflineOverflowingPower[\s\S]*?0\.05[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /schemaVersion >= <uint>91[\s\S]*?entityOverflowingPowerCapRemaining[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /schemaVersion != <uint>91/);
assert.match(world, /return encodeStateVersion\(state, <uint>118\)/);
assert.match(world, /pub overflowingPowerRuntimeStateTest\(/);
assert.match(world, /overflowingPowerRuntimeStateTest[\s\S]*?encodeState\(state\)[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /overflowingPowerRuntimeStateTest[\s\S]*?startOfflineIceBarrierCast[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /overflowingPowerRuntimeStateTest[\s\S]*?completeTemporalReversalCast[\s\S]*?entityOverflowingPowerCapUsed/);
assert.match(world, /completeTemporalReversalCast[\s\S]*?offerOfflineTemporalReversal[\s\S]*?spendOfflineAbilityResource\([\s\S]*?temporalReversal\.resourceCost\(true\)[\s\S]*?setAbilityCooldownExpiration/);

const testMain = read('scripts', 'woc_game', 'src', 'world', 'overflowing_power_runtime_test_main.zr');
const testPackage = read('scripts', 'woc_game', 'woc_m4_overflowing_power_runtime_tests.zrp');
assert.match(testMain, /state\.overflowingPowerRuntimeStateTest\(\)/);
assert.match(testPackage, /backend\s*=\s*"zr_vm:project"/);
assert.match(testPackage, /entry\s*=\s*"src\/world\/overflowing_power_runtime_test_main\.zr"/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '187-overflowing-power-runtime.md');
assert.match(plan, /rolling 30-second internal-cooldown aura/);
assert.match(plan, /every current Mage mana[\s\S]*?ability/);

process.stdout.write('WOS187 Overflowing Power runtime static guards passed\n');
