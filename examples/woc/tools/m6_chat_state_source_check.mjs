import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';

const SOURCE_COMMIT = '7c10f280eec380e9877e66ce16333089e171fe42';
const sourceRoot = resolve('..', '..', '..', 'dev', 'world-of-claudecraft');
const chat = gitShow('src/sim/social/chat.ts');
const router = functionBlock(chat, 'export function chat');
const throttle = functionBlock(chat, 'export function chatAllowed');
const wocSourceRoot = resolve('..', 'scripts', 'woc_game', 'src');

for (const needle of [
  'const CHAT_BURST = 8;',
  'const CHAT_REFILL = 2;',
  'const OVERHEAD_EMOTE_DURATION = 3.2;',
  'export function helpLines(): string[]',
]) {
  invariant(chat.includes(needle), `missing pinned chat fact: ${needle}`);
}

for (const needle of [
  'const raw = text.trim().slice(0, MAX_CHAT_MESSAGE_LEN);',
  'if (!raw) return null;',
  'if (!chatAllowed(ctx, r.meta.entityId))',
  "ctx.error(r.meta.entityId, 'You are sending messages too quickly.');",
  "const jm = /^\\/(join|leave)\\b\\s*(\\S*)\\s*$/i.exec(raw);",
  'handleChannelMembership(',
  'const cm = /^\\/(world|lfg)\\s+([\\s\\S]+)$/i.exec(raw);',
  'if (!mine?.has(channel))',
  'for (const [subPid, set] of ctx.channelSubs)',
  "let channel: 'say' | 'yell' = 'say';",
  'const range = channel === \'yell\' ? YELL_RANGE : SAY_RANGE;',
  'for (const meta of ctx.players.values())',
  'if (!e || dist2d(r.e.pos, e.pos) > range) continue;',
]) {
  invariant(router.includes(needle), `missing pinned chat-router behavior: ${needle}`);
}

for (const needle of [
  'b = { tokens: CHAT_BURST, at: ctx.time };',
  'Math.min(CHAT_BURST, b.tokens + (ctx.time - b.at) * CHAT_REFILL)',
  'b.at = ctx.time;',
  'if (b.tokens < 1) return false;',
  'b.tokens -= 1;',
]) {
  invariant(throttle.includes(needle), `missing pinned chat-throttle behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("social/chat_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['social/chat_state_test_main.zr', 'social/m6_scenario_matrix.zr']),
  `chat_state escaped the M6 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M6 chat state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

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
