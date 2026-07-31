import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/equip_procs.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'equip_procs_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'equip_procs_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const requiredStatements = [
  'if (target.dead) return;',
  'const id = wielder.mainhandItemId;',
  'if (!id) return;',
  'const item = ITEMS[id];',
  "if (item?.kind !== 'weapon' || !item.weaponProcs) return;",
  'if (!meetsLevelRequirement(wielder.level, item)) return;',
  'for (const proc of procs) {',
  'if (proc.trigger !== trigger) continue;',
  'if (!ctx.rng.chance(proc.chance)) continue;',
  'for (const eff of proc.effects) fireEffect(ctx, wielder, target, proc, eff);',
  "case 'chainArc': {",
  "case 'attackSlow':",
  "case 'dot':",
  "case 'hot':",
];
for (const statement of requiredStatements) invariant(source.includes(statement), `equip-procs ordering or effect route drifted: ${statement}`);
invariant(source.indexOf('if (target.dead) return;') < source.indexOf('const id = wielder.mainhandItemId;'), 'dead target must short-circuit before equipment lookup');
invariant(source.indexOf('if (proc.trigger !== trigger) continue;') < source.indexOf('if (!ctx.rng.chance(proc.chance)) continue;'), 'trigger filter must precede RNG');

const effectKinds = ['chainArc', 'attackSlow', 'dot', 'hot'];
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/equip_procs_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'equip_procs',
  effect_kinds: effectKinds,
  rng_order: 'target_alive; mainhand_id; item_weapon_with_procs; level_gate; matching_trigger; one_chance_draw_per_matching_proc; effects_in_content_order',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  'pub chainArcKind(): string { return "chainArc"; }\n' +
  'pub attackSlowKind(): string { return "attackSlow"; }\n' +
  'pub dotKind(): string { return "dot"; }\n' +
  'pub hotKind(): string { return "hot"; }\n' +
  'pub chainArcRoute(): int { return 1; }\n' +
  'pub attackSlowRoute(): int { return 2; }\n' +
  'pub dotRoute(): int { return 3; }\n' +
  'pub hotRoute(): int { return 4; }\n';
for (const [path, output, label] of [[jsonOutput, json, 'equip-procs JSON contract'], [zrOutput, zr, 'equip-procs Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:equip-procs-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:equip-procs-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} equip-procs contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
