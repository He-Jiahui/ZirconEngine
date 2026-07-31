import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/fire_mage.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'fire_mage_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'fire_mage_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['SCORCH_EXECUTE_HP', 'HEATING_UP_WINDOW', 'HOT_STREAK_DURATION', 'COMBUSTION_CDR_PER_CRIT', 'CAUTERIZE_ICD', 'CAUTERIZE_HEAL_FRAC', 'CAUTERIZE_BURN_PER_SEC', 'CAUTERIZE_BURN_DURATION', 'CAUTERIZE_FIRE_DMG_BONUS', 'IGNITE_DURATION', 'IGNITE_INTERVAL'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
const cauterizeHealDivisor = reciprocalInteger(constants.CAUTERIZE_HEAL_FRAC, 'CAUTERIZE_HEAL_FRAC');
const cauterizeBurnDivisor = reciprocalInteger(constants.CAUTERIZE_BURN_PER_SEC, 'CAUTERIZE_BURN_PER_SEC');
const igniteTickCount = quotientInteger(constants.IGNITE_DURATION, constants.IGNITE_INTERVAL, 'IGNITE_DURATION / IGNITE_INTERVAL');
invariant(source.includes("export const HOT_STREAK_BUILDERS: readonly string[] = [") && ['fireball', 'fire_blast', 'scorch', 'pyroblast', 'flamestrike', 'dragons_breath'].every((id) => source.includes(`'${id}'`)), 'Hot Streak builders drifted');
invariant(source.includes("export const HOT_STREAK_SPENDERS: readonly string[] = ['pyroblast', 'flamestrike'];"), 'Hot Streak spenders drifted');
invariant(source.includes("if (school !== 'fire') return false;") && source.includes("if (p.auras.some((a) => a.kind === 'combustion')) return true;") && source.includes("abilityId === 'scorch' && target && target.hp <= target.maxHp * SCORCH_EXECUTE_HP"), 'Fire guaranteed-crit rule drifted');
invariant(source.includes("if (!abilityId || !HOT_STREAK_BUILDERS.includes(abilityId)) return;") && source.includes('if (!crit) {') && source.includes("const heatingIdx = p.auras.findIndex((a) => a.id === 'heating_up');"), 'Hot Streak admission or break rule drifted');
invariant(source.includes("if (!p.auras.some((a) => a.kind === 'combustion')) {") && source.includes("const cd = p.cooldowns.get('combustion');") && source.includes('if (cd && cd > 0) p.cooldowns.set'), 'Combustion cooldown reduction rule drifted');
invariant(source.includes("id: 'hot_streak',") && source.includes("id: 'hot_streak_instant',") && source.includes('empowerAbilities: [...HOT_STREAK_SPENDERS]'), 'Hot Streak pair application drifted');
invariant(source.includes("if (!source || source === target || school !== 'fire') return 1;") && source.includes('1 + CAUTERIZE_FIRE_DMG_BONUS'), 'Cauterize fire multiplier drifted');
invariant(source.includes("if (target.kind !== 'player' || target.dead || incoming < target.hp) return null;") && source.includes("if (target.auras.some((a) => a.kind === 'cauterize_fatigue')) return null;") && source.includes('target.hp = Math.max(1, Math.round(target.maxHp * CAUTERIZE_HEAL_FRAC));') && source.includes('Math.max(1, Math.round(target.maxHp * CAUTERIZE_BURN_PER_SEC))'), 'Cauterize lethal-save rule drifted');
invariant(source.includes('if (burn <= 0 || target.dead) return;') && source.includes('const perTick = Math.max(1, Math.round(burn / (IGNITE_DURATION / IGNITE_INTERVAL)));') && source.includes("a.id === 'ignite' && a.sourceId === source.id") && source.includes('existing.value += perTick;'), 'Ignite stack/refresh rule drifted');
invariant(source.includes("if (spec === 'arcane') return 'temporal_barrier';") && source.includes("if (spec === 'fire') return 'blazing_barrier';") && source.includes("if (spec === 'frost') return 'ice_barrier';"), 'Personal barrier slot mapping drifted');
invariant(source.includes("if (!crit || amount <= 0 || ability === null || school !== 'fire') return;") && source.includes('if (!source || source.id === target.id) return;') && source.includes('if (!mods || mods.global.ignitionPct <= 0) return;') && source.includes('applyIgnite(ctx, source, target, Math.round(amount * mods.global.ignitionPct));'), 'Ignite crit admission drifted');
invariant(source.includes('return HOT_STREAK_SPENDERS.includes(abilityId) && ABILITIES[abilityId] !== undefined;'), 'Fire spender content-existence rule drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/fire_mage_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'fire_mage',
  constants,
  hot_streak_builders: ['fireball', 'fire_blast', 'scorch', 'pyroblast', 'flamestrike', 'dragons_breath'],
  hot_streak_spenders: ['pyroblast', 'flamestrike'],
  ordering: 'eligible_fire_builder; noncrit_removes_heating; crit_reduces_combustion_outside_combustion; first_crit_heating; second_crit_hot_streak_pair',
  cauterize: 'lethal_inclusive; fatigue_blocks; heal_to_quarter; burn_per_second_is_rounded_positive_twentieth',
  ignite: 'dead_or_nonpositive_burn_inert; rounded_positive_third_per_tick; owned_stack_adds_then_refreshes',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  'pub fireSchool(): string { return "fire"; }\n' +
  'pub combustionKind(): string { return "combustion"; }\n' +
  'pub combustionId(): string { return "combustion"; }\n' +
  'pub heatingUpId(): string { return "heating_up"; }\n' +
  'pub hotStreakId(): string { return "hot_streak"; }\n' +
  'pub hotStreakInstantId(): string { return "hot_streak_instant"; }\n' +
  'pub temporalBarrierId(): string { return "temporal_barrier"; }\n' +
  'pub blazingBarrierId(): string { return "blazing_barrier"; }\n' +
  'pub iceBarrierId(): string { return "ice_barrier"; }\n' +
  'pub cauterizeFatigueKind(): string { return "cauterize_fatigue"; }\n' +
  'pub cauterizingId(): string { return "cauterizing"; }\n' +
  'pub igniteId(): string { return "ignite"; }\n' +
  `pub scorchExecuteHp(): float { return ${constants.SCORCH_EXECUTE_HP}; }\n` +
  `pub heatingUpWindow(): float { return ${constants.HEATING_UP_WINDOW}.0; }\n` +
  `pub hotStreakDuration(): float { return ${constants.HOT_STREAK_DURATION}.0; }\n` +
  `pub combustionCdrPerCrit(): float { return ${constants.COMBUSTION_CDR_PER_CRIT}.0; }\n` +
  `pub cauterizeIcd(): float { return ${constants.CAUTERIZE_ICD}.0; }\n` +
  `pub cauterizeBurnDuration(): float { return ${constants.CAUTERIZE_BURN_DURATION}.0; }\n` +
  `pub cauterizeHealDivisor(): int { return ${cauterizeHealDivisor}; }\n` +
  `pub cauterizeBurnDivisor(): int { return ${cauterizeBurnDivisor}; }\n` +
  `pub cauterizeFireDamageBonus(): float { return ${constants.CAUTERIZE_FIRE_DMG_BONUS}; }\n` +
  `pub igniteDuration(): float { return ${constants.IGNITE_DURATION}.0; }\n` +
  `pub igniteInterval(): float { return ${constants.IGNITE_INTERVAL}.0; }\n` +
  `pub igniteTickCount(): int { return ${igniteTickCount}; }\n` +
  'pub isHotStreakBuilder(abilityId: string): bool { return abilityId == "fireball" || abilityId == "fire_blast" || abilityId == "scorch" || abilityId == "pyroblast" || abilityId == "flamestrike" || abilityId == "dragons_breath"; }\n' +
  'pub isHotStreakSpender(abilityId: string): bool { return abilityId == "pyroblast" || abilityId == "flamestrike"; }\n';
for (const [path, output, label] of [[jsonOutput, json, 'Fire Mage JSON contract'], [zrOutput, zr, 'Fire Mage Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:fire-mage-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:fire-mage-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Fire Mage contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
function reciprocalInteger(value, label) { return quotientInteger(1, value, `1 / ${label}`); }
function quotientInteger(numerator, denominator, label) {
  const quotient = numerator / denominator;
  invariant(Number.isInteger(quotient) && quotient > 0, `${label} must remain a positive integer ratio for exact integer projection`);
  return quotient;
}
