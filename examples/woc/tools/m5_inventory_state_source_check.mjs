import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const inventory = gitShow('src/sim/items.ts');
const sim = gitShow('src/sim/sim.ts');
const rules = gitShow('src/sim/equipment_rules.ts');
const items = gitShow('src/sim/content/items.ts');
const scenarios = gitShow('tests/parity/scenarios.ts');
const compactRules = rules.replace(/\s+/g, '');
const inventoryScenario = functionBlock(scenarios, 'function inventoryVendor');
const wocSourceRoot = resolve(wocRoot, 'scripts', 'woc_game', 'src');

for (const needle of [
  'function desiredEquipSlot(meta: PlayerMeta, itemId: string): EquipSlot | null',
  "if (!def?.slot || (def.kind !== 'weapon' && def.kind !== 'armor' && def.kind !== 'held_offhand'))",
  'if (targetSlot && !slotAcceptsItem(def, targetSlot))',
  'const slot = targetSlot ?? desiredEquipSlot(meta, itemId);',
  'if (!canEquipItemInSlot(meta.cls, def, slot, spec))',
  'let displacedSlot: EquipSlot | null = null;',
  'export function revalidateOffhandForSpec(ctx: SimContext, pid?: number): void',
]) {
  invariant(inventory.includes(needle), `missing current inventory rule: ${needle}`);
}

for (const needle of [
  "const slot = def.kind === 'food' ? 'eating' : 'drinking';",
  'remaining: CONSUME_DURATION,',
  'if (ctx.time < p.potionCooldownUntil) {',
  "p.potionCooldownUntil = ctx.time + POTION_COOLDOWN;",
  "} else if (def.kind === 'elixir') {",
]) {
  invariant(inventory.includes(needle), `missing current consumable rule: ${needle}`);
}

for (const needle of [
  'private standUp(p: Entity): void {',
  'if (isConsuming(p)) {',
  'p.eating = null;',
  'p.drinking = null;',
]) {
  invariant(sim.includes(needle), `missing current consume-cancel rule: ${needle}`);
}

for (const needle of [
  "return cls === 'rogue' || (cls === 'warrior' && spec === 'fury');",
  "return cls === 'warrior' && spec === 'fury';",
  "if (item.kind === 'held_offhand') {",
  "if (slot !== 'offhand' || !canDualWield(cls, spec)) return false;",
  "return hand === 'onehand' || (hand === 'twohand' && canDualWieldTwoHand(cls, spec));",
]) {
  invariant(compactRules.includes(needle.replace(/\s+/g, '')),
    `missing current equipment rule: ${needle}`);
}

for (const [itemId, facts] of [
  ['worn_sword', ["kind: 'weapon'", "slot: 'mainhand'", 'weapon: { min: 2, max: 5, speed: 2.0 }']],
  ['gnarled_staff', ["kind: 'weapon'", "slot: 'mainhand'", 'weapon: { min: 3, max: 6, speed: 2.9 }', 'stats: { int: 1 }']],
  ['greyjaw_hide_boots', ["kind: 'armor'", "slot: 'feet'", 'stats: { armor: 28, agi: 1, sta: 1 }']],
  ['milepost_boots', ["kind: 'armor'", "slot: 'feet'", 'stats: { armor: 30, agi: 1, sta: 1 }']],
]) {
  const block = itemBlock(items, itemId);
  for (const fact of facts) {
    invariant(block.includes(fact), `selected ${itemId} fact drifted: ${fact}`);
  }
}

