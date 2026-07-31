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

const interaction = source('src/sim/interaction.ts');
requireText(interaction, /autoLootForParty[\s\S]*?r\.e\.dead[\s\S]*?isInRaidInstance[\s\S]*?dist2d[\s\S]*?corpseLootRights[\s\S]*?false[\s\S]*?lootCorpse\(ctx, mobId, meta\.entityId, false, true\)/,
  'source autoloot silent no-FFA delegation drifted');
const payloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(payloads, /pub autoLootCommandId\(required: bool\): uint[\s\S]*?return <uint>134/,
  'autoloot command identity is missing');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /var autoLootCommand = payloads\.autoLootCommandId\(true\)/,
  'autoloot command reducer binding is missing');
requireText(world, /else if \(commandId == autoLootCommand\)[\s\S]*?applyOfflineAutoLootCommand/,
  'autoloot dispatch is missing');
requireText(world, /applyOfflineAutoLootCommand[\s\S]*?applyOfflineCorpseLootCommand/,
  'autoloot must reuse the authoritative corpse-loot reducer');
requireText(world, /pub autoLootCommandStateTest\(\): int[\s\S]*?autoLootCommandId[\s\S]*?entityLootable[\s\S]*?entityLastCommandSequence/,
  'autoloot state regression coverage is missing');
requireText(world, /if \(autoLootCommandStateTest\(\) != 1\) \{[\s\S]*?return -134;/,
  'world selfTest must execute autoloot coverage');

process.stdout.write(`WOS140 autoloot static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
