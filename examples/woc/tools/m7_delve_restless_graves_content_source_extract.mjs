import { execFileSync } from 'node:child_process';
import ts from 'typescript';
const root=process.env.WOC_GIT_ROOT, commit=process.env.WOC_GIT_COMMIT;
if(!root||!commit) throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
const sourceText=(path)=>execFileSync('git',['-C',root,'show',`${commit}:${path}`],{encoding:'utf8'});
const functionText=(path,name)=>{
  const text=sourceText(path), source=ts.createSourceFile(path,text,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
  const declaration=source.statements.find((s)=>ts.isFunctionDeclaration(s)&&s.name?.text===name);
  if(!declaration) throw new Error(`${name} missing from ${path}`);
  return declaration.getText(source);
};
const hook=functionText('src/sim/combat/damage.ts','handleDeath');
if(!hook) throw new Error('handleDeath missing');
if(!hook.includes("run?.affixes.includes('restless_graves')")||!hook.includes("mobId: 'reliquary_bonewalker'")) throw new Error('restless graves death hook drifted');
const tick=functionText('src/sim/delves/runs.ts','tickDelveRestlessGraves');
for(const marker of ['if (!run.restlessPending.length) return;','if (spawn.at <= ctx.time) ready.push(spawn);','else pending.push(spawn);','run.restlessPending = pending;','for (const spawn of ready)','mob.affixSpawned = true;','run.mobIds.push(mob.id);']) if(!tick.includes(marker)) throw new Error(`restless graves tick drifted: ${marker}`);
process.stdout.write(JSON.stringify({delay_seconds:3,spawn_mob_id:'reliquary_bonewalker',requires_affix:true,excludes_boss:true,excludes_elite:true,excludes_affix_spawned:true,ready_at_or_before_time:true,preserves_source_order:true,spawn_marks_affix_spawned:true}));
