import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';

const SOURCE_COMMIT = '7c10f280eec380e9877e66ce16333089e171fe42';
const sourceRoot = resolve('..', '..', '..', 'dev', 'world-of-claudecraft');
const duel = gitShow('src/sim/social/duel.ts');
const party = gitShow('src/sim/social/party.ts');
const compactDuel = duel.replace(/\s+/g, '');
const compactParty = party.replace(/\s+/g, '');
const request = functionBlock(duel, 'duelRequest');
const wocSourceRoot = resolve('..', 'scripts', 'woc_game', 'src');

for (const needle of [
  'const DUEL_COUNTDOWN = 3;',
  'const DUEL_FORFEIT_DISTANCE = 60;',
]) {
  invariant(duel.includes(needle), `missing pinned duel constant: ${needle}`);
}

for (const needle of [
  'if(targetPid===r.meta.entityId)return;',
  "if(ctx.duels.has(r.meta.entityId)||ctx.duels.has(targetPid))",
  'if(dist2d(r.e.pos,targetE.pos)>30)',
  'if(ctx.hasPendingSocialInvite(targetPid))',
  'ctx.duelInvites.set(targetPid,{fromPid:r.meta.entityId,expires:ctx.time+30});',
]) {
  invariant(compactDuel.includes(needle), `missing pinned duel-request behavior: ${needle}`);
}
invariant(!request.includes('.dead'), 'pinned duel request unexpectedly checks dead state');

for (const needle of [
  "if(!invite||invite.expires<ctx.time){ctx.error(r.meta.entityId,'Thechallengehasexpired.');return;}",
  'ctx.duelInvites.delete(r.meta.entityId);constother=ctx.players.get(invite.fromPid);',
  "state:'countdown',timer:DUEL_COUNTDOWN,",
  'for(constdPidof[duel.a,duel.b]){ctx.emit({type:\'duelCountdown\',seconds:DUEL_COUNTDOWN,pid:dPid});}',
  'constinvite=ctx.duelInvites.get(r.meta.entityId);ctx.duelInvites.delete(r.meta.entityId);',
  'if(dist2d(ea.pos,eb.pos)>DUEL_FORFEIT_DISTANCE){endDuel(ctx,duel,null);}',
  "ctx.bumpDeedStat(winner,'duelsWon',1);ctx.bumpDeedStat(loser,'duelsLost',1);",
]) {
  invariant(compactDuel.includes(needle), `missing pinned duel lifecycle behavior: ${needle}`);
}

for (const needle of [
  'if(invite.expires<this.ctx.time){map.delete(targetPid);returnfalse;}',
  'this.hasActiveInvite(this.partyInvites,targetPid)||',
  'this.hasActiveInvite(this.ctx.tradeInvites,targetPid)||',
  'this.hasActiveInvite(this.ctx.duelInvites,targetPid)',
]) {
  invariant(compactParty.includes(needle), `missing pinned shared-invite behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("social/duel_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['social/duel_state_test_main.zr', 'social/m6_scenario_matrix.zr']),
  `duel_state escaped the M6 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M6 duel state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function functionBlock(source, name) {
  const start = source.indexOf(`export function ${name}(`);
  invariant(start >= 0, `missing source function: ${name}`);
  return braceBlock(source, source.indexOf('{', start), name);
}

function braceBlock(source, start, label) {
  let depth = 0;
  let quote = '';
  let escaped = false;
  for (let index = start; index < source.length; index += 1) {
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
  throw new Error(`unterminated source block: ${label}`);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
