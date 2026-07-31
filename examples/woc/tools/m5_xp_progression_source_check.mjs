import { execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const sourceRoot = resolve(scriptDirectory, '..', '..', '..', 'dev', 'world-of-claudecraft');
const types = gitShow('src/sim/types.ts');
const damage = gitShow('src/sim/combat/damage.ts');
const progression = gitShow('src/sim/progression/xp.ts');
const compactTypes = types.replace(/\s+/g, '');
const compactDamage = damage.replace(/\s+/g, '');
const compactProgression = progression.replace(/\s+/g, '');

for (const needle of [
  'export const MAX_LEVEL = 20;',
  'const POSTCAP_GROWTH = 1.1;',
  'export const MAX_VIRTUAL_LEVEL = 200;',
  'export const PRESTIGE_XP_PER_RANK = xpForLevel(MAX_LEVEL);',
]) {
  invariant(types.includes(needle), `missing pinned XP constant: ${needle}`);
}

for (const needle of [
  'constcum:number[]=[0,0];',
  'for(letlvl=1;lvl<MAX_LEVEL;lvl++){total+=XP_TABLE[lvl-1];cum[lvl+1]=total;}',
  'letstep=XP_TABLE[MAX_LEVEL-1];',
  'for(letlvl=MAX_LEVEL;lvl<MAX_VIRTUAL_LEVEL;lvl++){total+=Math.round(step);cum[lvl+1]=total;step*=POSTCAP_GROWTH;}',
  'returnVLEVEL_CUM[Math.max(1,Math.min(MAX_VIRTUAL_LEVEL,Math.floor(level)))];',
  'if(VLEVEL_CUM[mid]<=xp)lo=mid;elsehi=mid-1;',
  'constfloor=VLEVEL_CUM[level];',
  'constnext=VLEVEL_CUM[Math.min(level+1,MAX_VIRTUAL_LEVEL)];',
  'return{level,into:Math.max(0,Math.min(span,lifetimeXp-floor)),span};',
  'returnearned<=0?0:Math.floor(earned/PRESTIGE_XP_PER_RANK);',
  'returnlevel>=MAX_LEVEL&&prestigeRank<maxPrestigeRank(lifetimeXp);',
  'returnMath.max(0,target-lifetimeXp);',
]) {
  invariant(compactTypes.includes(needle), `missing pinned XP formula: ${needle}`);
}

for (const needle of [
  'if(meta.lifetimeXp>=Number.MAX_SAFE_INTEGER){',
  'meta.lifetimeXp=Number.MAX_SAFE_INTEGER;',
  'accrueLifetimeXp(ctx,amount,meta,p);meta.counters.xpGained+=amount;',
  'for(letv=beforeVL+1;v<=afterVL;v++){ctx.emit({type:\'virtualLevelUp\',level:v,pid:p.id});}',
]) {
  invariant(compactDamage.includes(needle), `missing pinned XP award behavior: ${needle}`);
}
invariant(
  compactProgression.includes('if(!canPrestige(r.e.level,r.meta.lifetimeXp,r.meta.prestigeRank))returnfalse;'),
  'missing pinned prestige eligibility gate',
);

process.stdout.write(`checked M5 XP progression source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
