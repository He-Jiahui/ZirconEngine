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
  /moveInventoryItem\(from: number, to: number\)[\s\S]*?cmd: 'inv_move', from, to/,
  'source inventory-move client payload shape drifted');
const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'inv_move':[\s\S]*?typeof msg\.from === 'number'[\s\S]*?typeof msg\.to === 'number'[\s\S]*?sim\.moveInventoryItem\(msg\.from, msg\.to, pid\)/,
  'source inventory-move server admission drifted');
const sourceOrder = source('src/sim/inventory_order.ts');
requireText(sourceOrder,
  /export function moveStackToCell[\s\S]*?Number\.isInteger\(from\)[\s\S]*?Number\.isInteger\(to\)[\s\S]*?fromCell === to[\s\S]*?stack\.slot = cell[\s\S]*?moved\.slot = to[\s\S]*?displaced\.slot = fromCell/,
  'source fixed-cell inventory move semantics drifted');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 60) {
  throw new Error('WOS151 command payload schema must be 60');
}
const inventoryMove = contract.entries.find((entry) => entry.id === 21 && entry.name === 'inv_move');
if (!inventoryMove || inventoryMove.kind !== 'i32_pair' ||
    inventoryMove.min_byte_length !== 8 || inventoryMove.max_byte_length !== 8 ||
    inventoryMove.encoding !== 'i32_le_from+i32_le_to') {
  throw new Error('inv_move typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated,
  /inventoryMoveCommandId\(required: bool\): uint[\s\S]*?return <uint>21/,
  'generated inventory-move command id is missing');
requireText(generated,
  /pub payloadKind[\s\S]*?if \(id == <uint>21\) \{ return 50; \}/,
  'generated inventory-move payload kind is missing');
requireText(generated,
  /pub payloadMinLength[\s\S]*?if \(id == <uint>21\) \{ return 8; \}/,
  'generated inventory-move minimum payload bound is missing');
requireText(generated,
  /pub payloadMaxLength[\s\S]*?if \(id == <uint>21\) \{ return 8; \}/,
  'generated inventory-move maximum payload bound is missing');

const protocol = read('native', 'crates', 'woc_protocol', 'src', 'inventory_move_payload.rs');
requireText(protocol,
  /pub struct InventoryMovePayload[\s\S]*?pub from: i32[\s\S]*?pub to: i32[\s\S]*?pub fn encode[\s\S]*?pub fn decode/,
  'native inventory-move payload type is missing');
const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /MoveInventoryItem \{[\s\S]*?from: i32[\s\S]*?to: i32[\s\S]*?InventoryMovePayload/,
  'native client inventory-move intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world,
  /pub var entityInventoryStackManualSlots: container\.Array<int>/,
  'authoritative inventory manual-cell column is missing');
requireText(world,
  /writer\.u16\(schemaVersion[\s\S]*?entityInventoryStackManualSlots/,
  'WOS72 snapshot does not persist manual cells');
requireText(world,
  /schemaVersion >= <uint>72[\s\S]*?entityInventoryStackManualSlots\.add[\s\S]*?else[\s\S]*?entityInventoryStackManualSlots\.add\(-1\)/,
  'WOS72 decode does not restore manual cells or default legacy stacks');
requireText(world,
  /applyInventoryMoveCommand[\s\S]*?protocolBinary\.readI32LeAt[\s\S]*?moveM5InventoryStackToCell/,
  'authoritative inventory-move reducer is missing');
requireText(world,
  /moveM5InventoryStackToCell[\s\S]*?fromCell[\s\S]*?displaced[\s\S]*?entityInventoryStackManualSlots/,
  'authoritative fixed-cell move semantics are missing');
requireText(world,
  /commandId == inventoryMoveCommand[\s\S]*?applyInventoryMoveCommand/,
  'authoritative command routing does not reach inventory move');
requireText(world,
  /pub inventoryMoveCommandStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?encodeState/,
  'inventory-move authoritative regression is missing');
requireText(world,
  /if \(inventoryMoveCommandStateTest\(\) != 1\) \{[\s\S]*?return -145;/,
  'world selfTest must execute WOS151 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.entries.find((entry) => entry.id === 21)?.transport_coverage !== 'typed_contract') {
  throw new Error('inventory-move coverage projection is not typed_contract');
}

process.stdout.write(`WOS151 inventory-move static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
