import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const dispatch = gitShow('src/sim/combat/effect_dispatch.ts');
const heal = gitShow('src/sim/combat/heal.ts');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'power_echo_heal_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'power_echo_heal_state_test_main.zr'), 'utf8');
const testProject = JSON.parse(readFileSync(resolve(wocRoot, 'woc_m4_power_echo_heal_state_tests.zrp'), 'utf8'));

for (const needle of [
  'const healed = ctx.applyHeal(p, healTarget, healAmount, ability.name, ability.id);',
  "if (isSpell) {", "a.kind === 'power_echo'", 'p.auras.splice(echoIdx, 1);',
  'if (!healTarget.dead && healed > 0)', 'const echoHeal = Math.max(1, Math.round(healed * echoAura.value));',
  'ctx.applyHeal(p, healTarget, echoHeal, ability.name, ability.id, false, false);',
]) invariant(dispatch.includes(needle), `missing pinned Power Echo heal behavior: ${needle}`);
for (const needle of ['canCrit = true,', 'if (target.dead) return 0;', 'return healed;']) {
  invariant(heal.includes(needle), `missing pinned resolved-heal seam: ${needle}`);
}
for (const needle of [
  'pub resolvePowerEchoHeal(', 'if (!isSpell || !state.powerEchoArmed) return;',
  'state.powerEchoArmed = false;', 'state.auraConsumed = true;',
  'if (state.targetDead || resolvedHeal <= 0) return;', 'state.echoCanCrit = false;',
]) invariant(projection.includes(needle), `Power Echo heal projection omitted: ${needle}`);
invariant(testMain.includes('%import("combat/power_echo_heal_state")') && testMain.includes('echo.contractTest()'), 'missing Power Echo heal test entry');
invariant(testProject.name === 'woc_m4_power_echo_heal_state_tests' && testProject.source === 'src' && testProject.binary === 'bin-m4-power-echo-heal-state-tests' && testProject.entry === 'combat/power_echo_heal_state_test_main', 'Power Echo heal test project contract drifted');

process.stdout.write(`checked M4 Power Echo heal source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'utf8' }); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
