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
  /combustion:[\s\S]*?cost: 100,[\s\S]*?cooldown: 120,[\s\S]*?offGcd: true,[\s\S]*?kind: 'combustion',[\s\S]*?duration: 10/,
);
const fireMage = source('src/sim/combat/fire_mage.ts');
assert.match(
  fireMage,
  /if \(!p\.auras\.some\(\(a\) => a\.kind === 'combustion'\)\)[\s\S]*?cd - COMBUSTION_CDR_PER_CRIT[\s\S]*?kind: 'next_cast_free'[\s\S]*?empowerAbilities: \[\.\.\.HOT_STREAK_SPENDERS\][\s\S]*?kind: 'next_cast_instant'/,
);
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
assert.match(
  lifecycle,
  /consumeFreeCostFor[\s\S]*?consumeNextCastInstant[\s\S]*?hasScopedNextCastInstant/,
);
assert.match(
  lifecycle,
  /canCastFree && !togglingOff && consumeFreeCostFor[\s\S]*?res = \{ \.\.\.res, cost: 0 \}/,
);

const empower = read('scripts', 'woc_game', 'src', 'combat', 'empower_next_state.zr');
assert.match(empower, /nextCastInstantAuraActive[\s\S]*?find\(auras,"next_cast_instant",id\)/);
assert.match(empower, /consumeNextCastInstant[\s\S]*?auras\.removeAt\(k\)/);
assert.match(empower, /nextCastInstantAuraActive\(a,"fireball"\)[\s\S]*?consumeNextCastInstant\(a,"pyroblast"\)/);

const modifiers = read('scripts', 'woc_game', 'src', 'combat', 'next_cast_modifier_state.zr');
assert.match(modifiers, /freeCostKind\(\): uint \{ return <uint>1; \}/);
assert.match(modifiers, /instantKind\(\): uint \{ return <uint>2; \}/);
assert.match(modifiers, /cheapCostKind\(\): uint \{ return <uint>3; \}/);
assert.match(modifiers, /A zero-length partition is intentionally global[\s\S]*?strictly ascending/);
assert.match(modifiers, /consumesOnSuccessfulCast[\s\S]*?supportedKind/);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
assert.match(world, /entityNextCastModifierOffsets[\s\S]*?nextCastModifierTargetAbilityCodes/);
assert.match(world, /nextCastModifierStateIsValid[\s\S]*?nextCastModifiers\.validModifier[\s\S]*?canonicalTargets/);
assert.match(world, /setNextCastModifier[\s\S]*?currentIdentityCode == identityCode[\s\S]*?appendNextCastModifierRow/);
assert.match(world, /consumeNextCastModifierForAbility[\s\S]*?removeNextCastModifierAt/);
assert.match(world, /ageOfflineNextCastModifiers[\s\S]*?remaining > 0\.05/);
const currentSchema = world.match(
  /encodeState\(state: WorldState\)[\s\S]*?encodeStateVersion\(state, <uint>(\d+)\)/,
);
assert.ok(currentSchema && Number(currentSchema[1]) >= 85, 'current schema must retain WOS174 state');
assert.match(world, /if \(schemaVersion >= <uint>85\)[\s\S]*?nextCastModifierTargetAbilityCodes/);
assert.match(world, /nextCastModifierPersistenceStateTest[\s\S]*?encodeStateVersion\(restored, <uint>84\)/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '174-fire-mage-next-cast-runtime.md');
assert.match(plan, /next-cast modifier row shape/);
assert.match(plan, /No hardcoded `hasCombustion = false`/);

for (const [fixture, entry] of [
  ['woc_m4_fire_mage_proc_world_state_tests.zrp', 'fire_mage_proc_state_test_main'],
  ['woc_m4_next_cast_modifier_world_state_tests.zrp', 'next_cast_modifier_state_test_main'],
]) {
  const manifest = read('scripts', 'woc_game', fixture);
  assert.match(manifest, /backend\s*=\s*"zr_vm:project"/);
  assert.match(manifest, new RegExp(`entry\\s*=\\s*"src/world/${entry}\\.zr"`));
}

process.stdout.write('WOS174 Fire Mage next-cast static guards passed\n');
