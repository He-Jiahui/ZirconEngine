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
const generated = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs'),
  'utf8',
);
const protocol = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'command_payload.rs'),
  'utf8',
);
const finder = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'dungeon_finder_payload.rs'),
  'utf8',
);
const input = readFileSync(
  join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

assert.equal(payloads.schema_version, 38);
const expected = [
  [150, 'df_roles', 'dungeonFinderSetRoles', 'dungeonFinderRoles', 'DUNGEON_FINDER_ROLES_COMMAND_ID', 'dungeon_finder_roles', 1, 4, 'u8_count_0_to_3+u8_finder_role', ['roles']],
  [151, 'df_queue', 'dungeonFinderQueueJoin', 'dungeonFinderQueue', 'DUNGEON_FINDER_QUEUE_COMMAND_ID', 'dungeon_finder_activities', 1, 3089, 'u8_count_0_to_16+u8_utf8_activity_id_max_64_utf16', ['activities']],
  [152, 'df_queue_leave', 'dungeonFinderQueueLeave', 'dungeonFinderQueueLeave', 'DUNGEON_FINDER_QUEUE_LEAVE_COMMAND_ID', 'empty', 0, 0, 'empty', []],
  [153, 'df_proposal', 'dungeonFinderRespond', 'dungeonFinderProposal', 'DUNGEON_FINDER_PROPOSAL_COMMAND_ID', 'boolean', 1, 1, 'u8_false_or_true', ['accept']],
  [154, 'df_list_create', 'dungeonFinderListingCreate', 'dungeonFinderListingCreate', 'DUNGEON_FINDER_LIST_CREATE_COMMAND_ID', 'dungeon_finder_listing', 2, 202, 'u8_utf8_activity_id_max_64_utf16+u8_count_0_to_8+u8_finder_listing_tag', ['activity', 'tags']],
  [155, 'df_list_close', 'dungeonFinderListingClose', 'dungeonFinderListingClose', 'DUNGEON_FINDER_LIST_CLOSE_COMMAND_ID', 'empty', 0, 0, 'empty', []],
  [156, 'df_apply', 'dungeonFinderApply', 'dungeonFinderApply', 'DUNGEON_FINDER_APPLY_COMMAND_ID', 'dungeon_finder_listing_id', 8, 8, 'f64_le_listing_id', ['listing']],
  [157, 'df_apply_cancel', 'dungeonFinderApplyCancel', 'dungeonFinderApplyCancel', 'DUNGEON_FINDER_APPLY_CANCEL_COMMAND_ID', 'empty', 0, 0, 'empty', []],
  [158, 'df_app_respond', 'dungeonFinderApplicationRespond', 'dungeonFinderApplicationRespond', 'DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID', 'dungeon_finder_application_response', 9, 9, 'f64_le_applicant_id+u8_false_or_true', ['applicant', 'accept']],
];
for (const [id, name, method, zrFunction, rustConstant, kind, min, max, encoding, fields] of expected) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  assert.equal(entry?.name, name);
  assert.equal(entry?.kind, kind);
  assert.equal(entry?.min_byte_length, min);
  assert.equal(entry?.max_byte_length, max);
  assert.equal(entry?.encoding, encoding);
  assert.deepEqual(entry?.source_shape, { kind: 'client_send', method, fields });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
}
assert.equal(payloads.entries.find((entry) => entry.id === 151)?.max_utf8_bytes, 192);
assert.equal(payloads.entries.find((entry) => entry.id === 151)?.max_utf16_code_units, 64);
assert.equal(payloads.entries.find((entry) => entry.id === 154)?.max_utf8_bytes, 192);
assert.equal(payloads.entries.find((entry) => entry.id === 154)?.max_utf16_code_units, 64);
for (const symbol of [
  'DungeonFinderRolesPayload',
  'DungeonFinderActivitiesPayload',
  'DungeonFinderListingPayload',
  'DungeonFinderListingIdPayload',
  'DungeonFinderApplicationResponsePayload',
]) {
  assert.match(finder, new RegExp(`\\b${symbol}\\b`));
}
for (const symbol of [
  'validate_dungeon_finder_roles_payload',
  'validate_dungeon_finder_activities_payload',
  'validate_dungeon_finder_listing_payload',
  'validate_dungeon_finder_listing_id_payload',
  'validate_dungeon_finder_application_response_payload',
]) {
  assert.match(protocol, new RegExp(`\\b${symbol}\\b`));
}
for (const intent of [
  'SetDungeonFinderRoles',
  'JoinDungeonFinderQueue',
  'LeaveDungeonFinderQueue',
  'RespondDungeonFinderProposal',
  'CreateDungeonFinderListing',
  'CloseDungeonFinderListing',
  'ApplyToDungeonFinderListing',
  'CancelDungeonFinderApplication',
  'RespondToDungeonFinderApplication',
]) {
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}

assert.equal(coverage.totals.typed_contract_commands, 135);
assert.equal(coverage.totals.typed_contract_client_send_commands, 134);
assert.equal(coverage.totals.source_shape_only_commands, 22);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('Dungeon Finder command payload contracts are complete\n');
