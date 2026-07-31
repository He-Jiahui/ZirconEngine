import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/mass_resurrection.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'mass_resurrection_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'mass_resurrection_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  invariant(text.includes('if (!party) return false;'), 'mass resurrection solo gate drifted');
  invariant(text.includes("member?.kind === 'player' && (member.dead || member.ghost)"), 'mass resurrection eligibility drifted');
  invariant(text.includes('for (const memberId of party.members)') && text.includes('offerResurrection(ctx, caster, member, hpFrac);'), 'mass resurrection roster-order offer route drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/mass_resurrection_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    selection: { solo: 'no_targets', eligible: ['player', 'dead_or_ghost'], order: 'party_roster' },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'mass resurrection JSON contract');
  writeOrCheck(zrOutput, renderZr(), 'mass resurrection Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} mass resurrection contract for ${SOURCE_COMMIT}\n`);
}

function renderZr() {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    'pub noTargetsWhenSolo(required: bool): bool { return required; }\n' +
    'pub includesDeadOrGhostPlayers(required: bool): bool { return required; }\n' +
    'pub preservesPartyRosterOrder(required: bool): bool { return required; }\n';
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:mass-resurrection-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:mass-resurrection-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
