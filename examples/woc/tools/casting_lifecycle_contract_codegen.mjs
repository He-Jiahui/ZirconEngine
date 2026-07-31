import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CASTING_PATH = 'src/sim/combat/casting_lifecycle.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'casting_lifecycle_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'casting_lifecycle_contract.zr');
const checkOnly = process.argv.includes('--check');

const castingBlob = sourceBlob(CASTING_PATH);
const typesBlob = sourceBlob(TYPES_PATH);
const casting = castingBlob.toString('utf8');
const types = typesBlob.toString('utf8');
for (const statement of [
  'if (activeCast && isMassResurrectionAbility(activeCast.def)) {',
  'if (p.inCombat) {',
  'if (!hasDeadGroupMember(ctx, p)) {',
  'floes.value -= 1;',
  'if (floes.value <= 0) {',
  'p.castRemaining -= DT;',
  'p.castRemaining += CAST_PUSHBACK_SEC * factor;',
  'p.castRemaining - p.castTotal * CHANNEL_PUSHBACK_FRACTION * factor,',
]) invariant(casting.includes(statement), `casting lifecycle rule drifted: ${statement}`);
invariant(casting.indexOf('if (p.inCombat) {') < casting.indexOf('if (!hasDeadGroupMember(ctx, p)) {'), 'Mass Resurrection combat check must precede group-death check');
const tickRate = numberConstant(types, 'TICK_RATE');
const constants = {
  tick_rate: tickRate,
  dt: 1 / tickRate,
  cast_complete_eps: numberConstant(types, 'CAST_COMPLETE_EPS'),
  cast_pushback_sec: numberConstant(types, 'CAST_PUSHBACK_SEC'),
  channel_pushback_fraction: numberConstant(types, 'CHANNEL_PUSHBACK_FRACTION'),
};
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/casting_lifecycle_contract_codegen.mjs',
  source_blobs: {
    [CASTING_PATH]: createHash('sha256').update(castingBlob).digest('hex'),
    [TYPES_PATH]: createHash('sha256').update(typesBlob).digest('hex'),
  },
  id: 'casting_lifecycle',
  constants,
  mass_resurrection_order: 'stun; mass_resurrection_in_combat; mass_resurrection_no_dead_group_member; silence; school_lockout; tick',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub tickDelta(): float { return ${floatLiteral(constants.dt)}; }\n` +
  `pub castCompleteEpsilon(): float { return ${floatLiteral(constants.cast_complete_eps)}; }\n` +
  `pub castPushbackSeconds(): float { return ${floatLiteral(constants.cast_pushback_sec)}; }\n` +
  `pub channelPushbackFraction(): float { return ${floatLiteral(constants.channel_pushback_fraction)}; }\n` +
  'pub massResCombatReason(): int { return 1; }\n' +
  'pub massResNoDeadMemberReason(): int { return 2; }\n';
for (const [path, output, label] of [[jsonOutput, json, 'casting lifecycle JSON contract'], [zrOutput, zr, 'casting lifecycle Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:casting-lifecycle-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:casting-lifecycle-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} casting lifecycle contract for ${SOURCE_COMMIT}\n`);

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer' });
}

function numberConstant(source, name) {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([0-9.eE+-]+)`));
  invariant(match, `${name} is no longer a numeric literal`);
  return Number(match[1]);
}

function floatLiteral(value) {
  if (!Number.isFinite(value)) throw new Error(`non-finite generated float: ${value}`);
  const fixed = value.toFixed(9).replace(/0+$/, '').replace(/\.$/, '');
  return fixed.includes('.') ? fixed : `${fixed}.0`;
}

function invariant(condition, message) { if (!condition) throw new Error(message); }
