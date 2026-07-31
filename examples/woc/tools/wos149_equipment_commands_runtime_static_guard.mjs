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

const sourceOnline = source('src/net/online.ts');
requireText(sourceOnline,
  /equipItem\(itemId: string\)[\s\S]*?cmd: 'equip', item: itemId[\s\S]*?equipItemToSlot\(itemId: string, slot: EquipSlot\)[\s\S]*?cmd: 'equip', item: itemId, slot[\s\S]*?unequipItem\(slot: EquipSlot\)[\s\S]*?cmd: 'unequip_item', slot/,
  'source equipment client payload shapes drifted');

const sourceTypes = source('src/sim/types.ts');
requireText(sourceTypes,
  /ALL_EQUIP_SLOTS[\s\S]*?'mainhand'[\s\S]*?'offhand'[\s\S]*?'helmet'[\s\S]*?'neck'[\s\S]*?'shoulder'[\s\S]*?'chest'[\s\S]*?'waist'[\s\S]*?'legs'[\s\S]*?'gloves'[\s\S]*?'feet'[\s\S]*?'ring1'[\s\S]*?'ring2'/,
  'source live equipment-slot order drifted');

const sourceItems = source('src/sim/items.ts');
requireText(sourceItems,
  /export function equipItem[\s\S]*?targetSlot && !slotAcceptsItem[\s\S]*?canEquipItem[\s\S]*?meetsLevelRequirement[\s\S]*?removeItem\(itemId, 1[\s\S]*?recalcPlayerStats/,
  'source equipment admission, transfer, or stat semantics drifted');
requireText(sourceItems,
  /export function unequipItem[\s\S]*?meta\.equipment\[slot\][\s\S]*?canAddItem[\s\S]*?delete meta\.equipment\[slot\][\s\S]*?addItemSilent[\s\S]*?recalcPlayerStats/,
  'source unequip capacity, transfer, or stat semantics drifted');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 38) {
  throw new Error('WOS149 command payload schema must be 38');
}
const equip = contract.entries.find((entry) => entry.id === 20 && entry.name === 'equip');
if (!equip || equip.kind !== 'equipment_item_optional_slot' ||
    equip.min_byte_length !== 5 || equip.max_byte_length !== 261 ||
    equip.encoding !== 'u32_le_utf8_item_id+u8_optional_equip_slot') {
  throw new Error('equip typed payload contract is missing or non-canonical');
}
const unequip = contract.entries.find((entry) => entry.id === 22 && entry.name === 'unequip_item');
if (!unequip || unequip.kind !== 'equipment_slot' ||
    unequip.min_byte_length !== 1 || unequip.max_byte_length !== 1 ||
    unequip.encoding !== 'u8_equip_slot') {
  throw new Error('unequip_item typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated,
  /equipItemCommandId\(required: bool\): uint[\s\S]*?return <uint>20[\s\S]*?unequipItemCommandId\(required: bool\): uint[\s\S]*?return <uint>22/,
  'generated equipment command ids are missing');
requireText(generated,
  /payloadKind\(<uint>20, 1\) == 47[\s\S]*?payloadMinLength\(<uint>20, true\) == 5[\s\S]*?payloadMaxLength\(<uint>20, true\) == 261[\s\S]*?payloadKind\(<uint>22, 1\) == 48/,
  'generated equipment payload kinds or bounds are missing');

const protocol = read('native', 'crates', 'woc_protocol', 'src', 'equipment_payload.rs');
requireText(protocol,
  /pub enum EquipmentSlot[\s\S]*?Mainhand[\s\S]*?Offhand[\s\S]*?Helmet[\s\S]*?Feet[\s\S]*?Ring2/,
  'native equipment-slot enum is missing');
requireText(protocol,
  /pub struct EquipItemPayload[\s\S]*?pub item_id: String[\s\S]*?pub slot: Option<EquipmentSlot>[\s\S]*?pub struct UnequipItemPayload/,
  'native equipment payload types are missing');

const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /EquipItem \{[\s\S]*?item_id: String[\s\S]*?slot: Option<EquipmentSlot>[\s\S]*?EquipItemPayload/,
  'native client equip intent mapping is missing');
requireText(intent,
  /UnequipItem \{[\s\S]*?slot: EquipmentSlot[\s\S]*?UnequipItemPayload/,
  'native client unequip intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world,
  /applyM5EquipItemCommand[\s\S]*?m5InventoryItemCodeFromPayload[\s\S]*?m5EquipmentSlotFromWireCode[\s\S]*?equipM5InventoryItem/,
  'authoritative equip reducer is missing');
requireText(world,
  /applyM5UnequipItemCommand[\s\S]*?m5EquipmentSlotFromWireCode[\s\S]*?unequipM5InventoryItem/,
  'authoritative unequip reducer is missing');
requireText(world,
  /commandId == equipItemCommand[\s\S]*?applyM5EquipItemCommand[\s\S]*?commandId == unequipItemCommand[\s\S]*?applyM5UnequipItemCommand/,
  'authoritative command routing does not reach equipment reducers');
requireText(world,
  /pub equipmentCommandStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?encodeState/,
  'equipment authoritative regression is missing');
requireText(world,
  /if \(equipmentCommandStateTest\(\) != 1\) \{[\s\S]*?return -143;/,
  'world selfTest must execute WOS149 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
for (const id of [20, 22]) {
  if (coverage.entries.find((entry) => entry.id === id)?.transport_coverage !== 'typed_contract') {
    throw new Error(`equipment command ${id} coverage projection is not typed_contract`);
  }
}

process.stdout.write(`WOS149 equipment command static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
