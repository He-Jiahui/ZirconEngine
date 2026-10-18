import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const readSource = (...parts) => readFileSync(join(sourceRoot, ...parts), 'utf8');
const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
const generated = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'challenge_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const pluginFailure = read('..', '..', 'docs', 'plans', 'zircon_plugins', '08', 'failure-2026-07-18-woc-zrvm-deterministic-bulk-cross-platform-runtime.md');

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 36), {
  id: 36,
  name: 'challengeResponse',
  kind: 'challenge_response',
  min_byte_length: 12,
  max_byte_length: 780,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8_nonce+u32_le_utf8_response+u32_le_utf8_signature',
  source_shape: { kind: 'client_send', method: 'onMessage', fields: ['n', 'r', 'sig'] },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /msg\.t === 'challenge'[\s\S]*?signChallenge\(msg\.nonce, challengeResponse, this\.clientSeed\)[\s\S]*?cmd: 'challengeResponse', n: msg\.nonce, r: challengeResponse, sig: signature/,
);
assert.match(
  readSource('server', 'game.ts'),
  /case 'challengeResponse':[\s\S]*?typeof msg\.n === 'string'[\s\S]*?typeof msg\.r === 'string'[\s\S]*?typeof msg\.sig === 'string'[\s\S]*?verifyChallenge\(msg\.n, msg\.r, msg\.sig, session\.clientSeed\)/,
);
assert.match(
  readSource('src', 'sim', 'client_challenge.ts'),
  /function hashCyrb53[\s\S]*?str\.charCodeAt\(i\)[\s\S]*?Math\.imul[\s\S]*?toString\(36\)/,
);
assert.match(zr, /pub challengeResponseCommandId\(required: bool\): uint/);
assert.match(generated, /pub const CHALLENGE_RESPONSE_COMMAND_ID: u16 = 36;/);
assert.match(protocol, /pub struct ChallengeResponseCommandPayload/);
assert.match(input, /SendChallengeResponse \{\s*nonce: String,\s*response: String,\s*signature: String,/);
assert.match(
  input,
  /ClientGameplayIntent::SendChallengeResponse\s*\{\s*nonce,\s*response,\s*signature,?\s*\}\s*=>[\s\S]*?ChallengeResponseCommandPayload/,
);
assert.match(pluginFailure, /WOS171 challenge-response JavaScript arithmetic addendum/);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(coverage.entries.find((entry) => entry.id === 36)?.transport_coverage, 'typed_contract');

process.stdout.write('Challenge-response command payload contract is complete\n');
