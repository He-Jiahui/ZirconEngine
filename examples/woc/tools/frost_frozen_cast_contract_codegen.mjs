import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/frost_mage.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'frost_frozen_cast_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'frost_frozen_cast_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['ICE_LANCE_FROZEN_MULT'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
invariant(source.includes("import { isRooted } from './cc';"), 'Frozen resolution no longer uses the canonical CC root predicate');
invariant(source.includes("export const WINTERS_CHILL_SPENDERS: ReadonlySet<string> = new Set(['ice_lance']);"), 'Winter\'s Chill spender set drifted');
invariant(source.includes("if (!target || ability.school === 'physical') return INERT_FROZEN;") && source.includes('if (!isCommittedFrost(ctx, meta)) return INERT_FROZEN;'), 'Frozen resolution admission ordering drifted');
invariant(source.includes("const lanceMult = ability.id === 'ice_lance' ? ICE_LANCE_FROZEN_MULT : 1;") && source.includes('if (isRooted(target)) return { treatAsFrozen: true, damageMult: lanceMult };'), 'Root priority or Ice Lance multiplier drifted');
invariant(source.includes("if (ability.id === 'ice_lance' && consumeFingersCharge(ctx, p))") && source.includes('if (WINTERS_CHILL_SPENDERS.has(ability.id) && consumeWintersChillCharge(ctx, target))'), 'Frozen charge-consumption ordering drifted');
invariant(source.includes("const left = (aura.stacks ?? 1) - 1;") && source.includes("const left = (aura.charges ?? 1) - 1;"), 'Frozen charge default semantics drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/frost_frozen_cast_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'frost_frozen_cast',
  constants,
  ordering: 'target_and_physical_gate; committed_frost_gate; rooted_without_spend; ice_lance_fingers; winters_chill',
  default_charges: 'missing_stacks_or_charges_means_one',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  'pub physicalSchool(): string { return "physical"; }\n' +
  'pub iceLanceId(): string { return "ice_lance"; }\n' +
  'pub fingersKind(): string { return "fingers_of_frost"; }\n' +
  'pub wintersChillKind(): string { return "winters_chill"; }\n' +
  `pub iceLanceFrozenMult(): float { return ${constants.ICE_LANCE_FROZEN_MULT}.0; }\n` +
  'pub isWintersChillSpender(abilityId: string): bool { return abilityId == iceLanceId(); }\n';
for (const [path, output, label] of [[jsonOutput, json, 'Frost frozen-cast JSON contract'], [zrOutput, zr, 'Frost frozen-cast Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:frost-frozen-cast-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:frost-frozen-cast-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Frost frozen-cast contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
