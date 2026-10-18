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
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'command_payload.rs');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
const zr = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const entrance = JSON.parse(read('contracts', 'm3_dungeon_entrance.json'));

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 112), {
  id: 112,
  name: 'enter_dungeon',
  kind: 'utf8_id',
  min_byte_length: 4,
  max_byte_length: 260,
  max_utf8_bytes: 256,
  encoding: 'u32_le_utf8',
  source_shape: { kind: 'client_send', method: 'enterDungeon', fields: ['dungeon'] },
});
assert.match(
  readSource('src', 'net', 'online.ts'),
  /enterDungeon\(dungeonId: string\): Promise<boolean>\s*\{\s*return this\.cmdWithOutcome\(\{ cmd: 'enter_dungeon', dungeon: dungeonId \}\);/,
);
assert.match(
  readSource('src', 'net', 'online.ts'),
  /this\.rawCmd\(\{ \.\.\.payload, rid \}\);/,
);
assert.match(
  readSource('src', 'net', 'online.ts'),
  /msg\.t === 'commandOutcome'[\s\S]*?this\.resolveCommandOutcome\(msg\.rid, msg\.ok\);/,
);
assert.match(readSource('server', 'game.ts'), /case 'enter_dungeon':/);
assert.match(
  readSource('server', 'game.ts'),
  /\(x\) => x\.templateId === 'dungeon_door' && x\.dungeonId === dungeonId/,
);
assert.match(
  readSource('server', 'game.ts'),
  /this\.send\(session, \{ t: 'commandOutcome', rid: msg\.rid, ok: succeeded \}\);/,
);
assert.match(readSource('src', 'sim', 'instances', 'dungeons.ts'), /export function enterDungeon\(/);
assert.equal(entrance.dungeons.length, 6);
assert.deepEqual(entrance.dungeons.map((entry) => entry.id), [
  'hollow_crypt', 'sunken_bastion', 'gravewyrm_sanctum', 'drowned_temple',
  'nythraxis_crypt', 'nythraxis_boss_arena',
]);
assert.match(zr, /pub enterDungeonCommandId\(required: bool\): uint/);
assert.match(generated, /pub const ENTER_DUNGEON_COMMAND_ID: u16 = 112;/);
assert.match(protocol, /pub struct EnterDungeonCommandPayload/);
assert.match(protocol, /encode_utf8_id\(ENTER_DUNGEON_COMMAND_ID, &self\.dungeon_id\)/);
assert.match(input, /EnterDungeon \{\s*dungeon_id: String,/);
assert.match(
  input,
  /ClientGameplayIntent::EnterDungeon \{ dungeon_id \} =>\s*\{\s*EnterDungeonCommandPayload/,
);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);
assert.equal(coverage.entries.find((entry) => entry.id === 112)?.transport_coverage, 'typed_contract');

process.stdout.write('Enter-dungeon command payload contract is complete\n');
