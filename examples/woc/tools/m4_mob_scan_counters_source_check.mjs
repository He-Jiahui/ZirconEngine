import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/mob/scan_counters.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'mob_scan_counters_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'world', 'mob_scan_counters_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m4_mob_scan_counters_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  'aggroScanPlayerVisits: number;',
  'threatEntryVisits: number;',
  'return { aggroScanPlayerVisits: 0, threatEntryVisits: 0 };',
  'c.aggroScanPlayerVisits = 0;',
  'c.threatEntryVisits = 0;',
]) {
  invariant(source.includes(needle), `source mob-scan counter rule drifted: ${needle}`);
}

for (const needle of [
  'pub var aggroScanPlayerVisits: int;',
  'pub var threatEntryVisits: int;',
  'pub createMobScanCounters(): MobScanCounters {',
  'return new MobScanCounters();',
  'pub resetMobScanCounters(counters: MobScanCounters): void {',
  'counters.aggroScanPlayerVisits = 0;',
  'counters.threatEntryVisits = 0;',
  'pub recordAggroScanPlayerVisit(counters: MobScanCounters): void {',
  'pub recordThreatEntryVisit(counters: MobScanCounters): void {',
]) {
  invariant(projection.includes(needle), `WOC mob-scan counter projection is missing: ${needle}`);
}

for (const needle of [
  '%import("world/mob_scan_counters_state")',
  'counters.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC mob-scan counter test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m4_mob_scan_counters_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-mob-scan-counters-state-tests' &&
    testProject.entry === 'world/mob_scan_counters_state_test_main',
  'mob-scan counter test project contract drifted',
);

process.stdout.write(`checked M4 mob-scan counters: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
