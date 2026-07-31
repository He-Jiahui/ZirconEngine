import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const casting = gitShow('src/sim/combat/casting_lifecycle.ts');
const castBySlot = methodBlock(casting, 'export function castAbilityBySlot(');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const state = readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const payloads = JSON.parse(readFileSync(resolve(wocRoot, 'contracts', 'command_payloads.json'), 'utf8'));

for (const needle of [
  'const r = ctx.resolve(pid);',
  'const known = r.meta.known[slot];',
  'if (known) castAbility(ctx, known.def.id, pid, aim);',
]) {
  invariant(castBySlot.includes(needle), 'missing pinned cast-slot behavior: ' + needle);
}

const castSlot = payloads.entries.find((entry) => entry.id === 0);
invariant(
  castSlot?.name === 'castSlot' &&
    castSlot.kind === 'i32_value' &&
    castSlot.min_byte_length === 4 &&
    castSlot.max_byte_length === 4 &&
    castSlot.encoding === 'i32_le' &&
    castSlot.source_shape?.method === 'castAbilityBySlot',
  'cast-slot command payload contract drifted',
);

for (const needle of [
  'var castSlotCommand = payloads.castSlotCommandId(true);',
  'if (commandId == castSlotCommand) {',
  'applyTemporalReversalCastSlotCommand(',
  'castSlotIndexFromPayload(payloadBytes: container.Array<uint>, payloadOffset: uint): int',
  'return bits > <uint>2147483647 ? -1 : <int>bits;',
  'knownAbilityCodeAtSlot(state: WorldState, entityIndex: int, slot: int): uint',
  'slot >= end - start',
  'startTemporalReversalCast(state: WorldState, casterIndex: int, targetId: uint): void',
  'if (abilityCode != temporalReversalAbilityCode()) {',
  'throw "woc cast slot ability reducer is not implemented";',
  'pub temporalReversalSlotCommandStateTest(): int',
  'appendCastSlotCommand(',
  'if (temporalReversalSlotCommandStateTest() != 1)',
]) {
  invariant(state.includes(needle), 'WOS20 cast-slot projection omitted: ' + needle);
}

process.stdout.write('checked WOS20 cast-slot source projection: ' + SOURCE_COMMIT.slice(0, 15) + '\n');

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function methodBlock(source, declaration) {
  const start = source.indexOf(declaration);
  invariant(start >= 0, 'missing source method: ' + declaration);
  const signatureEnd = source.indexOf('): void {', start);
  invariant(signatureEnd >= 0, 'missing source method signature: ' + declaration);
  const open = source.indexOf('{', signatureEnd);
  invariant(open >= 0, 'missing source method body: ' + declaration);
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
    if (character === "'" || character === '"' || character === String.fromCharCode(96)) {
      quote = character;
      continue;
    }
    if (character === '{') depth += 1;
    if (character === '}') {
      depth -= 1;
      if (depth === 0) return source.slice(start, index + 1);
    }
  }
  throw new Error('unterminated source method: ' + declaration);
}
