import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/cc.ts';
const ABILITIES_SOURCE_PATH = 'src/sim/content/classes.ts';
const TALENT_ABILITIES_V2_A_SOURCE_PATH = 'src/sim/content/talent_abilities_v2_a.ts';
const PLAYER_MOTION_SOURCE_PATH = 'src/sim/player_motion.ts';
const ENTITY_SOURCE_PATH = 'src/sim/entity.ts';
const EFFECT_DISPATCH_SOURCE_PATH = 'src/sim/combat/effect_dispatch.ts';
const EMPOWER_NEXT_SOURCE_PATH = 'src/sim/combat/empower_next.ts';
const AURAS_SOURCE_PATH = 'src/sim/combat/auras.ts';
const SIM_SOURCE_PATH = 'src/sim/sim.ts';
const TYPES_SOURCE_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'cc_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'cc_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const abilitiesBlob = execFileSync(
  'git',
  ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${ABILITIES_SOURCE_PATH}`],
  { encoding: 'buffer' },
);
const abilitiesSource = abilitiesBlob.toString('utf8');
const talentAbilitiesV2ABlob = execFileSync(
  'git',
  ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${TALENT_ABILITIES_V2_A_SOURCE_PATH}`],
  { encoding: 'buffer' },
);
const talentAbilitiesV2ASource = talentAbilitiesV2ABlob.toString('utf8');
const playerMotionBlob = execFileSync(
  'git',
  ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${PLAYER_MOTION_SOURCE_PATH}`],
  { encoding: 'buffer' },
);
const playerMotionSource = playerMotionBlob.toString('utf8');
const entityBlob = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${ENTITY_SOURCE_PATH}`], { encoding: 'buffer' },
);
const entitySource = entityBlob.toString('utf8');
const effectDispatchBlob = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${EFFECT_DISPATCH_SOURCE_PATH}`],
  { encoding: 'buffer' },
);
const effectDispatchSource = effectDispatchBlob.toString('utf8');
const empowerNextBlob = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${EMPOWER_NEXT_SOURCE_PATH}`],
  { encoding: 'buffer' },
);
const empowerNextSource = empowerNextBlob.toString('utf8');
const aurasBlob = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${AURAS_SOURCE_PATH}`], { encoding: 'buffer' },
);
const aurasSource = aurasBlob.toString('utf8');
const simBlob = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SIM_SOURCE_PATH}`], { encoding: 'buffer' },
);
const simSource = simBlob.toString('utf8');
const typesBlob = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${TYPES_SOURCE_PATH}`], { encoding: 'buffer' },
);
const typesSource = typesBlob.toString('utf8');
const kinds = {
  stunned: ['stun', 'stasis', 'incapacitate', 'polymorph'],
  root: 'root',
  slow: 'slow',
  silence: 'silence',
  blind: 'blind',
  disarm: 'disarm',
  lockout: 'lockout',
  tongues: 'tongues',
};
for (const kind of kinds.stunned) invariant(source.includes(`a.kind === '${kind}'`), `stun kind ${kind} drifted`);
for (const kind of [kinds.root, kinds.slow, kinds.silence, kinds.blind, kinds.disarm, kinds.lockout, kinds.tongues]) invariant(source.includes(`a.kind === '${kind}'`), `CC kind ${kind} drifted`);
invariant(source.includes("a.kind === 'blind' && a.value > bonus") && source.includes("a.kind === 'tongues') m = Math.max(m, a.value)"), 'CC maximum-value semantics drifted');
invariant(source.includes("a.kind === 'lockout' && a.school === school"), 'school lockout predicate drifted');
invariant(
  abilitiesSource.includes("id: 'ice_floes'") &&
    abilitiesSource.includes("kind: 'ice_floes'") &&
    abilitiesSource.includes('cast while moving'),
  'Ice Floes mobility source drifted',
);
invariant(
  abilitiesSource.includes("id: 'ghost_wolf'") &&
    abilitiesSource.includes("kind: 'buff_speed', value: 1.4, duration: 3600") &&
    playerMotionSource.includes("a.kind === 'buff_speed'") &&
    playerMotionSource.includes('speed = Math.max(speed, a.value)'),
  'Ghost Wolf speed-buff source drifted',
);
invariant(
  abilitiesSource.includes("id: 'demon_skin'") &&
    abilitiesSource.includes("kind: 'buff_armor', value: 30, duration: 1800"),
  'Demon Skin armor-buff source drifted',
);
invariant(
  abilitiesSource.includes("id: 'primal_reflexes'") &&
    abilitiesSource.includes("kind: 'buff_dodge', value: 0.5, duration: 6") &&
    entitySource.includes("a.kind === 'buff_dodge') bonusDodge += a.value") &&
    entitySource.includes('e.dodgeChance = Math.max(0, 0.05 + s.agi * 0.0005 + bonusDodge)'),
  'Primal Reflexes dodge-buff source drifted',
);
invariant(
  talentAbilitiesV2ASource.includes("id: 'deterrence'") &&
    talentAbilitiesV2ASource.includes("kind: 'buff_dodge', value: 0.25, duration: 10") &&
    talentAbilitiesV2ASource.includes("kind: 'buff_dr', value: 0.3, duration: 10") &&
    effectDispatchSource.includes("case 'selfBuff':"),
  'Deterrence damage-reduction source drifted',
);
invariant(
  abilitiesSource.includes("id: 'tigers_fury'") &&
    abilitiesSource.includes("kind: 'buff_ap', value: 40, duration: 6") &&
    effectDispatchSource.includes("case 'selfBuff':") &&
    effectDispatchSource.includes('ctx.applyAura(p, {') &&
    entitySource.includes("a.kind === 'buff_ap') bonusAp += a.value"),
  "Tiger's Fury attack-power aura source drifted",
);
const faerieFireReductionMatch = /export const FAERIE_FIRE_ARMOR_PCT = ([0-9.]+);/.exec(typesSource);
invariant(faerieFireReductionMatch, 'Faerie Fire armor-reduction constant drifted');
const faerieFireArmorReduction = Number(faerieFireReductionMatch[1]);
invariant(
  abilitiesSource.includes("id: 'faerie_fire'") &&
    abilitiesSource.includes("type: 'faerieFire', duration: 40") &&
    effectDispatchSource.includes("case 'faerieFire':") &&
    effectDispatchSource.includes("kind: 'faerie_fire'") &&
    effectDispatchSource.includes('value: 0') &&
    simSource.includes("else if (a.kind === 'faerie_fire')") &&
    simSource.includes('reductionPct = Math.max(reductionPct, FAERIE_FIRE_ARMOR_PCT)') &&
    faerieFireArmorReduction === 0.1,
  'Faerie Fire aura or max-combined armor source drifted',
);
invariant(
  typesSource.includes("aura.kind === 'buff_rage_gen') mult += aura.value") &&
    typesSource.includes("aura.kind === 'buff_reckless') mult += RECKLESSNESS_RAGE_GEN") &&
    typesSource.includes("aura.kind === 'battle_stance') mult += STANCE_RAGE_GEN"),
  'Rage-generation aura source semantics drifted',
);
  invariant(
      abilitiesSource.includes("id: 'cold_blood'") &&
    abilitiesSource.includes("kind: 'next_attack_crit', value: 1, duration: 60") &&
    effectDispatchSource.includes('consumeNextAttackCrit(ctx, p)') &&
    empowerNextSource.includes("consumeAuraKind(ctx, e, 'next_attack_crit')"),
    'Cold Blood next-attack-crit source semantics drifted',
  );
invariant(
  abilitiesSource.includes("id: 'blade_flurry'") &&
      abilitiesSource.includes("kind: 'buff_haste', value: 1.2, duration: 12") &&
      simSource.includes("if (a.kind === 'buff_haste') m /= a.value;"),
  'Blade Flurry haste source semantics drifted',
);
invariant(
  abilitiesSource.includes("id: 'hemorrhage'") &&
    abilitiesSource.includes("kind: 'bleed_vuln', value: 0.4, duration: 12") &&
    effectDispatchSource.includes("case 'applyDebuff'") &&
    aurasSource.includes("if (targetAura.kind === 'bleed_vuln') bleedAmp += pctValue(targetAura.value);"),
  'Hemorrhage bleed-vulnerability source semantics drifted',
);
const motionKindCodes = {
  stun: 1,
  stasis: 2,
  incapacitate: 3,
  polymorph: 4,
  root: 5,
  ice_floes: 6,
  slow: 7,
  buff_speed: 8,
  buff_armor: 9,
  buff_dodge: 10,
  faerie_fire: 11,
  buff_ap: 12,
  lockout: 13,
  buff_rage_gen: 14,
  buff_reckless: 15,
    battle_stance: 16,
  next_attack_crit: 17,
  buff_haste: 18,
  bleed_vuln: 19,
  buff_dr: 20,
  };
const document = {
  schema_version: 2,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/cc_contract_codegen.mjs',
  source_blobs: {
    [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex'),
    [ABILITIES_SOURCE_PATH]: createHash('sha256').update(abilitiesBlob).digest('hex'),
    [TALENT_ABILITIES_V2_A_SOURCE_PATH]: createHash('sha256').update(talentAbilitiesV2ABlob).digest('hex'),
    [PLAYER_MOTION_SOURCE_PATH]: createHash('sha256').update(playerMotionBlob).digest('hex'),
    [ENTITY_SOURCE_PATH]: createHash('sha256').update(entityBlob).digest('hex'),
    [EFFECT_DISPATCH_SOURCE_PATH]: createHash('sha256').update(effectDispatchBlob).digest('hex'),
    [EMPOWER_NEXT_SOURCE_PATH]: createHash('sha256').update(empowerNextBlob).digest('hex'),
    [AURAS_SOURCE_PATH]: createHash('sha256').update(aurasBlob).digest('hex'),
    [SIM_SOURCE_PATH]: createHash('sha256').update(simBlob).digest('hex'),
    [TYPES_SOURCE_PATH]: createHash('sha256').update(typesBlob).digest('hex'),
  },
  kinds,
  motion_kind_codes: motionKindCodes,
  rage_generation_aura_kinds: ['buff_rage_gen', 'buff_reckless', 'battle_stance'],
  faerie_fire_armor_reduction: faerieFireArmorReduction,
  blind: 'maximum_positive_value_or_zero',
  tongues: 'maximum_value_or_one',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub isStunnedKind(kind: string): bool { return kind == \"stun\" || kind == \"stasis\" || kind == \"incapacitate\" || kind == \"polymorph\"; }\n` +
  `pub isRootKind(kind: string): bool { return kind == \"root\"; }\n` +
  `pub isSlowKind(kind: string): bool { return kind == \"slow\"; }\n` +
  `pub isSilenceKind(kind: string): bool { return kind == \"silence\"; }\n` +
  `pub isBlindKind(kind: string): bool { return kind == \"blind\"; }\n` +
  `pub isDisarmKind(kind: string): bool { return kind == \"disarm\"; }\n` +
  `pub isLockoutKind(kind: string): bool { return kind == \"lockout\"; }\n` +
  `pub isTonguesKind(kind: string): bool { return kind == \"tongues\"; }\n`;
