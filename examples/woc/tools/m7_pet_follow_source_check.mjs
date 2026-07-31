import { execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', 'dev', 'world-of-claudecraft');

const petAi = gitShow('src/sim/pet/pet_ai.ts');
const types = gitShow('src/sim/types.ts');
const normalizedPetAi = petAi.replace(/\s+/g, ' ');

for (const needle of [
  'const PET_FOLLOW_DISTANCE = 3.5;',
  'const PET_PATH_RECALC = 0.5;',
  'const PET_PATH_SPAN = 96;',
  'const PET_FORCE_RECOVERY_DISTANCE = 96;',
  'const PET_PATH_STALE_DISTANCE = 4;',
  'const PET_WAYPOINT_REACHED = 1;',
]) {
  invariant(petAi.includes(needle), `missing pinned pet follow constant: ${needle}`);
}
for (const needle of [
  'export const RUN_SPEED = 7;',
  'export const PET_TELEPORT_DISTANCE = 60;',
]) {
  invariant(types.includes(needle), `missing pinned shared movement constant: ${needle}`);
}
for (const needle of [
  'if (d <= PET_FOLLOW_DISTANCE) { pet.petPath = []; return; }',
  'if (ctx.isRooted(pet)) return;',
  'if (pet.petPathCooldown <= 0 && stale) recompute();',
  'pet.petPath.length > 1 && dist2d(pet.pos, pet.petPath[0]) < PET_WAYPOINT_REACHED',
  'pet.petPath.length <= 1 && d > PET_TELEPORT_DISTANCE',
  'd > PET_FORCE_RECOVERY_DISTANCE || !lineOfSightClear(ctx.cfg.seed, pet.pos, owner.pos, BODY_RADIUS)',
  'recompute(); if (pet.petPath.length <= 1) {',
  'ctx.rebucket(pet);',
  'Math.max(pet.moveSpeed, RUN_SPEED * 1.1) * ctx.moveSpeedMult(pet)',
]) {
  invariant(normalizedPetAi.includes(needle), `missing pinned pet follow branch: ${needle}`);
}

process.stdout.write(`checked M7 pet follow source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
