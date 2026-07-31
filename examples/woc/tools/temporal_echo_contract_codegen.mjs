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
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'temporal_echo_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'temporal_echo_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['TEMPORAL_ECHO_DURATION', 'ECHO_CONVERT_SINGLE', 'ECHO_CONVERT_AOE', 'ECHO_GROUP_CONVERT_SINGLE', 'ECHO_GROUP_CONVERT_AOE'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
invariant(source.includes("export const TEMPORAL_ECHO_ID = 'temporal_echo';"), 'Temporal Echo id drifted');
invariant(source.includes('if (aoe) return a.echoGroup ? ECHO_GROUP_CONVERT_AOE : ECHO_CONVERT_AOE;') && source.includes('return a.echoConvertRate ?? (a.echoGroup ? ECHO_GROUP_CONVERT_SINGLE : ECHO_CONVERT_SINGLE);'), 'Temporal Echo rate rule drifted');
invariant(source.includes("if (a.kind !== 'temporal_echo') return true;") && source.includes('return a.sourceId === viewerId;'), 'Temporal Echo visibility rule drifted');
invariant(source.includes("a.kind === 'temporal_echo' && a.sourceId === mageId && !a.echoGroup") && source.includes("a.kind === 'temporal_echo' && a.sourceId === mageId)"), 'Temporal Echo strip rules drifted');
invariant(source.includes('stripIndividualEcho(ctx, caster.id);') && source.includes('echoGroup: false,') && source.includes('echoConvertRate: ECHO_CONVERT_SINGLE,'), 'Temporal Echo individual placement drifted');
invariant(source.includes('if (existing && !existing.echoGroup) {') && source.includes('if (existing.remaining < duration) existing.remaining = duration;') && source.includes('echoGroup: true,') && source.includes('echoConvertRate: ECHO_GROUP_CONVERT_SINGLE,'), 'Temporal Echo group placement drifted');
invariant(source.includes("school !== 'arcane' || dealt <= 0") && source.includes("a.kind === 'temporal_echo' && a.sourceId === source.id") && source.includes('break;'), 'Temporal Echo conversion target rule drifted');
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/temporal_echo_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  id: 'temporal_echo',
  constants,
  conversion: 'living_targets_in_entity_order; first_matching_mark_per_target; source_must_be_player_arcane_positive_damage',
  placement: 'individual_strips_only_own_non_group; group_preserves_own_individual_and_only_extends_to_duration',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub echoKind(): string { return \"${document.id}\"; }\n` +
  `pub duration(): float { return ${constants.TEMPORAL_ECHO_DURATION}.0; }\n` +
  `pub singleRate(): float { return ${constants.ECHO_CONVERT_SINGLE}; }\n` +
  `pub areaRate(): float { return ${constants.ECHO_CONVERT_AOE}; }\n` +
  `pub groupSingleRate(): float { return ${constants.ECHO_GROUP_CONVERT_SINGLE}; }\n` +
  `pub groupAreaRate(): float { return ${constants.ECHO_GROUP_CONVERT_AOE}; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'Temporal Echo JSON contract'], [zrOutput, zr, 'Temporal Echo Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:temporal-echo-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:temporal-echo-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Temporal Echo contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
