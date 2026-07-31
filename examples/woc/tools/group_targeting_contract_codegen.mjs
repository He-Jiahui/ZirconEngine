import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/group_targeting.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'group_targeting_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'group_targeting_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const source = sourceBlob(SOURCE_PATH);
  const text = source.toString('utf8');
  invariant(text.includes('const memberIds = party ? party.members : [caster.id];'), 'solo group-targeting fallback drifted');
  invariant(text.includes('const meta = ctx.players.get(pid);') && text.includes('if (!e || !meta || e.dead) continue;'), 'player/liveness filter drifted');
  invariant(text.includes('const dx = e.pos.x - cx;') && text.includes('const dz = e.pos.z - cz;') && text.includes('dx * dx + dz * dz > r2'), 'planar radius filter drifted');
  invariant(text.includes('out.sort((a, b) => a.id - b.id);'), 'deterministic entity-id ordering drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/group_targeting_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(source) },
    semantics: {
      solo_fallback: 'caster_only',
      filters: ['existing_entity', 'player_meta', 'living', 'planar_radius'],
      radius_axes: ['x', 'z'],
      order: 'ascending_entity_id',
    },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'group targeting JSON contract');
  writeOrCheck(zrOutput, renderZr(), 'group targeting Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} group targeting contract for ${SOURCE_COMMIT}\n`);
}

function renderZr() {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    'pub casterOnlyWhenSolo(required: bool): bool { return required; }\n' +
    'pub requiresPlayerMeta(required: bool): bool { return required; }\n' +
    'pub excludesDead(required: bool): bool { return required; }\n' +
    'pub planarRadius(required: bool): bool { return required; }\n' +
    'pub ascendingEntityId(required: bool): bool { return required; }\n';
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:group-targeting-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:group-targeting-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
