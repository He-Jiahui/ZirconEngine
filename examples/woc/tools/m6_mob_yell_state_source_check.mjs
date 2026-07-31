import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/mob/yells.ts');
const types = gitShow('src/sim/types.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'mob_yell_state.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'social', 'mob_yell_state_test_main.zr'),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(resolve(wocRoot, 'scripts', 'woc_game', 'woc_m6_mob_yell_state_tests.zrp'), 'utf8'),
);

for (const needle of [
  "type: 'chat' as const",
  'fromPid: mob.id,',
  'from: mob.name,',
  'channel: \'yell\' as const,',
  'entityId: mob.id,',
  'for (const meta of ctx.players.values()) {',
  'if (!p || dist2d(p.pos, mob.pos) > range) continue;',
  'ctx.emit({ ...event, pid: meta.entityId });',
]) {
  invariant(source.includes(needle), `source mob-yell rule drifted: ${needle}`);
}
invariant(types.includes('export const YELL_RANGE = 100;'), 'source yell range drifted');

for (const needle of [
  'pub defaultYellRange(): float {',
  'return 100.0;',
  'pub playerReceivesMobYell(',
  'return dx * dx + dz * dz <= range * range;',
  'state.eventFromPids.add(mobId);',
  'state.eventFromNames.add(mobName);',
  'state.eventTexts.add(text);',
  'state.eventChannels.add("yell");',
  'state.eventEntityIds.add(mobId);',
  'state.eventRecipientIds.add(recipientId);',
  'while (index < state.playerIds.length) {',
]) {
  invariant(projection.includes(needle), `WOC mob-yell projection is missing: ${needle}`);
}

for (const needle of [
  '%import("social/mob_yell_state")',
  'mobYell.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC mob-yell test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m6_mob_yell_state_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m6-mob-yell-state-tests' &&
    testProject.entry === 'social/mob_yell_state_test_main',
  'mob-yell test project contract drifted',
);

process.stdout.write(`checked M6 mob-yell source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
