import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const party = gitShow('src/sim/social/party.ts');
const partyProjectionPath = resolve(
  workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src', 'social',
  'party_raid_state.zr',
);
const partyProjection = readFileSync(partyProjectionPath, 'utf8');
const invite = methodBlock(party, '  partyInvite(targetPid: number');
const accept = methodBlock(party, '  partyAccept(pid?: number)');
const decline = methodBlock(party, '  partyDecline(pid?: number)');
const convert = methodBlock(party, '  convertPartyToRaid(pid?: number)');
const unraid = methodBlock(party, '  convertRaidToParty(pid?: number)');
const move = methodBlock(party, '  moveRaidMember(targetPid: number');
const kick = methodBlock(party, '  partyKick(targetPid: number');
const promote = methodBlock(party, '  partyPromote(targetPid: number');
const remove = methodBlock(party, '  removeFromParty(pid: number');
const activeInvite = methodBlock(party, '  private hasActiveInvite(');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src');

for (const needle of [
  '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a',
  'pub convertToParty(',
  'pub decline(',
  'pub kick(',
  'pub promote(',
  'while (player < 11)',
]) {
  invariant(partyProjection.includes(needle), `party projection omitted current-head behavior: ${needle}`);
}

for (const needle of [
  'const PARTY_MAX = 5;',
  'const RAID_MIN = 5;',
  'const RAID_MAX = 10;',
  'const RAID_GROUP_MAX = 5;',
]) {
  invariant(party.includes(needle), `missing pinned party constant: ${needle}`);
}
for (const needle of [
  'if (invite.expires < this.ctx.time)',
  'map.delete(targetPid);',
  'this.hasActiveInvite(this.partyInvites, targetPid) ||',
  'this.hasActiveInvite(this.ctx.tradeInvites, targetPid) ||',
  'this.hasActiveInvite(this.ctx.duelInvites, targetPid)',
]) {
  invariant(party.includes(needle), `missing pinned social-invite behavior: ${needle}`);
}
for (const needle of [
  'if (targetPid === r.meta.entityId) return;',
  'myParty && myParty.leader !== r.meta.entityId',
  'myParty && myParty.members.length >= this.partyCapacity(myParty)',
  'if (this.partyOf(targetPid))',
  'if (this.hasPendingSocialInvite(targetPid))',
  'this.partyInvites.set(targetPid, { fromPid: r.meta.entityId, expires: this.ctx.time + 30 });',
]) {
  invariant(invite.includes(needle), `missing pinned party-invite behavior: ${needle}`);
}
const expiredAccept = accept.indexOf('if (!invite || invite.expires < this.ctx.time)');
const deleteInvite = accept.indexOf('this.partyInvites.delete(r.meta.entityId);');
invariant(expiredAccept >= 0, 'missing expired party-accept rejection');
invariant(deleteInvite > expiredAccept, 'party accept must reject an expired invite before deletion');
for (const needle of [
  'id: this.nextPartyId++,',
  'members: [invite.fromPid],',
  'raid: false,',
  'this.partyByPid.set(invite.fromPid, party.id);',
  'party.members.push(r.meta.entityId);',
  'party.raidGroups.set(r.meta.entityId, raidGroup);',
]) {
    invariant(accept.includes(needle), `missing pinned party-accept behavior: ${needle}`);
}
for (const needle of [
  'this.partyInvites.delete(r.meta.entityId);',
  "`${r.meta.name} declines your invitation.`",
]) {
  invariant(decline.includes(needle), `missing pinned party-decline behavior: ${needle}`);
}
for (const needle of [
  'if (party.members.length < RAID_MIN)',
  'party.raid = true;',
  'this.normalizeRaidGroups(party);',
]) {
    invariant(convert.includes(needle), `missing pinned raid-convert behavior: ${needle}`);
}
for (const needle of [
  'if (!party.raid)',
  'if (party.members.length > PARTY_MAX)',
  'party.raid = false;',
  'party.raidGroups.clear();',
]) {
  invariant(unraid.includes(needle), `missing pinned raid-fold behavior: ${needle}`);
}
for (const needle of [
  "if (!party?.raid)",
  'if (party.leader !== r.meta.entityId)',
  'if (inTargetGroup >= RAID_GROUP_MAX)',
  'party.raidGroups.set(targetPid, group);',
]) {
    invariant(move.includes(needle), `missing pinned raid-group behavior: ${needle}`);
}
for (const needle of [
  'if (!party || party.leader !== r.meta.entityId)',
  'if (!party.members.includes(targetPid) || targetPid === r.meta.entityId) return;',
  "this.removeFromParty(targetPid, 'has been removed from the party');",
]) {
  invariant(kick.includes(needle), `missing pinned party-kick behavior: ${needle}`);
}
for (const needle of [
  'if (!party.members.includes(targetPid) || targetPid === party.leader) return;',
  'party.leader = targetPid;',
  'this.announceLooterShift(party, beforeLooter);',
]) {
  invariant(promote.includes(needle), `missing pinned party-promote behavior: ${needle}`);
}
for (const needle of [
  'party.members = party.members.filter((m) => m !== pid);',
  'party.raidGroups.delete(pid);',
  'if (party.members.length <= 1)',
  'this.parties.delete(party.id);',
  'this.ctx.dropPartyMarkers(party.id);',
  'party.leader = party.members[0];',
  'if (party.raid) this.normalizeRaidGroups(party);',
]) {
  invariant(remove.includes(needle), `missing pinned party-leave behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("social/party_raid_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['social/m6_scenario_matrix.zr', 'social/party_raid_state_test_main.zr']),
  `party_raid_state escaped the M6 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M6 party/raid state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function methodBlock(source, declaration) {
  const start = source.indexOf(declaration);
  invariant(start >= 0, `missing source method: ${declaration}`);
  const open = source.indexOf('{', start);
  invariant(open >= 0, `missing source method body: ${declaration}`);
  let depth = 0;
  let quote = '';
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let index = open; index < source.length; index += 1) {
    const character = source[index];
    const next = source[index + 1] ?? '';
    if (lineComment) {
      if (character === '\n') lineComment = false;
      continue;
    }
    if (blockComment) {
      if (character === '*' && next === '/') {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === quote) quote = '';
      continue;
    }
    if (character === '/' && next === '/') {
      lineComment = true;
      index += 1;
      continue;
    }
    if (character === '/' && next === '*') {
      blockComment = true;
      index += 1;
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
  throw new Error(`unterminated source method: ${declaration}`);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
