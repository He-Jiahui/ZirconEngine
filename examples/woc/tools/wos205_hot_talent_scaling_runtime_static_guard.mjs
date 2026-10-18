import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const readSource = (...parts) => readFileSync(join(sourceRoot, ...parts), 'utf8');
const scalingPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'combat', 'hot_talent_scaling_state.zr');

assert.match(
  readSource('src', 'sim', 'content', 'classes.ts'),
  /case 'hot':\s*return \{ \.\.\.eff, total: Math\.round\(eff\.total \* healMult \* hotMult \+ flat\) \};/,
);
assert.match(
  readSource('src', 'sim', 'combat', 'effect_dispatch.ts'),
  /const hotBase = Math\.max\(1, Math\.round\(eff\.total \/ \(eff\.duration \/ eff\.interval\)\)\);/,
);
assert.ok(existsSync(scalingPath), 'WOS205 hot talent scaling module is missing');

const scaling = read('scripts', 'woc_game', 'src', 'combat', 'hot_talent_scaling_state.zr');
const pure = read('scripts', 'woc_game', 'src', 'combat', 'pure_hot_profile_state.zr');
const hybrid = read('scripts', 'woc_game', 'src', 'combat', 'hybrid_hot_profile_state.zr');
const state = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const testMain = read('scripts', 'woc_game', 'src', 'world', 'hot_talent_runtime_test_main.zr');
const manifest = JSON.parse(read('scripts', 'woc_game', 'woc_m4_hot_talent_runtime_tests.zrp'));

assert.match(scaling, /pub resolveHotTotal\(/);
assert.match(scaling, /"healPct"/);
assert.match(scaling, /"hotHealPct"/);
assert.match(scaling, /"dmgPct"/);
assert.match(scaling, /"flatDmg"/);
assert.match(scaling, /resolveHotTotal\(116, talentState, "rejuvenation"\) == 176/);
assert.match(pure, /pub resolvePureHotProfileWithResolvedTotal\(/);
assert.match(pure, /pub pureHotProfileSnapshotMatches\(/);
assert.match(hybrid, /pub resolveHybridHotProfileWithResolvedTotal\(/);
assert.match(hybrid, /pub hybridHotProfileSnapshotMatches\(/);
assert.match(state, /var hotTalentScaling = %import\("combat\/hot_talent_scaling_state"\);/);
assert.match(state, /resolvePureHotProfileWithResolvedTotal\(/);
assert.match(state, /resolveHybridHotProfileWithResolvedTotal\(/);
assert.match(state, /pureHotProfiles\.pureHotProfileSnapshotMatches\(/);
assert.match(state, /hybridHotProfiles\.hybridHotProfileSnapshotMatches\(/);
assert.match(state, /pub hotTalentRuntimeStateTest\(\): int/);
assert.match(state, /offlineHotHeals\[0\] != 66/);
assert.match(state, /"regrowth"/);
assert.match(state, /"renew"/);
assert.match(state, /var regrowthDraws = regrowthState\.rngDraws[\s\S]*?regrowthRestored\.rngDraws != regrowthDraws/);
assert.match(state, /var renewDraws = renewState\.rngDraws[\s\S]*?renewRestored\.rngDraws != renewDraws/);
assert.doesNotMatch(state, /applyOfflineDirectHeal\([\s\S]{0,300}"hotHealPct"/);
assert.match(testMain, /state\.hotTalentRuntimeStateTest\(\)/);
assert.equal(manifest.backend, 'zr_vm:project');

process.stdout.write('WOS205 HoT talent scaling static guard passed\n');
