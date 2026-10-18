import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = path.resolve(root, '..', '..');
const sourceRoot = path.resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), 'utf8');
const source = (file) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${file}`], { encoding: 'utf8' },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const classes = source('src/sim/content/classes.ts');
const coldBloodStart = classes.indexOf('  cold_blood: {');
const coldBloodNext = classes.slice(coldBloodStart + 1).search(/\n  [a-z_]+: \{/);
if (coldBloodStart < 0 || coldBloodNext < 0) {
  throw new Error('source Cold Blood definition is missing');
}
const coldBloodSource = classes.slice(coldBloodStart, coldBloodStart + 1 + coldBloodNext);
for (const [pattern, message] of [
  [/class: 'rogue'/, 'class'],
  [/learnLevel: 10/, 'learn level'],
  [/cost: 0,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 120,[\s\S]*?range: 0/,
    'cost/cast/cooldown/range'],
  [/school: 'physical',[\s\S]*?requiresTarget: false/, 'school and target mode'],
  [/type: 'selfBuff', kind: 'next_attack_crit', value: 1, duration: 60/,
    'next-attack-crit effect'],
]) {
  requireText(coldBloodSource, pattern, `source Cold Blood ${message} drifted`);
}
if (/offGcd:\s*true/.test(coldBloodSource)) {
  throw new Error('source Cold Blood is not an off-GCD ability');
}

const dispatch = source('src/sim/combat/effect_dispatch.ts');
const autoAttack = source('src/sim/combat/auto_attack.ts');
const lifecycle = source('src/sim/combat/casting_lifecycle.ts');
for (const [text, pattern, message] of [
  [dispatch, /case 'directDamage':[\s\S]*?consumeNextAttackCrit\(ctx, p\)/,
    'direct-damage consumption'],
  [dispatch, /case 'finisherDamage':[\s\S]*?consumeNextAttackCrit\(ctx, p\)/,
    'finisher consumption'],
  [dispatch, /case 'judgement':[\s\S]*?consumeNextAttackCrit\(ctx, p\)/,
    'Judgement consumption'],
  [autoAttack, /consumeNextAttackCrit\(ctx, atk\)/, 'ranged auto-attack consumption'],
  [autoAttack, /consumeNextAttackCrit\(ctx, attacker\)/, 'melee auto-attack consumption'],
  [lifecycle, /consumeNextAttackCrit\(ctx, src\)/, 'channel direct-damage consumption'],
]) {
  requireText(text, pattern, `source ${message} drifted`);
}

const contractGenerator = read('tools', 'cc_contract_codegen.mjs');
const m4Generator = read('tools', 'm4_ability_codegen.mjs');
const m4ZrGenerator = read('tools', 'm4_ability_zr_codegen.mjs');
requireText(contractGenerator, /next_attack_crit:\s*17/, 'CC contract misses next_attack_crit');
requireText(m4Generator, /'cold_blood'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/,
  'Cold Blood is absent from the M4 projection');
requireText(m4ZrGenerator, /document\.entries\.length === 117/,
  'M4 Zr projection count did not retain Cold Blood');

const contract = JSON.parse(read('reference', 'current-head', 'cc_contract.json'));
if (contract.motion_kind_codes?.next_attack_crit !== 17) {
  throw new Error('generated CC contract next_attack_crit code drifted');
}
const m4 = JSON.parse(read('contracts', 'm4_abilities.json'));
const coldBlood = m4.entries.find((entry) => entry.id === 'cold_blood');
if (m4.entries.length !== 117 || !coldBlood || coldBlood.index !== 104 ||
    coldBlood.definition.class !== 'rogue' || coldBlood.definition.learnLevel !== 10 ||
    coldBlood.definition.cost !== 0 || coldBlood.definition.castTime !== 0 ||
    coldBlood.definition.cooldown !== 120 || coldBlood.definition.range !== 0 ||
    coldBlood.definition.school !== 'physical' || coldBlood.definition.requiresTarget ||
    coldBlood.definition.offGcd || coldBlood.definition.effects?.[0]?.type !== 'selfBuff' ||
    coldBlood.definition.effects?.[0]?.kind !== 'next_attack_crit' ||
    coldBlood.definition.effects?.[0]?.value !== 1 ||
    coldBlood.definition.effects?.[0]?.duration !== 60) {
  throw new Error('Cold Blood M4 projection drifted');
}

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /coldBloodAbilityCode[\s\S]*?coldBloodPayloadAbilityIsExact[\s\S]*?coldBloodProfileIsValid/,
  'Cold Blood identity/profile boundary is missing');
requireText(world, /startOfflineColdBloodCast[\s\S]*?catalogAdmission[\s\S]*?coldBloodProfileIsValid[\s\S]*?ghostWolfGlobalCooldownSeconds[\s\S]*?setAbilityCooldownExpiration[\s\S]*?applyOfflineMotionAuraWithDetails/,
  'Cold Blood must use normal-GCD admission, cooldown, and a motion aura');
requireText(world, /applySupportedCastSlotCommand[\s\S]*?coldBloodAbilityCode\(\)[\s\S]*?startOfflineColdBloodCast[\s\S]*?applySupportedCastCommand[\s\S]*?coldBloodPayloadAbilityIsExact[\s\S]*?startOfflineColdBloodCast/,
  'Cold Blood slot and typed routes are missing');
requireText(world, /offlineNextAttackCritAuraIndex[\s\S]*?consumeOfflineNextAttackCrit[\s\S]*?offlineDamageCritChance[\s\S]*?offlinePhysicalCritical/,
  'Cold Blood single-consumption boundary is missing');
requireText(world, /offlinePhysicalCritical[\s\S]*?var forced = consumeOfflineNextAttackCrit[\s\S]*?nextAuthoritativeRandomUnit[\s\S]*?return forced \|\| rolled/,
  'physical next-attack crit must consume before retaining its critical-roll draw');
requireText(world, /prepareOfflineAutoActor[\s\S]*?actor\.nextAttackCrit = offlineNextAttackCritIsActive[\s\S]*?consumeOfflineNextAttackCritAfterAutoHit/,
  'Cold Blood auto-attack projection/consumption is missing');
requireText(world, /numeric\.source\.spellCritChance = offlineDamageCritChance/,
  'Cold Blood Judgement consumption is missing');
if (/spell\.critChance = offlineSpellCritChance\(state, casterIndex\)/.test(world)) {
  throw new Error('a retained direct-damage spell path bypasses Cold Blood consumption');
}
requireText(world, /pub coldBloodRuntimeStateTest[\s\S]*?appendCastSlotCommand[\s\S]*?appendTypedCastCommandForTest[\s\S]*?offlineDamageCritChance[\s\S]*?offlinePhysicalCritical[\s\S]*?autoAttackState\.meleeSwing[\s\S]*?autoAttackState\.fixedTick[\s\S]*?startOfflineAetherSurgeCast[\s\S]*?completeOfflineAetherSurgeCast[\s\S]*?startOfflineEviscerateCast[\s\S]*?startOfflineJudgementCast[\s\S]*?ageMotionAurasForEntity[\s\S]*?appendCancelAuraCommandForTest/,
  'Cold Blood runtime coverage is incomplete');

const fixture = read('scripts', 'woc_game', 'woc_m4_cold_blood_runtime_tests.zrp');
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/cold_blood_runtime_test_main.zr"')) {
  throw new Error('Cold Blood ZrVM project fixture is missing');
}

process.stdout.write(`WOS230 Cold Blood static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