for (const needle of [
  "name: 'inventory_vendor',",
  "sim.equipItem('cryptbone_helm', buyer);",
  "sim.equipItem('roadwardens_helm', buyer);",
  "sim.unequipItem('helmet', buyer);",
  "sim.buyBackItem('wolf_fang', buyer);",
]) {
  invariant(inventoryScenario.includes(needle), `missing inventory_vendor scenario behavior: ${needle}`);
}

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("progression/inventory_vendor_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
const wocInventory = readFileSync(
  resolve(wocSourceRoot, 'progression', 'inventory_vendor_state.zr'),
  'utf8',
);
const wocCatalog = readFileSync(
  resolve(wocSourceRoot, 'generated', 'm5_content_catalog.zr'),
  'utf8',
);
invariant(
  JSON.stringify(fixtureImporters) ===
    JSON.stringify(['progression/inventory_vendor_state_test_main.zr', 'progression/m5_scenario_matrix.zr']),
  `inventory_vendor_state escaped the M5 fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M5 inventory state source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

for (const needle of [
  'maxArmorRankForClass(playerClass: string): int',
  'armorRank(armorType: string): int',
  'canEquipCatalogItem(state: InventoryState, itemId: string): bool',
  'meetsCatalogLevel(state: InventoryState, itemId: string): bool',
  'var itemLevelRequirement = %import("progression/item_level_requirement_state")',
  'catalog.flag("item", index, "requiredLevelPresent")',
  'itemLevelRequirement.requiredLevelForCatalogItem(',
  'if (!meetsCatalogLevel(state, itemId)) { state.lastError = -8; return false; }',
  'catalog.itemAllowsClass(index, state.playerClass)',
  'catalog.text("item", index, "weaponHand") == "twohand"',
  'armorRules.playerClass = "paladin";',
  'clothRules.playerClass = "mage";',
]) {
  invariant(wocInventory.includes(needle), 'missing WOC equipment-rules projection: ' + needle);
}
for (const needle of [
  'pub advanceRegenTick(state: InventoryState): void',
  'state.eatingRemaining = state.eatingRemaining - 2.0;',
  'state.drinkingRemaining = state.drinkingRemaining - 2.0;',
  'pub advanceTimers(state: InventoryState, elapsed: float): void',
  'state.potionCooldownRemaining = state.potionCooldownRemaining - elapsed;',
  'state.elixirActive = false;',
  'state.resourceKind == "mana"',
  'consumables.resource != 72',
  'consumables.maxHp != 90',
  'pub standUp(state: InventoryState): void',
  'if (state.eatingItem != "" || state.drinkingItem != "")',
  'standUp(interrupted);',
]) {
  invariant(wocInventory.includes(needle), 'missing WOC consumable projection: ' + needle);
}
for (const needle of [
  'weaponHand: definition.weapon?.hand ??',
  'const explicitRequiredLevel = Number.isFinite(definition.requiredLevel)',
  'requiredLevel: explicitRequiredLevel,',
  'requiredLevelPresent: Number.isFinite(definition.requiredLevel),',
  'pub itemAllowsClass(index: int, cls: string): bool {',
]) {
  const present = needle.startsWith('pub ')
    ? wocCatalog.includes(needle)
    : readFileSync(resolve(wocRoot, 'tools', 'm5_content_zr_codegen.mjs'), 'utf8').includes(needle);
  invariant(present, 'missing M5 equipment catalog projection: ' + needle);
}

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function functionBlock(source, declaration) {
  const start = source.indexOf(declaration);
  invariant(start >= 0, `missing source function: ${declaration}`);
  return bracedBlock(source, source.indexOf('{', start), declaration);
}

function itemBlock(source, itemId) {
  const start = source.indexOf(`  ${itemId}: {`);
  invariant(start >= 0, `missing selected item: ${itemId}`);
  return bracedBlock(source, source.indexOf('{', start), itemId);
}

function bracedBlock(source, open, label) {
  invariant(open >= 0, `missing source body: ${label}`);
  let depth = 0;
  let quote = '';
  let escaped = false;
  for (let index = open; index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === quote) quote = '';
      continue;
    }
    if (character === "'" || character === '"' || character === '`') {
      quote = character;
      continue;
    }
    if (character === '{') depth += 1;
    if (character === '}') {
      depth -= 1;
      if (depth === 0) return source.slice(open, index + 1);
    }
  }
  throw new Error(`unterminated source body: ${label}`);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
