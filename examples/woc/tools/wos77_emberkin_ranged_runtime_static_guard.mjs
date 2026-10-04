// 调用入口：在 examples/woc/tools 目录直接执行 node wos77_emberkin_ranged_runtime_static_guard.mjs；缺少源码契约时脚本抛错退出。
// 检查锁定的 Emberkin 射程、伤害与随机数规则及 Zr 纯规则，再核对世界投射物、固定帧顺序、自测和合同文本。
// 这些断言只核对源码文本与元数据结构；通过并不证明运行时行为等价。

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (...parts) => readFileSync(resolve(root, ...parts), 'utf8');
const petSource = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_ai.ts');
const petContent = read('..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'content', 'warlock_pets.ts');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
const rules = read('scripts', 'woc_game', 'src', 'instances', 'emberkin_ranged_attack_rules.zr');
const contract = read('contracts', 'world-state.md');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(label + ': missing ' + JSON.stringify(expected));
}

for (const expected of [
  'dmgBase: 5,',
  'dmgPerLevel: 1.1,',
  'attackSpeed: 2.0,',
  "petRanged: { range: 25, school: 'fire' },",
]) requireText(petContent, expected, 'Emberkin source content');

for (const expected of [
  'const PET_LEASH = 40;',
  'const crit = ctx.rng.chance(0.05);',
  'ctx.rng.range(src.weapon.min, src.weapon.max)',
  '(ctx.effectiveAttackPower(src) / 14) * src.weapon.speed',
  "ctx.dealDamage(src, tgt, Math.max(1, Math.round(dmg)), crit, ranged.school, null, 'hit');",
]) requireText(petSource, expected, 'Emberkin source pet AI');

for (const expected of [
  'pub fireboltRange(): float',
  'return 25.0;',
  'pub leashDistance(): float',
  'return 40.0;',
  'pub fireboltCritChance(): float',
  'return 0.05;',
  'pub shouldKeepFacing(distanceSquared: float): bool',
  'return distanceSquared < 0.01;',
  'pub resolveFireboltDamage(',
  '(attackPower / 14.0) * weaponSpeed',
  'pub contractTest(): int',
]) requireText(rules, expected, 'WOS77 pure rules');

for (const expected of [
  'var emberkinRanged = %import("instances/emberkin_ranged_attack_rules");',
  'offlineEmberkinFireboltProjectileProfileIsValid(',
  'landOfflineEmberkinFireboltProjectile(',
  'var crit = nextAuthoritativeRandomUnit(state) < emberkinRanged.fireboltCritChance();',
  'if (!emberkinRanged.shouldKeepFacing(targetDistanceSquared)) {',
  'nextAuthoritativeRandomUnit(state),',
  'stepOfflineEmberkinRangedAttack(state);',
  'pub emberkinRangedProjectileStateTest(): int',
]) requireText(world, expected, 'WOS77 world reducer');

// 比较源码位置，要求同一帧中先推进投射物，再更新宠物，最后处理怪物 AI。
const projectilePhase = world.indexOf('stepOfflineEastbrookProjectiles(state);');
const petPhase = world.indexOf('stepOfflineEmberkinRangedAttack(state);', projectilePhase);
const mobPhase = world.indexOf('stepOfflineEastbrookMobIdleAggro(state);');
if (projectilePhase < 0 || petPhase < projectilePhase || mobPhase < petPhase) {
  throw new Error('WOS77 fixed-tick order drifted: projectiles, pet update, then mob AI');
}

requireText(contract, 'WOS77 extends that same schema-62 entity/queue projection', 'WOS77 contract');
requireText(contract, 'strict 5% crit then', 'WOS77 contract');
requireText(contract, 'weapon-range order', 'WOS77 contract');

process.stdout.write('WOS77 Emberkin Firebolt runtime static guard passed\n');
