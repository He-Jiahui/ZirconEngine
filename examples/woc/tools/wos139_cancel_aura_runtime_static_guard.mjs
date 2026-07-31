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

const sourceCancel = source('src/sim/combat/aura_cancel.ts');
requireText(sourceCancel, /isCancelableAura[\s\S]*?!isDebuffAura/,
  'source cancel-aura helpful-only predicate drifted');
requireText(sourceCancel, /removeCancelableAura[\s\S]*?findIndex[\s\S]*?a\.id === auraId[\s\S]*?isCancelableAura/,
  'source cancel-aura first-match semantics drifted');
const sourceSim = source('src/sim/sim.ts');
requireText(sourceSim, /cancelAura\(auraId: string[\s\S]*?removeCancelableAura\(e\.auras, auraId\)/,
  'source authoritative cancel-aura reducer drifted');

const payloads = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(payloads, /pub cancelAuraCommandId\(required: bool\): uint[\s\S]*?return <uint>3/,
  'cancel_aura command identity is missing');
const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /var cancelAuraCommand = payloads\.cancelAuraCommandId\(true\)/,
  'cancel_aura command reducer binding is missing');
requireText(world, /else if \(commandId == cancelAuraCommand\)[\s\S]*?applyOfflineCancelAuraCommand/,
  'cancel_aura dispatch is missing');
requireText(world, /applyOfflineCancelAuraCommand[\s\S]*?formAbilityCodeForExactPayload[\s\S]*?clearActiveForm[\s\S]*?clearOfflineProwl/,
  'cancel_aura form cancellation is missing');
requireText(world, /applyOfflineCancelAuraCommand[\s\S]*?m4AbilityCodeFromPayload[\s\S]*?offlineCancelableMotionAuraKind[\s\S]*?removeMotionAuraAt/,
  'cancel_aura helpful motion-aura cancellation is missing');
requireText(world, /pub cancelAuraCommandStateTest\(\): int[\s\S]*?appendCancelAuraCommandForTest[\s\S]*?cancelDebuff[\s\S]*?cancelProwl[\s\S]*?cancelForm/,
  'cancel_aura state regression coverage is missing');
requireText(world, /if \(cancelAuraCommandStateTest\(\) != 1\) \{[\s\S]*?return -133;/,
  'world selfTest must execute cancel_aura coverage');

process.stdout.write(`WOS139 cancel_aura static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
