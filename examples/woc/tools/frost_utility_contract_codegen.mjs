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
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'frost_utility_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'frost_utility_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['BLIZZARD_ORB_CDR_PER_ENEMY', 'BLIZZARD_ORB_CDR_CAP', 'ICICLE_MAX'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
invariant(source.includes("a.kind === 'fingers_of_frost' && abilityId === 'ice_lance'") && source.includes("a.kind === 'brain_freeze' && abilityId === 'flurry'"), 'Frost proc glow rule drifted');
invariant(source.includes("const icicles = auras.find((a) => a.kind === 'icicles');") && source.includes('return icicles ? (icicles.stacks ?? 1) : 0;'), 'Icicle read rule drifted');
invariant(source.includes("return abilityId === 'flurry' && p.auras.some((a) => a.kind === 'brain_freeze');"), 'Brain Freeze cooldown bypass rule drifted');
invariant(source.includes("if (abilityId === 'blizzard') p.blizzardOrbCdr = 0;") && source.includes("if (abilityId !== 'blizzard' || struck <= 0) return;"), 'Blizzard admission or reset rule drifted');
invariant(source.includes('const spent = p.blizzardOrbCdr ?? 0;') && source.includes('const refund = Math.min(struck * BLIZZARD_ORB_CDR_PER_ENEMY, BLIZZARD_ORB_CDR_CAP - spent);') && source.includes("const cur = p.cooldowns.get('frozen_orb');") && source.includes("if (cur <= refund) p.cooldowns.delete('frozen_orb');"), 'Blizzard refund ordering drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/frost_utility_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'frost_utility',
  constants,
  blizzard_refund: 'positive_hits_only; budget_advances_before_optional_cooldown_mutation; refund_is_min(per_enemy_times_hits, cap_minus_spent)',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  'pub iceLanceId(): string { return "ice_lance"; }\n' +
  'pub flurryId(): string { return "flurry"; }\n' +
  'pub blizzardId(): string { return "blizzard"; }\n' +
  'pub frozenOrbId(): string { return "frozen_orb"; }\n' +
  'pub fingersKind(): string { return "fingers_of_frost"; }\n' +
  'pub brainFreezeKind(): string { return "brain_freeze"; }\n' +
  'pub iciclesKind(): string { return "icicles"; }\n' +
  `pub blizzardOrbCdrPerEnemy(): float { return ${constants.BLIZZARD_ORB_CDR_PER_ENEMY}; }\n` +
  `pub blizzardOrbCdrCap(): float { return ${constants.BLIZZARD_ORB_CDR_CAP}.0; }\n` +
  `pub icicleMax(): int { return ${constants.ICICLE_MAX}; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'Frost utility JSON contract'], [zrOutput, zr, 'Frost utility Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:frost-utility-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:frost-utility-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Frost utility contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
