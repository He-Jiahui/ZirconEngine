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

const body = (text, signature) => {
  const escapedSignature = signature.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const match = new RegExp(
    `^[ \\t]*${escapedSignature}[^;{]*?\\)\\s*:\\s*[^\\n{]+\\{`, 'm',
  ).exec(text);
  assert.notEqual(match, null, `missing function definition: ${signature}`);
  const start = match.index;
  const open = text.indexOf('{', start);
  assert.notEqual(open, -1, `missing function opening brace: ${signature}`);
  let depth = 0;
  for (let index = open; index < text.length; index += 1) {
    if (text[index] === '{') depth += 1;
    if (text[index] === '}') {
      depth -= 1;
      if (depth === 0) return text.slice(open + 1, index);
    }
  }
  throw new Error(`unterminated function: ${signature}`);
};

const classes = source('src/sim/content/classes.ts');
assert.match(
  classes,
  /glacial_front:[\s\S]*?castTime: 2\.4,[\s\S]*?empowerStages: 4,[\s\S]*?cooldown: 12,[\s\S]*?slowMult: 0\.5,[\s\S]*?rootDuration: 1/,
);
assert.match(
  classes,
  /dragons_breath:[\s\S]*?empowerStages: 4,[\s\S]*?hotStreakOnce: true,[\s\S]*?incapacitateDuration: 3/,
);
assert.match(
  source('src/net/online.ts'),
  /releaseEmpoweredAbility\(abilityId: string\): void \{[\s\S]*?cmd: 'releaseEmpowered', ability: abilityId/,
);
assert.match(
  source('src/sim/combat/casting_lifecycle.ts'),
  /releaseEmpoweredAbility[\s\S]*?empoweredStageForProgress[\s\S]*?applyAbility\(ctx, p, meta, \{ \.\.\.res, empowerLevel: level \}\)/,
);
assert.match(
  source('src/sim/combat/effect_dispatch.ts'),
  /case 'empoweredCone':[\s\S]*?ctx\.rng\.chance\(ctx\.spellCrit\(p\)\)[\s\S]*?ctx\.rng\.range\(stage\.min, stage\.max\)[\s\S]*?if \(m\.dead\) continue;[\s\S]*?applyRootAura[\s\S]*?hotStreakOnce/,
);

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const testMain = read(
  'scripts', 'woc_game', 'src', 'world', 'empowered_cone_state_test_main.zr',
);
const testPackage = read(
  'scripts', 'woc_game', 'woc_m4_empowered_cone_world_state_tests.zrp',
);
assert.match(world, /var empoweredCone = %import\("combat\/empowered_cone_state"\);/);
assert.match(world, /var glacialFront = %import\("combat\/glacial_front_state"\);/);
assert.match(
  body(world, 'pub applyCommands('),
  /releaseEmpoweredCommandId\(true\)[\s\S]*?commandId == releaseEmpoweredCommand[\s\S]*?applyOfflineReleaseEmpoweredCommand/,
);
assert.match(
  body(world, 'applySupportedCastSlotCommand('),
  /glacialFrontAbilityCode\(\)[\s\S]*?startOfflineGlacialFrontCast/,
);
assert.match(
  body(world, 'applySupportedCastCommand('),
  /typedAbilityCode == glacialFrontAbilityCode\(\)[\s\S]*?startOfflineGlacialFrontCast/,
);
const glacialStart = body(world, 'startOfflineGlacialFrontCast(');
assert.match(glacialStart, /knownAbilityPartitionContains/);
assert.match(glacialStart, /abilityCooldownExpiresAt/);
assert.match(glacialStart, /empoweredConeContract\.castTime/);
assert.match(glacialStart, /cast\.armEmpowered[\s\S]*?stageCount/);
assert.match(
  body(world, 'applyOfflineReleaseEmpoweredCommand('),
  /m4AbilityCodeFromPayload[\s\S]*?abilityCode != glacialFrontAbilityCode\(\)[\s\S]*?motionAuraEntityIsStunned[\s\S]*?casting\.releaseEmpoweredAbility[\s\S]*?completeOfflineGlacialFrontCast/,
);
assert.match(
  body(world, 'completeOfflineGlacialFrontCast('),
  /setAbilityCooldownExpiration[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?nextAuthoritativeRandomUnit\(state\)[\s\S]*?empoweredCone\.resolveHit[\s\S]*?clearOfflineBreakableIncapacitateOnDamage[\s\S]*?motionAuraKindCode\("slow"\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FROST[\s\S]*?motionAuraKindCode\("root"\)[\s\S]*?OFFLINE_PROJECTILE_SCHOOL_FROST/,
);
assert.match(
  body(world, 'applyOfflineMotionAuraWithDetailsAndSchool('),
  /entityMotionAuraAbilityCodes\[auraIndex\] == abilityCode &&[\s\S]*?entityMotionAuraKindCodes\[auraIndex\] == kindCode/,
);
assert.match(
  body(world, 'stepRetainedCasting('),
  /completedAbility == glacialFrontAbilityCode\(\)[\s\S]*?completeOfflineGlacialFrontCast/,
);
assert.match(
  body(world, 'pub glacialFrontCommandStateTest('),
  /appendReleaseEmpoweredCommandForTest[\s\S]*?entityLastResolvedEmpowerLevel\[0\] != 2[\s\S]*?entityLastResolvedEmpowerLevel\[0\] != 4[\s\S]*?wrongBytes[\s\S]*?motionAuraKindCode\("stun"\)[\s\S]*?interruptedRelease/,
);
assert.match(testMain, /state\.glacialFrontCommandStateTest\(\)/);
assert.match(
  testPackage,
  /"entry": "world\/empowered_cone_state_test_main"/,
);

const plan = read('..', '..', 'docs', 'plans', 'woc', '172-empowered-cone-world-state.md');
assert.match(plan, /WOS172a integrates `glacial_front` only/);
assert.match(plan, /`dragons_breath` remains deferred/);

process.stdout.write('WOS172 Glacial Front WorldState static guards passed\n');
