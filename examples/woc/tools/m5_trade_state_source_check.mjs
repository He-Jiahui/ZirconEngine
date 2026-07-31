import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const sourceRoot = resolve(scriptDirectory, '..', '..', '..', 'dev', 'world-of-claudecraft');
const trade = gitShow('src/sim/social/trade.ts');
const request = functionBlock(trade, 'export function tradeRequest');
const accept = functionBlock(trade, 'export function tradeAccept');
const setOffer = functionBlock(trade, 'export function tradeSetOffer');
const confirm = functionBlock(trade, 'export function tradeConfirm');
const update = functionBlock(trade, 'export function updateTradesAndInvites');
const wocSourceRoot = resolve(scriptDirectory, '..', 'scripts', 'woc_game', 'src');

invariant(trade.includes('const TRADE_RANGE = 10;'), 'missing pinned trade range');
for (const needle of [
  'if (targetPid === r.meta.entityId) return;',
  'ctx.trades.has(r.meta.entityId) || ctx.trades.has(targetPid)',
  'dist2d(r.e.pos, targetE.pos) > TRADE_RANGE',
  'ctx.hasPendingSocialInvite(targetPid)',
  'ctx.tradeInvites.set(targetPid, { fromPid: r.meta.entityId, expires: ctx.time + 30 });',
]) {
  invariant(request.includes(needle), `missing pinned trade-request behavior: ${needle}`);
}
invariant(!request.includes('.dead'), 'trade request unexpectedly gained a dead-actor gate');

const expiredAccept = accept.indexOf('if (!invite || invite.expires < ctx.time)');
const deleteInvite = accept.indexOf('ctx.tradeInvites.delete(r.meta.entityId);');
invariant(expiredAccept >= 0, 'missing expired trade-accept rejection');
invariant(deleteInvite > expiredAccept, 'trade accept must reject an expired invite before deletion');
for (const needle of [
  'offerA: { items: [], copper: 0 },',
  'offerB: { items: [], copper: 0 },',
  'acceptedA: false,',
  'acceptedB: false,',
  'ctx.trades.set(session.a, session);',
  'ctx.trades.set(session.b, session);',
]) {
  invariant(accept.includes(needle), `missing pinned trade-accept behavior: ${needle}`);
}

for (const needle of [
  'for (const slot of items.slice(0, 6))',
  "typeof slot.itemId !== 'string' || !Number.isFinite(slot.count)",
  'const count = Math.max(1, Math.floor(slot.count));',
  "def.kind === 'quest' || def.soulbound",
  'merged.set(slot.itemId, (merged.get(slot.itemId) ?? 0) + count);',
  'if (ctx.countItem(itemId, r.meta.entityId) < count) continue;',
  'copper: Math.max(0, Math.min(Math.floor(copper), r.meta.copper)),',
  'session.acceptedA = false;',
  'session.acceptedB = false;',
]) {
  invariant(setOffer.includes(needle), `missing pinned trade-offer behavior: ${needle}`);
}

for (const needle of [
  'if (!(session.acceptedA && session.acceptedB)) return;',
  'session.offerA.copper <= metaA.copper',
  'offerCovered(ctx, session.offerA.items, session.a)',
  'const instancedCount = Math.max(0, s.count - ctx.countFungibleItem(s.itemId, giverPid));',
  'if (countFit(scratch, capacity, s.itemId, plainCount) < plainCount) return false;',
  '!fitsAfterSwap(metaA, session.b, session.offerA.items, session.offerB.items)',
  '!fitsAfterSwap(metaB, session.a, session.offerB.items, session.offerA.items)',
  'metaA.copper = metaA.copper - session.offerA.copper + session.offerB.copper;',
  'metaB.copper = metaB.copper - session.offerB.copper + session.offerA.copper;',
  'transferOffer(ctx, session.offerA.items, session.a, session.b);',
  'transferOffer(ctx, session.offerB.items, session.b, session.a);',
  'const nonEmpty =',
  "ctx.bumpDeedStat(metaA, 'tradesCompleted', 1);",
  'closeTrade(ctx, session);',
]) {
  invariant(confirm.includes(needle), `missing pinned trade-confirm behavior: ${needle}`);
}

for (const needle of [
  'function transferOffer(ctx: SimContext, items: InvSlot[], fromPid: number, toPid: number): void {',
  'const instances = removePreferFungible(ctx, s.itemId, s.count, fromPid);',
  'const plainCount = s.count - instances.length;',
  'for (const instance of instances) ctx.addItemInstance(s.itemId, instance, toPid);',
]) {
  invariant(trade.includes(needle), `missing pinned instance-transfer behavior: ${needle}`);
}

for (const needle of [
  'for (const map of [ctx.partyInvites, ctx.tradeInvites, ctx.duelInvites])',
  'if (invite.expires < ctx.time) map.delete(pid);',
  'dist2d(ea.pos, eb.pos) > TRADE_RANGE + 4 || ea.dead || eb.dead',
  'tradeCancel(ctx, session.a);',
]) {
  invariant(update.includes(needle), `missing pinned trade-update behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("progression/trade_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['progression/m5_scenario_matrix.zr', 'progression/trade_state_test_main.zr']),
  `trade_state escaped the M5 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M5 trade state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function functionBlock(source, declaration) {
  const start = source.indexOf(declaration);
  invariant(start >= 0, `missing source function: ${declaration}`);
  const open = source.indexOf('{', start);
  invariant(open >= 0, `missing source function body: ${declaration}`);
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
  throw new Error(`unterminated source function: ${declaration}`);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
