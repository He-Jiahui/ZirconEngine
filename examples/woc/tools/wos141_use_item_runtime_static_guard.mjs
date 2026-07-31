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

const items = source('src/sim/items.ts');
requireText(items, /useItem\(ctx[\s\S]*?countItem[\s\S]*?def\.kind === 'potion'[\s\S]*?potionCooldownUntil[\s\S]*?restoresMana[\s\S]*?restoresHp[\s\S]*?removeItem[\s\S]*?POTION_COOLDOWN/,
  'source potion item-use semantics drifted');
requireText(items, /def\.kind === 'weapon'[\s\S]*?equipItem[\s\S]*?def\.kind === 'bag'[\s\S]*?equipBagCmd/,
  'source equipment item-use delegation drifted');
const payloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(payloads, /pub useItemCommandId\(required: bool\): uint[\s\S]*?return <uint>23/,
  'use command identity is missing');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /var useItemCommand = payloads\.useItemCommandId\(true\)/,
  'use command reducer binding is missing');
requireText(world, /else if \(commandId == useItemCommand\)[\s\S]*?applyOfflineUseItemCommand/,
  'use command dispatch is missing');
requireText(world, /applyOfflineUseItemCommand[\s\S]*?m5InventoryItemCodeFromPayload[\s\S]*?entityPotionCooldownUntil[\s\S]*?removeM5InventoryItem[\s\S]*?equipM5InventoryItem[\s\S]*?equipM5InventoryBag/,
  'use command retained potion/equipment reducer is missing');
requireText(world, /updateOfflinePotionCooldowns[\s\S]*?entityPotionCooldownRemaining/,
  'potion cooldown aging is missing');
requireText(world, /pub useItemCommandStateTest\(\): int[\s\S]*?minor_healing_potion[\s\S]*?minor_mana_potion[\s\S]*?entityPotionCooldownUntil/,
  'use command state regression coverage is missing');
requireText(world, /if \(useItemCommandStateTest\(\) != 1\) \{[\s\S]*?return -135;/,
  'world selfTest must execute use-item coverage');

process.stdout.write(`WOS141 use-item static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
