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
  /reportTelemetry\(kind: string, data: Record<string, number>\)[\s\S]*?cmd: 'telemetry', kind, \.\.\.data/,
  'source telemetry client payload shape drifted');
const sourceServer = source('server/game.ts');
requireText(sourceServer,
  /case 'telemetry':\s*break;/,
  'source telemetry command is no longer an accepted no-op');

const contract = JSON.parse(read('contracts', 'command_payloads.json'));
if (contract.schema_version !== 51) {
  throw new Error('WOS150 command payload schema must be 44');
}
const telemetry = contract.entries.find((entry) => entry.id === 125 && entry.name === 'telemetry');
if (!telemetry || telemetry.kind !== 'telemetry_numeric_fields' ||
    telemetry.min_byte_length !== 6 || telemetry.max_byte_length !== 65536 ||
    telemetry.max_utf8_bytes !== 256 || telemetry.max_collection_entries !== 256 ||
    telemetry.encoding !== 'u32_le_utf8_kind+u16_le_field_count+repeated_u32_le_utf8_key_f64_le_value') {
  throw new Error('telemetry typed payload contract is missing or non-canonical');
}

const generated = read('scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
requireText(generated,
  /telemetryCommandId\(required: bool\): uint[\s\S]*?return <uint>125/,
  'generated telemetry command id is missing');
requireText(generated,
  /payloadKind\(<uint>125, 1\) == 49[\s\S]*?payloadMinLength\(<uint>125, true\) == 6[\s\S]*?payloadMaxLength\(<uint>125, true\) == 65536/,
  'generated telemetry payload kind or bounds are missing');

const generatedRust = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
requireText(generatedRust,
  /name: "telemetry"[\s\S]*?kind: CommandPayloadKind::TelemetryNumericFields[\s\S]*?max_collection_entries: 256/,
  'generated Rust telemetry descriptor is missing the collection bound');

const protocol = read('native', 'crates', 'woc_protocol', 'src', 'telemetry_payload.rs');
requireText(protocol,
  /pub struct TelemetryPayload[\s\S]*?pub kind: String[\s\S]*?pub data: BTreeMap<String, f64>/,
  'native telemetry payload type is missing');
requireText(protocol,
  /validate_telemetry_payload[\s\S]*?decode_payload/,
  'native telemetry payload validator is missing');
requireText(protocol,
  /validate_field_count\([^\n]*descriptor\.max_collection_entries/,
  'native telemetry payload does not consume its generated collection bound');

const intent = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');
requireText(intent,
  /ReportTelemetry \{[\s\S]*?kind: String[\s\S]*?data: BTreeMap<String, f64>[\s\S]*?TelemetryPayload/,
  'native client telemetry intent mapping is missing');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world,
  /applyTelemetryNoopCommand[\s\S]*?fieldCount[\s\S]*?cursor[\s\S]*?payloadLength/,
  'authoritative telemetry structural no-op is missing');
requireText(world,
  /commandId == telemetryCommand[\s\S]*?applyTelemetryNoopCommand/,
  'authoritative command routing does not accept telemetry');
requireText(world,
  /pub telemetryNoopStateTest\(\): int[\s\S]*?encodeState[\s\S]*?applyCommands[\s\S]*?encodeState/,
  'telemetry authoritative no-mutation regression is missing');
requireText(world,
  /if \(telemetryNoopStateTest\(\) != 1\) \{[\s\S]*?return -144;/,
  'world selfTest must execute WOS150 coverage');

const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
if (coverage.entries.find((entry) => entry.id === 125)?.transport_coverage !== 'typed_contract') {
  throw new Error('telemetry coverage projection is not typed_contract');
}

process.stdout.write(`WOS150 telemetry no-op static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
