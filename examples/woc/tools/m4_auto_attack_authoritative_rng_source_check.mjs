import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const wocRoot = resolve(workspaceRoot, 'examples', 'woc', 'scripts', 'woc_game');
const projection = readFileSync(resolve(wocRoot, 'src', 'combat', 'auto_attack_state.zr'), 'utf8');
const testMain = readFileSync(resolve(wocRoot, 'src', 'combat', 'auto_attack_state_test_main.zr'), 'utf8');

for (const needle of [
  'pub class AutoActor',
  'pub class AutoTarget',
  'pub class AutoEvents',
  'pub initializeAuthoritativeRng(',
  'pub authoritativeRngState(',
  'pub authoritativeRngDraws(',
  'pub authoritativeRngDigest(',
  'var rng = %import("kernel/rng")',
  'events.rngState = (events.rngState + <uint>1831565813) & <uint>4294967295;',
  'events.rngDigest = rng.fold(events.rngDigest, value, 1);',
  'pub authoritativeRngContractTest(): int',
]) {
  invariant(projection.includes(needle), `WOC authoritative auto-attack RNG is missing: ${needle}`);
}

invariant(
  testMain.includes('autoAttack.authoritativeRngContractTest()'),
  'missing authoritative auto-attack RNG test entry behavior',
);

process.stdout.write('checked M4 auto-attack authoritative RNG source\n');

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
