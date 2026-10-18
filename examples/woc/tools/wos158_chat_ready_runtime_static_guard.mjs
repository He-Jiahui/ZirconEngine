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

const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'chat':[\s\S]*?typeof msg\.text !== 'string'[\s\S]*?const text = msg\.text\.trim\(\)/,
  'source chat server admission drifted');
const sourceSim = source('src/sim/sim.ts');
requireText(sourceSim, /MAX_CHAT_MESSAGE_LEN = 255/,
  'source maximum chat length drifted');
const sourceChat = source('src/sim/social/chat.ts');
requireText(sourceChat,
  /text\.trim\(\)\.slice\(0, MAX_CHAT_MESSAGE_LEN\)[\s\S]*?chatAllowed[\s\S]*?\/\^\\\/ready\(\?:check\)\?\\s\*\$\/i[\s\S]*?readyCheckStart/,
  'source chat ready-route ordering drifted');
requireText(sourceChat,
  /const CHAT_BURST = 8[\s\S]*?const CHAT_REFILL = 2[\s\S]*?b\.tokens = Math\.min[\s\S]*?b\.tokens -= 1/,
  'source chat token bucket drifted');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
if (payloadContract.schema_version !== 60) throw new Error('WOS158 payload schema must be 60');
const payload = payloadContract.entries.find((entry) => entry.id === 37 && entry.name === 'chat');
if (!payload || payload.kind !== 'chat_text' || payload.min_byte_length !== 4 ||
    payload.max_byte_length !== 1024 || payload.max_utf8_bytes !== 1020 ||
    payload.max_utf16_code_units !== 255 || payload.source_shape?.method !== 'chat') {
  throw new Error('chat typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated,
  /chatCommandId\(required: bool\): uint[\s\S]*?return <uint>37[\s\S]*?if \(id == <uint>37\) \{ return 54; \}/,
  'generated chat payload metadata is missing');
const nativePayload = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
requireText(nativePayload,
  /pub struct ChatCommandPayload[\s\S]*?encode_utf8_id\(CHAT_COMMAND_ID[\s\S]*?encode_utf16\(\)\.count\(\)/,
  'native chat payload or UTF-16 admission is missing');
const clientIntent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(clientIntent,
  /Chat \{[\s\S]*?text: String[\s\S]*?ChatCommandPayload \{ text \}[\s\S]*?CHAT_COMMAND_ID/,
  'native client chat intent mapping is missing');

const chatWireState = read('scripts', 'woc_game', 'src', 'social', 'chat_wire_state.zr');
requireText(chatWireState,
  /maximumUtf16CodeUnits[\s\S]*?ecmaWhitespace[\s\S]*?utf16Units[\s\S]*?matchesReady[\s\S]*?routeFromPayload[\s\S]*?unicodeReady[\s\S]*?<uint>194[\s\S]*?<uint>160/,
  'chat state does not enforce the source text boundary and ready aliases');
requireText(chatWireState,
  /bucketCapacity\(\): float \{ return 8\.0; \}[\s\S]*?refillPerSecond\(\): float \{ return 2\.0; \}[\s\S]*?refilledTokens/,
  'chat state token bucket is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /writer\.u16\(schemaVersion, 1, 1\)/,
  'world-state writer schema must advance to WOS78');
requireText(world,
  /commandId == chatCommand[\s\S]*?applyChatCommand/,
  'authoritative command routing does not reach chat');
requireText(world,
  /applyChatCommand\([\s\S]*?routeFromPayload[\s\S]*?emptyRoute[\s\S]*?consumeChatToken[\s\S]*?readyRoute[\s\S]*?startPartyReadyCheck/,
  'authoritative chat ready route is incomplete');
requireText(world,
  /if \(schemaVersion >= <uint>78\)[\s\S]*?entityChatTokenBucketPresent[\s\S]*?entityChatTokenBuckets[\s\S]*?entityChatTokenAtMicros/,
  'WOS78 chat token migration is missing');
requireText(world,
  /pub chatCommandStateTest\(\): int[\s\S]*?entityChatTokenBuckets[\s\S]*?500000[\s\S]*?decodeState\(encodeState\(state\)\)/,
  'chat token regression is incomplete');
requireText(world,
  /partyReadyCheckCommandStateTest[\s\S]*?appendChatCommandForTest[\s\S]*?entityReadyCheckPartyIds/,
  'party ready-check regression does not use the chat route');

const main = read('scripts', 'woc_game', 'src', 'main.zr');
if ((main.match(/world_state[^\r\n]*WOS118/g) ?? []).length !== 2) {
  throw new Error('plugin metadata must publish WOS118 in both runtime paths');
}
const nativeLib = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
requireText(nativeLib,
  /WORLD_STATE_FORMAT: &str = "WOS118";[\s\S]*?WORLD_STATE_SCHEMA_VERSION: u16 = 118;/,
  'native WOS118 identity is missing');
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.totals.typed_contract_commands !== 157 ||
    coverage.totals.typed_contract_client_send_commands !== 156 ||
    coverage.totals.source_shape_only_commands !== 0 ||
    coverage.totals.unmapped_dispatch_commands !== 8) {
  throw new Error('WOS158 command payload coverage totals drifted');
}
const worldContract = read('contracts', 'world-state.md');
requireText(worldContract,
  /WOS82 mech chroma delta[\s\S]*?schema 47[\s\S]*?\/readycheck[\s\S]*?This\s+is\s+not\s+a\s+claim\s+that\s+the\s+remaining\s+chat\s+router\s+is\s+complete/,
  'world-state contract does not document the scoped WOS158 delta');

process.stdout.write(`WOS158 chat-ready static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
