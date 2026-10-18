import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = path.resolve(root, '..', '..');
const sourceRoot = path.resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), 'utf8');
const source = (file) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${file}`], { encoding: 'utf8' },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const sourceTypes = source('src/sim/types.ts');
for (const id of [
  'wave', 'laugh', 'question', 'cheer', 'dance', 'point', 'flex',
  'salute', 'cry', 'bow', 'clap', 'roar', 'kneel',
]) {
  requireText(sourceTypes, new RegExp(`'${id}'`), `source emote id ${id} drifted`);
}
const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'emote':[\s\S]*?isOverheadEmoteId\(msg\.emote\)[\s\S]*?sim\.playEmote\(msg\.emote, pid\)/,
  'source emote server admission drifted');
const sourceChat = source('src/sim/social/chat.ts');
requireText(sourceChat,
  /OVERHEAD_EMOTE_DURATION = 3\.2[\s\S]*?playEmote[\s\S]*?overheadEmoteId = emoteId[\s\S]*?overheadEmoteUntil = ctx\.time \+ OVERHEAD_EMOTE_DURATION[\s\S]*?overheadEmoteSeq \+= 1/,
  'source emote state transition drifted');
const sourceRoster = source('src/sim/entity_roster.ts');
requireText(sourceRoster,
  /overheadEmoteId && ctx\.time >= e\.overheadEmoteUntil[\s\S]*?overheadEmoteId = null[\s\S]*?overheadEmoteUntil = 0/,
  'source emote expiration drifted');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 60) throw new Error('WOS152 command payload schema must be 60');
const emote = contract.entries.find((entry) => entry.id === 38 && entry.name === 'emote');
if (!emote || emote.kind !== 'emote_id' || emote.min_byte_length !== 1 ||
    emote.max_byte_length !== 1 || emote.encoding !== 'u8_emote_id') {
  throw new Error('emote typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated, /emoteCommandId\(required: bool\): uint[\s\S]*?return <uint>38/,
  'generated emote command id is missing');
requireText(generated, /pub payloadKind[\s\S]*?if \(id == <uint>38\) \{ return 51; \}/,
  'generated emote payload kind is missing');
requireText(generated, /pub payloadLength[\s\S]*?if \(id == <uint>38\) \{ return 1; \}/,
  'generated emote payload length is missing');

const protocol = read('native', 'crates', 'woc_protocol', 'src', 'emote_payload.rs');
requireText(protocol,
  /pub enum EmoteId[\s\S]*?Wave[\s\S]*?Kneel[\s\S]*?pub struct EmoteCommandPayload[\s\S]*?pub emote: EmoteId/,
  'native closed emote payload type is missing');
const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /PlayEmote \{[\s\S]*?emote: EmoteId[\s\S]*?EmoteCommandPayload/,
  'native client emote intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
for (const column of [
  'entityOverheadEmoteCodes', 'entityOverheadEmoteUntil', 'entityOverheadEmoteSequences',
]) requireText(world, new RegExp(`pub var ${column}:`), `WOS73 ${column} is missing`);
requireText(world, /writer\.u16\(schemaVersion[\s\S]*?entityOverheadEmoteCodes[\s\S]*?entityOverheadEmoteUntil[\s\S]*?entityOverheadEmoteSequences/,
  'WOS73 snapshot does not persist emote state');
requireText(world, /schemaVersion >= <uint>73[\s\S]*?entityOverheadEmoteCodes\.add[\s\S]*?entityOverheadEmoteUntil\.add[\s\S]*?entityOverheadEmoteSequences\.add/,
  'WOS73 decoder does not restore emote state');
requireText(world, /applyEmoteCommand[\s\S]*?payloadBytes[\s\S]*?3\.2[\s\S]*?entityOverheadEmoteSequences/,
  'authoritative emote reducer is missing');
requireText(world, /expireOverheadEmotes[\s\S]*?timeMicros[\s\S]*?entityOverheadEmoteCodes[\s\S]*?entityOverheadEmoteUntil/,
  'fixed-tick emote expiration is missing');
requireText(world, /commandId == emoteCommand[\s\S]*?applyEmoteCommand/,
  'authoritative command routing does not reach emote');
requireText(world, /pub emoteCommandStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?encodeState[\s\S]*?expireOverheadEmotes/,
  'emote authoritative regression is missing');
requireText(world, /if \(emoteCommandStateTest\(\) != 1\) \{[\s\S]*?return -146;/,
  'world selfTest must execute WOS152 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0) {
  throw new Error('WOS152 command payload coverage totals drifted');
}

process.stdout.write(`WOS152 emote static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
