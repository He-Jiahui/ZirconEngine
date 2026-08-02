import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/chronomancy.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'aether_surge_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'aether_surge_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['AETHER_SURGE_MAX_CHARGES', 'AETHER_SURGE_DMG_PER_CHARGE', 'AETHER_SURGE_COST_PER_CHARGE', 'AETHER_SURGE_CHARGE_WINDOW', 'AETHER_SURGE_FREE_PROC_CHANCE', 'AETHER_SURGE_FREE_WINDOW', 'AETHER_DARTS_BONUS_PER_CHARGE', 'AETHER_DARTS_FULL_CHARGE_MISSILES', 'AETHER_SURGE_CAST_HASTE_PER_CHARGE', 'AETHER_SURGE_PROC_CAST_MULT', 'PERFECT_MOMENT_DURATION'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
invariant(source.includes("export const ARCANE_SURGE_ID = 'arcane_surge';") && source.includes("const AETHER_SURGE_FREE_ID = 'aether_surge_free';") && source.includes("export const PERFECT_MOMENT_ID = 'perfect_moment';"), 'Aether Surge identities drifted');
invariant(source.includes('return (1 + AETHER_SURGE_COST_PER_CHARGE) ** aetherSurgeStacks(e);') && source.includes('return 1 + AETHER_SURGE_DMG_PER_CHARGE * aetherSurgeStacks(e);'), 'Aether Surge multiplier rules drifted');
invariant(source.includes('const charges = Math.min(AETHER_SURGE_MAX_CHARGES, aetherSurgeStacks(e));') && source.includes('if (e.auras.some((a) => a.id === AETHER_SURGE_FREE_ID)) mult *= AETHER_SURGE_PROC_CAST_MULT;'), 'Aether Surge cast multiplier drifted');
invariant(source.includes('const next = Math.min(AETHER_SURGE_MAX_CHARGES, aetherSurgeStacks(caster) + 1);') && source.includes('value: next,') && source.includes('stacks: next,'), 'Aether Surge stack cap drifted');
invariant(source.includes("if (abilityId !== 'arcane_missiles') return;") && source.includes('aetherSurgeStacks(caster) >= AETHER_SURGE_MAX_CHARGES ? AETHER_DARTS_FULL_CHARGE_MISSILES : 0'), 'Aether Darts channel-start rule drifted');
invariant(source.includes('if (caster.aetherDartsConsumePending) {') && source.includes('if (!perfectMomentActive(caster)) {') && source.includes('const bolts = caster.aetherDartsTicks || ticks;') && source.includes('Math.round(total / bolts)'), 'Aether Darts first-landed rule drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/aether_surge_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'aether_surge',
  constants,
  charge_invariant: 'authoritative writers only produce integer charges capped at four',
  darts: 'first_landed_consumes_unless_perfect_moment; positive rounding is per actual missile count',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  'pub surgeId(): string { return "arcane_surge"; }\n' +
  'pub freeId(): string { return "aether_surge_free"; }\n' +
  'pub perfectMomentId(): string { return "perfect_moment"; }\n' +
  'pub dartsId(): string { return "arcane_missiles"; }\n' +
  `pub maxCharges(): int { return ${constants.AETHER_SURGE_MAX_CHARGES}; }\n` +
  `pub damagePerCharge(): float { return ${constants.AETHER_SURGE_DMG_PER_CHARGE}; }\n` +
  `pub chargeWindow(): float { return ${constants.AETHER_SURGE_CHARGE_WINDOW}.0; }\n` +
  `pub freeProcChance(): float { return ${constants.AETHER_SURGE_FREE_PROC_CHANCE}; }\n` +
  `pub freeWindow(): float { return ${constants.AETHER_SURGE_FREE_WINDOW}.0; }\n` +
  `pub dartsBonusPerCharge(): int { return ${constants.AETHER_DARTS_BONUS_PER_CHARGE}; }\n` +
  `pub fullChargeMissiles(): int { return ${constants.AETHER_DARTS_FULL_CHARGE_MISSILES}; }\n` +
  `pub castHastePerCharge(): float { return ${constants.AETHER_SURGE_CAST_HASTE_PER_CHARGE}; }\n` +
  `pub procCastMult(): float { return ${constants.AETHER_SURGE_PROC_CAST_MULT}; }\n` +
  `pub perfectMomentDuration(): float { return ${constants.PERFECT_MOMENT_DURATION}.0; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'Aether Surge JSON contract'], [zrOutput, zr, 'Aether Surge Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:aether-surge-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:aether-surge-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Aether Surge contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
