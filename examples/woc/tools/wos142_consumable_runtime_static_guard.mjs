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
requireText(items, /def\.kind === 'food' \|\| def\.kind === 'drink'[\s\S]*?p\.inCombat[\s\S]*?isSwimming[\s\S]*?p\.sitting = true[\s\S]*?CONSUME_DURATION/,
  'source food/drink admission drifted');
const auras = source('src/sim/combat/auras.ts');
requireText(auras, /tickCount % 40[\s\S]*?\['eating', 'drinking'\][\s\S]*?hpPer2s[\s\S]*?manaPer2s[\s\S]*?remaining -= 2/,
  'source two-second consumable tick drifted');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /entityEatingItemCodes[\s\S]*?entityEatingHpPerTwoSeconds[\s\S]*?entityEatingRemaining[\s\S]*?entityDrinkingItemCodes[\s\S]*?entityDrinkingManaPerTwoSeconds[\s\S]*?entityDrinkingRemaining/,
  'WOS consumable state columns are missing');
requireText(world, /applyOfflineUseItemCommand[\s\S]*?kind == "food" \|\| kind == "drink"[\s\S]*?entitySitting/,
  'food/drink use reducer is missing');
requireText(world, /stepOfflineConsumables[\s\S]*?state\.tick % <uint>40[\s\S]*?entityHp[\s\S]*?entityResources/,
  'food/drink fixed-tick progression is missing');
requireText(world, /clearOfflineConsumables[\s\S]*?entityEatingRemaining[\s\S]*?entityDrinkingRemaining/,
  'food/drink interruption cleanup is missing');
requireText(world, /standUpRequested[\s\S]*?entitySitting\[index\] = false;\s*clearOfflineConsumables\(state, index\)/,
  'movement interruption wiring is missing');
requireText(world, /clearOfflineBreakableIncapacitateOnDamage[\s\S]*?if \(amount <= 0\)[\s\S]*?clearOfflineConsumables\(state, targetIndex\)/,
  'damage interruption wiring is missing');
requireText(world, /writer\.u16\(<uint>71, 1, 1\)/,
  'WOS71 encoder schema is missing');
requireText(world, /schemaVersion != <uint>68 &&\s*schemaVersion != <uint>69/,
  'WOS71 decoder admission is missing');
requireText(world, /if \(schemaVersion >= <uint>69\)[\s\S]*?entityEatingItemCodes[\s\S]*?entityDrinkingItemCodes/,
  'WOS71 consumable tail decode is missing');
requireText(world, /pub consumableCommandStateTest\(\): int[\s\S]*?tough_jerky[\s\S]*?spring_water[\s\S]*?encodeState[\s\S]*?stepOfflineConsumables/,
  'consumable state regression coverage is missing');
requireText(world, /if \(consumableCommandStateTest\(\) != 1\) \{[\s\S]*?return -136;/,
  'world selfTest must execute consumable coverage');
const main = read('scripts', 'woc_game', 'src', 'main.zr');
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'lib.rs');
if ((main.match(/world_state[^\r\n]*WOS71/g) ?? []).length !== 2 ||
    !protocol.includes('WORLD_STATE_FORMAT: &str = "WOS71"') ||
    !protocol.includes('WORLD_STATE_SCHEMA_VERSION: u16 = 71')) {
  throw new Error('WOS71 public protocol identity is missing');
}

process.stdout.write(`WOS142 consumable static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
