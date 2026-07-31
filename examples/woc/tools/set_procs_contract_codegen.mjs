import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/set_procs.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'set_procs_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'set_procs_contract.zr');
const checkOnly = process.argv.includes('--check');

const blobs = Object.fromEntries([SOURCE_PATH, TYPES_PATH].map((path) => [path, execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer' })]));
const source = blobs[SOURCE_PATH].toString('utf8');
const types = blobs[TYPES_PATH].toString('utf8');
for (const field of ['trigger', 'chance', 'aura', 'duration', 'value?', 'icd?', 'applyTo?', 'tickInterval?', 'maxStacks?', 'school?']) invariant(types.includes(field), `SetProc field ${field} drifted`);
invariant(source.includes('const matching = source.setProcs.filter((proc) => proc.trigger === trigger);'), 'set proc trigger filtering drifted');
invariant(source.includes('if (proc.icd && ctx.time < (source.procReadyAt[proc.id] ?? 0)) continue;') && source.includes("const recipient = proc.applyTo === 'target' ? target : source;") && source.includes('if (!recipient || recipient.dead) continue;') && source.includes('if (!ctx.rng.chance(proc.chance)) continue;'), 'set proc RNG eligibility ordering drifted');
invariant(source.includes('source.procReadyAt[proc.id] = ctx.time + (proc.icd ?? 0);'), 'set proc cooldown write drifted');
invariant(source.includes('const base = proc.value ?? 0;') && source.includes('if (proc.maxStacks) {') && source.includes('stacks = Math.min(proc.maxStacks, (existing?.stacks ?? 0) + 1);'), 'set proc stack calculation drifted');
invariant(source.includes("...(proc.tickInterval !== undefined ? { tickInterval: proc.tickInterval } : {})") && source.includes("...(stacks !== undefined ? { stacks } : {})") && source.includes("school: proc.school ?? 'arcane',"), 'set proc aura projection drifted');
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/set_procs_contract_codegen.mjs',
  source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, createHash('sha256').update(value).digest('hex')])),
  trigger_values: ['spellCast', 'weaponCrit', 'spellCrit', 'kill'],
  default_school: 'arcane',
  rng_order: 'trigger_then_icd_then_recipient_liveness_then_one_chance_draw_then_cooldown_and_aura',
  stack_rule: 'first_matching_proc_id_and_source_id; min(maxStacks, previous_or_zero_plus_one)',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\npub defaultSchool(): string { return \"${document.default_school}\"; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'set proc JSON contract'], [zrOutput, zr, 'set proc Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:set-procs-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:set-procs-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} set proc contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
