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
const payload = readFileSync(
  join(projectRoot, 'native', 'crates', 'woc_protocol', 'src', 'world_object_payload.rs'),
  'utf8',
);
const input = readFileSync(
  join(projectRoot, 'native', 'apps', 'woc_client', 'src', 'input', 'intent.rs'),
  'utf8',
);
const coverage = JSON.parse(
  readFileSync(join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json'), 'utf8'),
);

assert.equal(payloads.schema_version, 51);
const expected = [
  [12, 'loot', 'lootCorpse', 'loot', 'LOOT_COMMAND_ID', 'LootCorpse', ['id']],
  [15, 'pickup', 'pickUpObject', 'pickup', 'PICKUP_COMMAND_ID', 'PickUpObject', ['id']],
  [117, 'delve_interact', 'delveInteract', 'delveInteract', 'DELVE_INTERACT_COMMAND_ID', 'InteractWithDelveObject', ['objectId']],
  [123, 'collect_delve_chest_loot', 'collectDelveChestLoot', 'collectDelveChestLoot', 'COLLECT_DELVE_CHEST_LOOT_COMMAND_ID', 'CollectDelveChestLoot', ['objectId']],
  [134, 'autoloot', 'autoLoot', 'autoLoot', 'AUTO_LOOT_COMMAND_ID', 'AutoLoot', ['id']],
];
for (const [id, name, method, zrFunction, rustConstant, intent, fields] of expected) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  assert.equal(entry?.name, name);
  assert.equal(entry?.kind, 'world_object_id');
  assert.equal(entry?.min_byte_length, 8);
  assert.equal(entry?.max_byte_length, 8);
  assert.equal(entry?.encoding, 'f64_le_world_object_id');
  assert.deepEqual(entry?.source_shape, { kind: 'client_send', method, fields });
  assert.match(zr, new RegExp(`pub ${zrFunction}CommandId\\(required: bool\\): uint`));
  assert.match(generated, new RegExp(`pub const ${rustConstant}: u16 = ${id};`));
  assert.match(input, new RegExp(`\\b${intent}\\b`));
}
for (const symbol of [
  'WorldObjectAction',
  'WorldObjectIdPayload',
  'validate_world_object_id_payload',
]) {
  assert.match(payload, new RegExp(`\\b${symbol}\\b`));
}
assert.match(protocol, /CommandPayloadKind::WorldObjectId/);

assert.equal(coverage.totals.typed_contract_commands, 148);
assert.equal(coverage.totals.typed_contract_client_send_commands, 147);
assert.equal(coverage.totals.source_shape_only_commands, 9);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

process.stdout.write('world-object command payload contracts are complete\n');
