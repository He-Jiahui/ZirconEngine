import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/area_echo.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'area_echo_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'area_echo_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
const constants = Object.fromEntries(['AOE_ECHO_RADIUS', 'AOE_ECHO_MULT', 'AOE_ECHO_MAX_TARGETS', 'SWEEP_MULT'].map((name) => {
  const match = source.match(new RegExp(`export const ${name}\\s*=\\s*([\\d.]+);`));
  invariant(match, `${name} is no longer a literal`);
  return [name, Number(match[1])];
}));
for (const type of ['weaponStrike', 'directDamage', 'aoeDamage', 'aoeRoot', 'groundAoE']) invariant(source.includes(`effect.type === '${type}'`), `area echo effect type ${type} drifted`);
invariant(source.includes("aura.kind === 'aoe_echo'") && source.includes("aura.kind === 'sweeping_strikes'"), 'area echo aura kinds drifted');
invariant(source.includes('const replayed = Math.max(1, Math.round(amount * multiplier));') && source.includes('if (hostile.id === primary.id || !ctx.hasLineOfSight(source, hostile)) continue;') && source.includes('if (targets >= maxTargets) return;'), 'area echo replay ordering drifted');
invariant(source.includes('const remaining = (aura.charges ?? 1) - 1;') && source.includes('if (remaining <= 0) {') && source.includes('aura.charges = remaining;'), 'area echo charge mutation drifted');
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/area_echo_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  constants,
  qualification: { requires_any: ['weaponStrike', 'directDamage'], excludes_any: ['aoeDamage', 'aoeRoot', 'groundAoE'] },
  aura_kinds: { echo: 'aoe_echo', sweep: 'sweeping_strikes' },
  charge: 'first_echo_aura; missing_charges_defaults_to_one; remove_when_remaining_is_nonpositive',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub echoRadius(): float { return ${constants.AOE_ECHO_RADIUS}.0; }\n` +
  `pub echoMultiplier(): float { return ${constants.AOE_ECHO_MULT}; }\n` +
  `pub echoMaxTargets(): int { return ${constants.AOE_ECHO_MAX_TARGETS}; }\n` +
  `pub sweepMultiplier(): float { return ${constants.SWEEP_MULT}.0; }\n` +
  `pub isSingleTargetEffect(kind: string): bool { return kind == \"weaponStrike\" || kind == \"directDamage\"; }\n` +
  `pub isAreaEffect(kind: string): bool { return kind == \"aoeDamage\" || kind == \"aoeRoot\" || kind == \"groundAoE\"; }\n` +
  `pub isEchoAura(kind: string): bool { return kind == \"aoe_echo\"; }\n` +
  `pub isSweepAura(kind: string): bool { return kind == \"sweeping_strikes\"; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'area echo JSON contract'], [zrOutput, zr, 'area echo Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:area-echo-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:area-echo-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} area echo contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
