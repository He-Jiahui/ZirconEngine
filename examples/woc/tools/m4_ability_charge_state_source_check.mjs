import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocSourceRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'src');
const lifecycle = gitShow('src/sim/combat/casting_lifecycle.ts');
const auras = gitShow('src/sim/combat/auras.ts');
const dispatch = gitShow('src/sim/combat/effect_dispatch.ts');
const persistence = gitShow('src/sim/cooldown_persist.ts');
const projection = readFileSync(resolve(wocSourceRoot, 'combat', 'ability_charge_state.zr'), 'utf8');
const testMain = readFileSync(
  resolve(wocSourceRoot, 'combat', 'ability_charge_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(readFileSync(
  resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game', 'woc_m4_ability_charge_state_tests.zrp'),
  'utf8',
));

for (const needle of [
  'const maxCharges = 1 + Math.max(0, Math.floor(bonusCharges));',
  'state.charges = Math.max(0, state.charges - 1);',
  'state.recharges ??= state.recharge > 0 ? [state.recharge] : [];',
  'state.recharges.push(cooldown);',
  'state.recharges.sort((a, b) => a - b);',
  'if (state.charges <= 0) p.cooldowns.set(abilityId, state.recharge);',
]) {
  invariant(lifecycle.includes(needle), `missing pinned ability-charge cast behavior: ${needle}`);
}
for (const needle of [
  'if (!state.recharges) {',
  'state.recharges = Array.from(',
  'state.recharges = state.recharges.map((t) => t - delta);',
  'while (state.recharges.length > 0 && state.recharges[0] <= 0) {',
  'state.charges = Math.min(state.maxCharges, state.charges + 1);',
  'state.recharges = [];',
]) {
  invariant(auras.includes(needle), `missing pinned ability-charge timer behavior: ${needle}`);
}
for (const needle of [
  'chargeState.recharges.pop();',
  'chargeState.recharge = chargeState.recharges[0] ?? 0;',
  'p.cooldowns.delete(\'raging_gale\');',
  'chargeState.charges = chargeState.maxCharges;',
  'chargeState.recharge = 0;',
]) {
  invariant(dispatch.includes(needle), `missing pinned ability-charge refund/reset behavior: ${needle}`);
}
for (const needle of [
  'recharges?: number[];',
  'recharges: state.recharges',
  'without them converts on the first recharge tick',
]) {
  invariant(persistence.includes(needle), `missing pinned ability-charge persistence behavior: ${needle}`);
}

for (const needle of [
  'pub class AbilityChargeState',
  'pub spendAbilityCharge(', 'pub seedLegacySequentialRecharge(',
  'pub tickAbilityCharges(', 'pub refundNewestAbilityCharge(',
  'pub resetFullAbilityCharges(', 'pub contractTest(): int',
  'state.rechargesMaterialized = true;',
  'appendRechargeSorted(state, state.rechargeLength);',
  'state.recharges.removeAt(state.recharges.length - 1);',
]) {
  invariant(projection.includes(needle), `ability-charge projection omitted: ${needle}`);
}
invariant(
  testMain.includes('%import("combat/ability_charge_state")') &&
    testMain.includes('charges.contractTest()'),
  'missing ability-charge test entry behavior',
);
invariant(
  testProject.name === 'woc_m4_ability_charge_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-ability-charge-state-tests' &&
    testProject.entry === 'combat/ability_charge_state_test_main',
  'ability-charge test project contract drifted',
);

const fixtureImporters = zrFiles(wocSourceRoot)
  .filter((path) => readFileSync(path, 'utf8').includes('%import("combat/ability_charge_state")'))
  .map((path) => relative(wocSourceRoot, path).replaceAll('\\', '/'))
  .sort();
invariant(
  JSON.stringify(fixtureImporters) === JSON.stringify(['combat/ability_charge_state_test_main.zr']),
  `ability_charge_state escaped the focused fixture boundary: ${fixtureImporters.join(', ')}`,
);

process.stdout.write(`checked M4 ability-charge source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function zrFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return zrFiles(path);
    return entry.isFile() && entry.name.endsWith('.zr') ? [path] : [];
  });
}
