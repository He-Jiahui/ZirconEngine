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

const sourceOnline = source('src/net/online.ts');
requireText(sourceOnline,
  /acceptLinkedQuest\(questId: string, fromPid: number\)[\s\S]*?cmd: 'qlinkaccept'[\s\S]*?quest: questId[\s\S]*?from: fromPid/,
  'source linked-quest client payload shape drifted');

const sourceQuests = source('src/sim/quests/quest_commands.ts');
requireText(sourceQuests,
  /acceptLinkedQuest[\s\S]*?quest\.shareable === false[\s\S]*?myParty\.id !== sharerParty\.id[\s\S]*?questState\(ctx, questId, meta\.entityId\) !== 'available'[\s\S]*?finalizeQuestAccept/,
  'source linked-quest party, shareability, or availability semantics drifted');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 38) {
  throw new Error('WOS148 command payload schema must be 38');
}
const linked = contract.entries.find((entry) => entry.id === 19 && entry.name === 'qlinkaccept');
if (!linked || linked.kind !== 'linked_quest_acceptance' ||
    linked.min_byte_length !== 12 || linked.max_byte_length !== 268 ||
    linked.encoding !== 'u32_le_utf8_quest_id+f64_le_sharer_pid') {
  throw new Error('qlinkaccept typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated,
  /linkedQuestAcceptCommandId\(required: bool\): uint[\s\S]*?return <uint>19/,
  'generated linked-quest command id is missing');
requireText(generated,
  /payloadKind\(<uint>19, 1\) == 46[\s\S]*?payloadMinLength\(<uint>19, true\) == 12[\s\S]*?payloadMaxLength\(<uint>19, true\) == 268/,
  'generated linked-quest payload kind or bounds are missing');

const protocol = read('native', 'crates', 'woc_protocol', 'src', 'linked_quest_payload.rs');
requireText(protocol,
  /pub struct LinkedQuestAcceptancePayload[\s\S]*?pub quest_id: String[\s\S]*?pub sharer_pid: f64/,
  'native linked-quest payload type is missing');
requireText(protocol,
  /validate_linked_quest_acceptance_payload[\s\S]*?validate_sharer_pid/,
  'native linked-quest payload validator is missing');

const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /AcceptLinkedQuest \{[\s\S]*?quest_id: String[\s\S]*?sharer_pid: f64[\s\S]*?LinkedQuestAcceptancePayload/,
  'native client linked-quest intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world,
  /applyOfflineLinkedQuestAcceptCommand[\s\S]*?readF64LeAt[\s\S]*?partyIdForIndex[\s\S]*?setOfflineQuestState/,
  'authoritative linked-quest reducer is missing');
requireText(world,
  /commandId == linkedQuestAcceptCommand[\s\S]*?applyOfflineLinkedQuestAcceptCommand/,
  'authoritative command routing does not reach linked-quest acceptance');
requireText(world,
  /pub linkedQuestAcceptanceStateTest\(\): int[\s\S]*?applyCommands[\s\S]*?offlineQuestStateFor/,
  'linked-quest authoritative regression is missing');
requireText(world,
  /if \(linkedQuestAcceptanceStateTest\(\) != 1\) \{[\s\S]*?return -142;/,
  'world selfTest must execute WOS148 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
const coverageEntry = coverage.entries.find((entry) => entry.id === 19);
if (coverageEntry?.transport_coverage !== 'typed_contract') {
  throw new Error('qlinkaccept coverage projection is not typed_contract');
}

process.stdout.write(`WOS148 linked quest acceptance static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
