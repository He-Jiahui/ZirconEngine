import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const payloads = JSON.parse(
  readFileSync(join(projectRoot, 'contracts', 'command_payloads.json'), 'utf8'),
);
const zr = readFileSync(
  join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr'),
  'utf8',
);
const rust = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'),
  'utf8',
);
const protocol = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'),
  'utf8',
);
const input = readFileSync(
  join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

const expected = [
  [132, 'guild_event_create', 'guild_event_create', 'guildEventCreate', ['day', 'hour', 'title', 'note']],
  [72, 'friend_add', 'utf8_id', 'friendAdd', ['name']],
  [73, 'friend_remove', 'utf8_id', 'friendRemove', ['name']],
  [74, 'block_add', 'utf8_id', 'blockAdd', ['name']],
  [75, 'block_remove', 'utf8_id', 'blockRemove', ['name']],
  [77, 'guild_create', 'utf8_id', 'guildCreate', ['name']],
  [78, 'guild_invite', 'utf8_id', 'guildInvite', ['name']],
  [79, 'guild_accept', 'empty', 'guildAccept', []],
  [80, 'guild_decline', 'empty', 'guildDecline', []],
  [81, 'guild_leave', 'empty', 'guildLeave', []],
  [82, 'guild_kick', 'utf8_id', 'guildKick', ['name']],
  [83, 'guild_promote', 'utf8_id', 'guildPromote', ['name']],
  [84, 'guild_demote', 'utf8_id', 'guildDemote', ['name']],
  [85, 'guild_transfer', 'utf8_id', 'guildTransfer', ['name']],
  [86, 'guild_disband', 'empty', 'guildDisband', []],
  [133, 'guild_event_remove', 'u32_index', 'guildEventRemove', ['id']],
  [160, 'ignore_add', 'utf8_id', 'ignoreAdd', ['name']],
  [161, 'ignore_remove', 'utf8_id', 'ignoreRemove', ['name']],
];

for (const [id, name, kind, method, fields] of expected) {
  if (kind === 'guild_event_create') {
    assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
      id,
      name,
      kind,
      min_byte_length: 13,
      max_byte_length: 863,
      max_utf8_bytes: 640,
      max_day_utf8_bytes: 10,
      max_title_utf8_bytes: 192,
      max_note_utf8_bytes: 640,
      encoding: 'u32_le_utf8_day+u8_presence+f64_le_hour+u32_le_utf8_title+u32_le_utf8_note',
      source_shape: { kind: 'client_send', method, fields },
    });
    assert.match(zr, new RegExp(`pub ${camel(name)}CommandId\\(required: bool\\): uint`));
    assert.match(rust, new RegExp(`pub const ${name.toUpperCase()}_COMMAND_ID: u16 = ${id};`));
    continue;
  }
  assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
    id,
    name,
    kind,
    min_byte_length: kind === 'empty' ? 0 : kind === 'u32_index' ? 4 : 4,
    max_byte_length: kind === 'empty' || kind === 'u32_index' ? 0 + (kind === 'u32_index' ? 4 : 0) : 260,
    ...(kind === 'utf8_id' ? { max_utf8_bytes: 256 } : {}),
    encoding: kind === 'empty' ? 'empty' : kind === 'u32_index' ? 'u32_le' : 'u32_le_utf8',
    source_shape: { kind: 'client_send', method, fields },
  });
  assert.match(zr, new RegExp(`pub ${camel(name)}CommandId\\(required: bool\\): uint`));
  assert.match(rust, new RegExp(`pub const ${name.toUpperCase()}_COMMAND_ID: u16 = ${id};`));
}

assert.match(protocol, /pub struct SocialNameCommandPayload/);
assert.match(protocol, /pub struct GuildEventCreateCommandPayload/);
assert.match(protocol, /pub struct GuildEventRemoveCommandPayload/);
for (const variant of [
  'AddFriend', 'RemoveFriend', 'AddBlock', 'RemoveBlock', 'CreateGuild', 'InviteToGuild',
  'AcceptGuildInvite', 'DeclineGuildInvite', 'LeaveGuild', 'KickGuildMember',
  'PromoteGuildMember', 'DemoteGuildMember', 'TransferGuildLeadership', 'DisbandGuild',
  'CreateGuildEvent', 'RemoveGuildEvent', 'AddIgnore', 'RemoveIgnore',
]) {
  assert.match(input, new RegExp(`\\b${variant}\\b`));
}
assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);

process.stdout.write('social command payload contracts are complete\n');

function camel(name) {
  return name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}
