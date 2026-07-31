import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const source = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'sim.ts');
const petSource = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_ai.ts');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  'if (d > spell.range) {',
  'if (!isRooted(pet)) this.moveToward(pet, target.pos, pet.moveSpeed * this.moveSpeedMult(pet));',
  'pet.swingTimer = Math.max(0, pet.swingTimer - DT);',
]) requireText(source, expected, 'source ranged-pet chase');

for (const expected of [
  'private moveToward(e: Entity, dest: Vec3, speed: number, ignoreObstacles = false): boolean {',
  'const step = Math.min(speed * DT, d);',
  'const nx = e.pos.x + Math.sin(desired) * step;',
]) requireText(source, expected, 'source open-ground move');

requireText(petSource, 'function petFollow(ctx: SimContext, pet: Entity, owner: Entity): void {', 'source pet dispatcher');

for (const expected of [
  'stepOfflineEmberkinOpenGroundMoveToward(',
  'stepOfflineEmberkinStraightChase(',
  '<float>state.entityMoveSpeed[petIndex]',
  'stepOfflineEmberkinStraightChase(state, petIndex, targetIndex);',
  'pub emberkinChaseStateTest(): int',
]) requireText(world, expected, 'WOS81 reducer');

const chase = world.indexOf('stepOfflineEmberkinStraightChase(state, petIndex, targetIndex);');
const cooldown = world.indexOf('emberkinRanged.cooldownAfterNoTarget(', chase);
if (chase < 0 || cooldown < chase) throw new Error('WOS81 movement must precede no-fire cooldown decay');

requireText(contract, 'WOS81 retains Emberkin\'s source ranged-pet out-of-range arm', 'WOS81 contract');
requireText(contract, 'root checks, move-speed modifiers', 'WOS81 contract');
requireText(contract, 'equivalent behavior.', 'WOS81 contract');

process.stdout.write('WOS81 Emberkin chase runtime static guard passed\n');
