import { execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/pet/pet_ai.ts');
const normalized = source.replace(/\s+/g, ' ');

for (const needle of [
  'const PET_ASSIST_RANGE = 50;',
  'const PET_AGGRESSIVE_RANGE = 18;',
  'const PET_OWNER_IDLE_TICKS = 1200;',
]) {
  invariant(source.includes(needle), `missing pinned pet target constant: ${needle}`);
}
for (const needle of [
  "if (pet.petMode === 'passive') return null;",
  'ctx.tickCount - ownerMeta.lastActiveTick > PET_OWNER_IDLE_TICKS',
  'pet.petMode === \'aggressive\' ? PET_AGGRESSIVE_RANGE : PET_ASSIST_RANGE',
  'ctx.grid.forEachInRadius(pet.pos.x, pet.pos.z, PET_ASSIST_RANGE, (m) => {',
  'if (m.id === pet.id || m.dead || !ctx.isHostileTo(pet, m)) return;',
  "m.kind === 'mob' && (m.aggroTargetId === owner.id || m.aggroTargetId === pet.id)",
  "owner.targetId === m.id && (owner.autoAttack || (m.kind === 'mob' && m.threat.has(owner.id)))",
  "pet.petMode === 'aggressive' && !ownerIdle && dist2d(pet.pos, m.pos) <= PET_AGGRESSIVE_RANGE",
  'if (!engagingUs && !ownerOffense && !aggressive) return;',
  'if (d < bestD) {',
]) {
  invariant(normalized.includes(needle), `missing pinned pet target branch: ${needle}`);
}

process.stdout.write(`checked M7 pet target source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
