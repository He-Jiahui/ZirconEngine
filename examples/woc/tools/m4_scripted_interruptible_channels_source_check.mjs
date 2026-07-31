import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/mob/healer_channel.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'combat', 'scripted_interruptible_channels.zr'),
  'utf8',
);
const testMain = readFileSync(
  resolve(
    wocRoot,
    'scripts',
    'woc_game',
    'src',
    'combat',
    'scripted_interruptible_channels_test_main.zr',
  ),
  'utf8',
);
const testProject = JSON.parse(
  readFileSync(
    resolve(wocRoot, 'scripts', 'woc_game', 'woc_m4_scripted_interruptible_channels_tests.zrp'),
    'utf8',
  ),
);

for (const needle of [
  "export const NYTHRAXIS_SPIRIT_MENDING_CAST_ID = 'nythraxis_spirit_mending';",
  'export const SCRIPTED_INTERRUPTIBLE_CHANNELS:',
  "[NYTHRAXIS_SPIRIT_MENDING_CAST_ID]: { school: 'shadow' },",
]) {
  invariant(source.includes(needle), `source scripted-channel rule drifted: ${needle}`);
}

for (const needle of [
  'return "nythraxis_spirit_mending";',
  'if (castId == nythraxisSpiritMendingCastId(true)) {',
  'return "shadow";',
  'return "";',
  'return scriptedInterruptibleChannelSchool(castId, required) != "";',
]) {
  invariant(projection.includes(needle), `WOC scripted-channel projection is missing: ${needle}`);
}

for (const needle of [
  '%import("combat/scripted_interruptible_channels")',
  'channels.contractTest()',
]) {
  invariant(testMain.includes(needle), `WOC scripted-channel test is missing: ${needle}`);
}

invariant(
  testProject.name === 'woc_m4_scripted_interruptible_channels_tests' &&
    testProject.source === 'src' &&
    testProject.binary === 'bin-m4-scripted-interruptible-channels-tests' &&
    testProject.entry === 'combat/scripted_interruptible_channels_test_main',
  'scripted-channel test project contract drifted',
);

process.stdout.write(`checked M4 scripted interruptible channels: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