const motionZr =
  `pub motionAuraKindCode(kind: string): uint {\n` +
  Object.entries(motionKindCodes)
    .map(([kind, code]) => `    if (kind == ${JSON.stringify(kind)}) { return <uint>${code}; }`)
    .join('\n') +
  `\n    return <uint>0;\n}\n` +
  `pub isMotionAuraKindCode(code: uint): bool { return code >= <uint>1 && code <= <uint>${Object.keys(motionKindCodes).length}; }\n` +
  `pub isMotionStunnedKindCode(code: uint): bool {\n` +
  `    return code == <uint>${motionKindCodes.stun} || code == <uint>${motionKindCodes.stasis} ||\n` +
  `        code == <uint>${motionKindCodes.incapacitate} || code == <uint>${motionKindCodes.polymorph};\n}\n` +
  `pub isMotionRootedKindCode(code: uint): bool {\n` +
  `    return isMotionStunnedKindCode(code) || code == <uint>${motionKindCodes.root};\n}\n` +
  `pub isIceFloesKindCode(code: uint): bool { return code == <uint>${motionKindCodes.ice_floes}; }\n` +
  `pub isMotionSlowKindCode(code: uint): bool { return code == <uint>${motionKindCodes.slow}; }\n` +
  `pub isMotionSpeedBuffKindCode(code: uint): bool { return code == <uint>${motionKindCodes.buff_speed}; }\n`;
const faerieFireZr =
  `pub faerieFireArmorReduction(): float { return ${faerieFireArmorReduction}; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'CC JSON contract'], [zrOutput, zr + motionZr + faerieFireZr, 'CC Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:cc-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:cc-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} CC contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
