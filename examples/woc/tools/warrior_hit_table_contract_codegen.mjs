import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/warrior_hit_table.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'warrior_hit_table_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'warrior_hit_table_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const parryBase = literal(source, /const WARRIOR_PARRY_BASE\s*=\s*([\d.]+);/, 'warrior parry base');
const parryPerStrength = literal(source, /const WARRIOR_PARRY_PER_STRENGTH\s*=\s*([\d.]+);/, 'warrior parry per strength');
invariant(source.includes('const WARRIOR_FRONT_ARC = Math.PI / 2;'), 'warrior front arc drifted');
invariant(source.includes("defender.kind !== 'player' || defender.templateId !== 'warrior'") && source.includes('Math.abs(normAngle(angleTo(defender.pos, attacker.pos) - defender.facing)) < WARRIOR_FRONT_ARC'), 'warrior defense gate drifted');
invariant(source.includes('return Math.max(0, WARRIOR_PARRY_BASE + str * WARRIOR_PARRY_PER_STRENGTH);'), 'warrior parry formula drifted');
invariant(source.includes('defender.blockValue > 0 && defender.blockChance > 0 ? defender.blockChance : 0'), 'warrior block gate drifted');
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/warrior_hit_table_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  parry_base: parryBase,
  parry_per_strength: parryPerStrength,
  front_arc_radians: Math.PI / 2,
  front_arc: 'strictly_less_than_half_pi_after_angle_normalization',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub parryBase(): float { return ${document.parry_base}; }\n` +
  `pub parryPerStrength(): float { return ${document.parry_per_strength}; }\n` +
  `pub frontArc(): float { return ${document.front_arc_radians}; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'warrior hit table JSON contract'], [zrOutput, zr, 'warrior hit table Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:warrior-hit-table-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:warrior-hit-table-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} warrior hit table contract for ${SOURCE_COMMIT}\n`);

function literal(source, expression, label) { const match = source.match(expression); invariant(match, `${label} is no longer a literal contract`); return Number(match[1]); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
