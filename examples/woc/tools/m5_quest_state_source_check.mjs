import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const sourceRoot = resolve('..', '..', '..', 'dev', 'world-of-claudecraft');
const zone = gitShow('src/sim/content/zone1.ts');
const commands = gitShow('src/sim/quests/quest_commands.ts');
const credit = gitShow('src/sim/quests/quest_credit.ts');
const fallback = gitShow('src/sim/quest_fallback.ts');
const damage = gitShow('src/sim/combat/damage.ts');
const compactCommands = commands.replace(/\s+/g, '');
const compactCredit = credit.replace(/\s+/g, '');
const compactDamage = damage.replace(/\s+/g, '');
const wolves = questBlock(zone, 'q_wolves');
const boars = questBlock(zone, 'q_boars');
const wocSourceRoot = resolve('..', 'scripts', 'woc_game', 'src');

for (const needle of [
  "id: 'q_wolves',",
  "{ type: 'kill', targetMobId: 'forest_wolf', count: 8, label: 'Forest Wolf slain' }",
  'xpReward: 250,',
  'copperReward: 75,',
  'itemRewards: {},',
]) {
  invariant(wolves.includes(needle), `missing pinned q_wolves fact: ${needle}`);
}
for (const needle of [
  "id: 'q_boars',",
  "{ type: 'collect', itemId: 'boar_hide', count: 5, label: 'Bristly Boar Hide' }",
  'xpReward: 350,',
  'copperReward: 120,',
  'itemRewards: {},',
]) {
  invariant(boars.includes(needle), `missing pinned q_boars fact: ${needle}`);
}
for (const [questId, block] of [
  ['q_wolves', wolves],
  ['q_boars', boars],
]) {
  for (const unsupportedField of [
    'requiresQuest:',
    'minLevel:',
    'retired:',
    'shareable:',
    'requiredItem:',
  ]) {
    invariant(
      !block.includes(unsupportedField),
      `selected ${questId} gained unsupported quest field: ${unsupportedField}`,
    );
  }
}

for (const needle of [
  "if(p.dead){ctx.error(meta.entityId,\"Youcan'tdothatwhiledead.\");return;}",
  "if(questState(ctx,questId,meta.entityId)!=='available')",
  'finalizeQuestAccept(ctx,questId,quest,meta,selection);',
  "if(!myParty||!sharerParty||myParty.id!==sharerParty.id)",
  'meta.questLog.delete(questId);',
  "if(qp.state!=='ready')",
  'for(const[index,obj]ofquest.objectives.entries()){if(obj.type===\'collect\'&&obj.itemId){ctx.removeItem(obj.itemId,questObjectiveRequired(quest,qp,index),meta.entityId);}}',
  "qp.state='done';meta.questLog.delete(questId);constfirstCompletion=!meta.questsDone.has(questId);meta.questsDone.add(questId);if(firstCompletion)meta.counters.questsCompleted++;",
  'ctx.grantXp(quest.xpReward,meta);',
]) {
  invariant(compactCommands.includes(needle), `missing pinned quest-command behavior: ${needle}`);
}

for (const needle of [
  'functioncreditDiscreteQuestObjectives(',
  'questObjectiveRequired(quest,qp,objectiveIndex);',
  "objective.type==='kill'&&objective.targetMobId===mob.templateId",
  "objective.type==='craft'&&objective.recipeId===recipeId",
  "if(objective.type!=='gather')returnfalse;",
  'consthave=Math.min(required,ctx.countItem(obj.itemId,meta.entityId));',
  'objectiveIndex:i,',
  'current:have,',
  'required,',
  'if(have>qp.counts[i])meta.counters.questProgress+=have-qp.counts[i];',
  'if(ready&&qp.state===\'active\')',
  "}elseif(!ready&&qp.state==='ready'){",
]) {
  invariant(compactCredit.includes(needle), `missing pinned quest-credit behavior: ${needle}`);
}

for (const needle of [
  'export function questFallbackGrants(',
  'const required = quest.requiredItems;',
  'if(!hasItem(itemId)&&!grants.includes(itemId))grants.push(itemId);',
]) {
  invariant(fallback.replace(/\s+/g, '').includes(needle.replace(/\s+/g, '')),
    `missing pinned quest fallback behavior: ${needle}`);
}

for (const needle of [
  'if(meta.lifetimeXp>=Number.MAX_SAFE_INTEGER){',
  'meta.lifetimeXp=Number.MAX_SAFE_INTEGER;',
  'accrueLifetimeXp(ctx,amount,meta,p);meta.counters.xpGained+=amount;',
]) {
  invariant(compactDamage.includes(needle), `missing pinned quest reward XP behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("progression/quest_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['progression/m5_scenario_matrix.zr', 'progression/quest_state_test_main.zr']),
  `quest_state escaped the M5 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M5 quest state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function questBlock(source, questId) {
  const start = source.indexOf(`  ${questId}: {`);
  invariant(start >= 0, `missing selected quest block: ${questId}`);
  let depth = 0;
  let quote = '';
  let escaped = false;
  for (let index = source.indexOf('{', start); index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === quote) quote = '';
      continue;
    }
    if (character === "'" || character === '"' || character === '`') {
      quote = character;
      continue;
    }
    if (character === '{') depth += 1;
    if (character === '}') {
      depth -= 1;
      if (depth === 0) return source.slice(start, index + 1);
    }
  }
  throw new Error(`unterminated selected quest block: ${questId}`);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
