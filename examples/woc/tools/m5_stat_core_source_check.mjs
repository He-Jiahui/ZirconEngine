import { execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const sourceRoot = resolve(scriptDirectory, '..', '..', '..', 'dev', 'world-of-claudecraft');
const entity = gitShow('src/sim/entity.ts');
const types = gitShow('src/sim/types.ts');
const compact = entity.replace(/\s+/g, '');

for (const needle of [
  'export const SPELL_POWER_PER_INT = 0.5;',
]) {
  invariant(types.includes(needle), `missing pinned stat core constant: ${needle}`);
}
for (const needle of [
  'consts=Math.max(0,sta);',
  'returnMath.min(s,20)+Math.max(0,s-20)*10;',
  'consti=Math.max(0,int);',
  'returnMath.min(i,20)+Math.max(0,i-20)*15;',
  'def.baseStats.str+def.statsPerLevel.str*(lvl-1)',
  "cls==='warrior'||cls==='paladin'||cls==='shaman'||cls==='druid'",
  "cls==='rogue'||cls==='hunter'",
  'Math.max(0,Math.round((apFromStats+bonusAp)*(1+(mods?.stats.apPct??0)+buffApPct)),)',
  'Math.max(0,Math.round((s.agi*2+bonusAp)*(1+(mods?.stats.apPct??0)+buffApPct)))',
  'Math.max(0,Math.round(s.int*SPELL_POWER_PER_INT+bonusSp))',
  '0.05+s.agi*0.0005',
  'Math.max(0,0.05+s.agi*0.0005+bonusDodge)',
  'def.baseHp+def.hpPerLevel*(lvl-1)+hpFromStamina(s.sta)',
  'if(bearForm)e.maxHp=Math.round(e.maxHp*1.15);',
  'if(mods?.stats.maxHpPct)e.maxHp=Math.round(e.maxHp*(1+mods.stats.maxHpPct));',
  'if(scaleMul>1)e.maxHp=Math.round(e.maxHp*scaleMul);',
  'def.baseMana+def.manaPerLevel*(lvl-1)+manaFromIntellect(s.int)',
]) {
  invariant(compact.includes(needle), `missing pinned stat core formula: ${needle}`);
}

process.stdout.write(`checked M5 stat core source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
