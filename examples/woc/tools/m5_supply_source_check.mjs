import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// supply.ts is currently an untracked upstream worktree file, so it cannot be
// retrieved from 5ef9f7cb. Pin its exact bytes until its owner commits it.
const SOURCE_WORKTREE_SHA256 = '220ee4ff05ede7b402c1fa0412b15ea4fa7fa47c79da0a4950a1a96509fd92ff';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourcePath = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'supply.ts');
const sourceBytes = readFileSync(sourcePath);
const source = sourceBytes.toString('utf8');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'progression', 'supply_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'progression', 'supply_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m5_supply_state_tests.zrp'), 'utf8'),
);

invariant(
  createHash('sha256').update(sourceBytes).digest('hex') === SOURCE_WORKTREE_SHA256,
  'upstream supply worktree bytes changed; commit or deliberately refresh this projection',
);

for (const needle of [
  'export const SUPPLY_OBSERVATION_WIDTH = 15;',
  'export const SUPPLY_OBSERVATION_COUNT_CAP = 16;',
  'export const SUPPLY_COPPER_CAP = 1_000;',
  "food: { itemId: 'baked_bread', count: 5 },",
  "drink: { itemId: 'spring_water', count: 5 },",
  "healingPotion: { itemId: 'minor_healing_potion', count: 1 },",
  "manaPotion: { itemId: 'minor_mana_potion', count: 1 },",
  "if (kind === 'food') foodCount += stack.count;",
  "if (kind === 'drink') drinkCount += stack.count;",
  "if (kind === 'potion' && (item?.potionHp ?? 0) > 0)",
  "if (kind === 'potion' && (item?.potionMana ?? 0) > 0)",
  'sim.copper >= price * count',
  'canAddItem(sim.inventory, capacity, itemId, count)',
  'freeBagSlots: Math.max(0, capacity - used),',
]) {
  invariant(source.includes(needle), `source supply rule drifted: ${needle}`);
}

for (const needle of [
  'pub supplyObservationWidth(): int { return 15; }',
  'pub supplyObservationCountCap(): int { return 16; }',
  'pub supplyCopperCap(): int { return 1000; }',
  'if (kind == "food") { output.foodCount = output.foodCount + count; }',
  'if (kind == "drink") { output.drinkCount = output.drinkCount + count; }',
  'itemMetric(itemId, "potionHp") > 0.0',
  'itemMetric(itemId, "potionMana") > 0.0',
  'output.potionReady = state.potionCooldownRemaining <= 0.0;',
  'restockFeasible(state, "baked_bread", 5)',
  'restockFeasible(state, "spring_water", 5)',
  'restockFeasible(state, "minor_healing_potion", 1)',
  'restockFeasible(state, "minor_mana_potion", 1)',
  'appendBoolean(observation, status.foodCount > 0);',
  'appendBoolean(observation, status.manaPotionRestockFeasible);',
]) {
  invariant(projection.includes(needle), `WOC supply projection is missing: ${needle}`);
}

assertOrder(projection, [
  'appendBoolean(observation, status.foodCount > 0);',
  'appendBoolean(observation, status.drinkCount > 0);',
  'appendBoolean(observation, status.healingPotionCount > 0);',
  'appendBoolean(observation, status.manaPotionCount > 0);',
  'appendBoolean(observation, status.potionReady);',
  'appendBoolean(observation, status.foodRestockFeasible);',
  'appendBoolean(observation, status.manaPotionRestockFeasible);',
]);

for (const needle of [
  '%import("progression/supply_state")',
  'observation.values.length != supply.supplyObservationWidth()',
  'constrained.potionCooldownRemaining = 0.5;',
  'poor.copper = 24;',
]) {
  invariant(testMain.includes(needle), `WOC supply contract is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m5_supply_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m5-supply-state-tests' &&
    testProject.entry === 'progression/supply_state_test_main',
  'supply state test project contract drifted',
);

process.stdout.write(`checked M5 supply worktree: ${SOURCE_WORKTREE_SHA256.slice(0, 15)}\n`);

function assertOrder(text, needles) {
  let prior = -1;
  for (const needle of needles) {
    const position = text.indexOf(needle);
    invariant(position >= 0, `missing ordered rule: ${needle}`);
    invariant(position > prior, `source order drifted at: ${needle}`);
    prior = position;
  }
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
