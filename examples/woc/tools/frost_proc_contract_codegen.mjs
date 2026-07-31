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
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'frost_proc_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'frost_proc_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['FINGERS_OF_FROST_CHANCE', 'FINGERS_OF_FROST_MAX_STACKS', 'FINGERS_OF_FROST_DURATION', 'BRAIN_FREEZE_CHANCE', 'BRAIN_FREEZE_DURATION', 'BRAIN_FREEZE_FLURRY_MULT', 'WINTERS_CHILL_CHARGES', 'WINTERS_CHILL_DURATION', 'ICICLE_MAX', 'ICICLE_DURATION'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
invariant(source.includes("const existing = p.auras.find((a) => a.kind === 'fingers_of_frost');") && source.includes('if ((existing.stacks ?? 1) >= FINGERS_OF_FROST_MAX_STACKS) return;') && source.includes('existing.stacks = (existing.stacks ?? 1) + 1;'), 'Fingers of Frost cap/update rule drifted');
invariant(source.includes("if (p.auras.some((a) => a.kind === 'brain_freeze')) return;") && source.includes("id: 'brain_freeze',"), 'Brain Freeze anti-refresh rule drifted');
invariant(source.includes("const existing = p.auras.find((a) => a.kind === 'icicles');") && source.includes('existing.remaining = ICICLE_DURATION;') && source.includes('if ((existing.stacks ?? 1) >= ICICLE_MAX) return;'), 'Icicle refresh/cap rule drifted');
invariant(source.includes('const fingers = ctx.rng.chance(FINGERS_OF_FROST_CHANCE);') && source.includes('const brain = ctx.rng.chance(BRAIN_FREEZE_CHANCE);') && source.indexOf('const fingers = ctx.rng.chance') < source.indexOf('const brain = ctx.rng.chance'), 'Frostbolt draw ordering drifted');
invariant(source.includes("const existing = target.auras.find((a) => a.id === 'winters_chill');") && source.includes('existing.charges = WINTERS_CHILL_CHARGES;') && source.includes('existing.sourceId = p.id;'), 'Winter\'s Chill reset rule drifted');
invariant(source.includes("if (ability.class !== 'mage' || p.kind !== 'player') return;") && source.includes("if (ability.id === 'frostbolt') {") && source.includes("} else if (ability.id === 'flurry' && isCommittedFrost(ctx, meta)) {") && source.includes('if (target && !target.dead) applyWintersChill(ctx, p, target);'), 'Frost after-cast routing drifted');
invariant(source.includes("if (res.def.id !== 'flurry') return res;") && source.includes("const idx = p.auras.findIndex((a) => a.kind === 'brain_freeze');") && source.includes('castTime: 0,') && source.includes('cooldown: 0,') && source.includes('Math.round(eff.min * BRAIN_FREEZE_FLURRY_MULT)'), 'Brain Freeze override rule drifted');

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/frost_proc_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'frost_proc',
  constants,
  draw_order: 'committed_frost_only; exactly_two_draws; fingers_then_brain; capped_or_active_results_are_discarded_after_draw',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  'pub fingersKind(): string { return "fingers_of_frost"; }\n' +
  'pub brainFreezeKind(): string { return "brain_freeze"; }\n' +
  'pub iciclesKind(): string { return "icicles"; }\n' +
  'pub wintersChillId(): string { return "winters_chill"; }\n' +
  `pub fingersMaxStacks(): int { return ${constants.FINGERS_OF_FROST_MAX_STACKS}; }\n` +
  `pub fingersDuration(): float { return ${constants.FINGERS_OF_FROST_DURATION}.0; }\n` +
  `pub brainFreezeDuration(): float { return ${constants.BRAIN_FREEZE_DURATION}.0; }\n` +
  `pub brainFreezeFlurryMult(): float { return ${constants.BRAIN_FREEZE_FLURRY_MULT}; }\n` +
  `pub icicleMax(): int { return ${constants.ICICLE_MAX}; }\n` +
  `pub icicleDuration(): float { return ${constants.ICICLE_DURATION}.0; }\n` +
  `pub wintersChillCharges(): int { return ${constants.WINTERS_CHILL_CHARGES}; }\n` +
  `pub wintersChillDuration(): float { return ${constants.WINTERS_CHILL_DURATION}.0; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'Frost proc JSON contract'], [zrOutput, zr, 'Frost proc Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:frost-proc-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:frost-proc-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Frost proc contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
