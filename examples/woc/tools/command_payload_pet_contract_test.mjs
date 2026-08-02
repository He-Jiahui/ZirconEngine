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
  [53, 'pet_abandon', 'empty', 'abandonPet', []],
  [54, 'pet_rename', 'utf8_id', 'renamePet', ['name']],
  [55, 'pet_revive', 'empty', 'revivePet', []],
  [56, 'pet_attack', 'empty', 'petAttack', []],
  [58, 'pet_taunt', 'empty', 'petTaunt', []],
  [59, 'pet_auto_taunt', 'boolean', 'setPetAutoTaunt', ['enabled']],
  [61, 'pet_feed', 'utf8_id', 'feedPet', ['item']],
  [62, 'pet_heal', 'empty', 'healPet', []],
  [63, 'pet_mode', 'utf8_id', 'setPetMode', ['mode']],
];

for (const [id, name, kind, method, fields] of expected) {
  assert.deepEqual(payloads.entries.find((entry) => entry.id === id), {
    id,
    name,
    kind,
    min_byte_length: kind === 'empty' ? 0 : kind === 'boolean' ? 1 : 4,
    max_byte_length: kind === 'empty' ? 0 : kind === 'boolean' ? 1 : 260,
    ...(kind === 'utf8_id' ? { max_utf8_bytes: 256 } : {}),
    encoding: kind === 'empty' ? 'empty' : kind === 'boolean' ? 'u8_false_or_true' : 'u32_le_utf8',
    source_shape: { kind: 'client_send', method, fields },
  });
  assert.match(zr, new RegExp(`pub ${camel(name)}CommandId\\(required: bool\\): uint`));
  assert.match(rust, new RegExp(`pub const ${name.toUpperCase()}_COMMAND_ID: u16 = ${id};`));
}

assert.match(protocol, /pub struct PetRenameCommandPayload/);
assert.match(protocol, /pub struct PetAutoTauntCommandPayload/);
assert.match(protocol, /pub struct PetFeedCommandPayload/);
assert.match(protocol, /pub struct PetModeCommandPayload/);
for (const variant of [
  'AbandonPet', 'RenamePet', 'RevivePet', 'PetAttack', 'PetTaunt',
  'SetPetAutoTaunt', 'FeedPet', 'HealPet', 'SetPetMode',
]) {
  assert.match(input, new RegExp(`\\b${variant}\\b`));
}
assert.equal(coverage.totals.typed_contract_commands, 148);
assert.equal(coverage.totals.typed_contract_client_send_commands, 147);
assert.equal(coverage.totals.source_shape_only_commands, 9);

process.stdout.write('pet command payload contracts are complete\n');

function camel(name) {
  return name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}
