import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ONLINE_PATH = 'src/net/online.ts';
const SERVER_PATH = 'server/game.ts';
const PET_COMMANDS_PATH = 'src/sim/pet/pet_commands.ts';
const PET_AI_PATH = 'src/sim/pet/pet_ai.ts';
const PET_DATA_PATH = 'src/sim/content/mage_pets.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'pet_water_jet_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'pet_water_jet_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  const commandCatalog = readJson(join(referenceRoot, 'command_catalog.json'));
  const payloadCatalog = readJson(join(referenceRoot, 'command_payload_catalog.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before Pet Water Jet contracts');
  invariant(commandCatalog.source_commit === SOURCE_COMMIT && payloadCatalog.source_commit === SOURCE_COMMIT,
    'Pet Water Jet command references are not pinned to the current source');

  const blobs = Object.fromEntries([ONLINE_PATH, SERVER_PATH, PET_COMMANDS_PATH, PET_AI_PATH, PET_DATA_PATH]
    .map((path) => [path, sourceBlob(path)]));
  const online = blobs[ONLINE_PATH].toString('utf8');
  const server = blobs[SERVER_PATH].toString('utf8');
  const commands = [
    { name: 'pet_water_jet', method: 'petWaterJet', fields: [] },
    { name: 'pet_auto_water_jet', method: 'setPetAutoWaterJet', fields: [{ name: 'enabled', type: 'boolean' }] },
  ].map((definition) => bindDefinition(definition, commandCatalog, payloadCatalog));
  invariant(online.includes("this.cmd({ cmd: 'pet_water_jet' });"),
    'pet_water_jet client envelope is no longer empty');
  invariant(online.includes("this.cmd({ cmd: 'pet_auto_water_jet', enabled });"),
    'pet_auto_water_jet client envelope no longer forwards enabled directly');
  invariant(server.includes("case 'pet_water_jet':") && server.includes('sim.petWaterJet(pid);'),
    'pet_water_jet server handler drifted');
  invariant(server.includes("case 'pet_auto_water_jet':") &&
    server.includes("typeof msg.enabled === 'boolean'") &&
    server.includes('sim.setPetAutoWaterJet(msg.enabled, pid);'),
  'pet_auto_water_jet server handler drifted');
  invariant(blobs[PET_COMMANDS_PATH].toString('utf8').includes('export function petWaterJet') &&
    blobs[PET_COMMANDS_PATH].toString('utf8').includes('export function setPetAutoWaterJet'),
  'Pet Water Jet command reducer drifted');
  invariant(blobs[PET_AI_PATH].toString('utf8').includes('function updateWaterJetChannel') &&
    blobs[PET_AI_PATH].toString('utf8').includes('export function startWaterJet'),
  'Pet Water Jet channel owner drifted');
  const data = blobs[PET_DATA_PATH].toString('utf8');
  const jet = parseJet(data);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/pet_water_jet_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    commands,
    water_elemental: { range: 25, jet },
    source_semantics: {
      manual: 'silent no-op unless the live pet has a jet, is not dead/casting/cooling down, and the owner target is hostile and in range',
      auto: 'enabled only for a pet with a jet; pet AI starts the same channel once a valid target is in reach',
      channel: 'the channel locks pet target and shared petTauntTimer cooldown; a broken connection removes only this pet source water_jet and water_jet_slow auras',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'Pet Water Jet JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Pet Water Jet Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Pet Water Jet contract for ${SOURCE_COMMIT}\n`);
}

function parseJet(source) {
  const match = source.match(/range:\s*(\d+)[\s\S]*?jet:\s*\{\s*total:\s*(\d+),\s*duration:\s*(\d+),\s*interval:\s*(\d+),\s*slow:\s*([\d.]+),\s*cooldown:\s*(\d+)\s*\}/);
  invariant(match, 'water elemental Pet Water Jet data is missing or no longer a literal contract');
  return {
    total: Number(match[2]), duration_seconds: Number(match[3]), interval_seconds: Number(match[4]),
    slow_multiplier: Number(match[5]), cooldown_seconds: Number(match[6]), range: Number(match[1]),
  };
}

function bindDefinition(definition, commandCatalog, payloadCatalog) {
  const command = commandCatalog.entries.find((entry) => entry.name === definition.name);
  invariant(command?.kind === 'client_send', `${definition.name} is not a client-send command`);
  const payload = payloadCatalog.entries.find((entry) => entry.id === command.index);
  invariant(payload?.name === definition.name, `${definition.name} source payload row is missing`);
  const matchingSite = payload.client_sends.find((site) =>
    site.method === definition.method &&
    JSON.stringify(site.fields.map((field) => ({ name: field.name, type: field.server_type?.type }))) ===
      JSON.stringify(definition.fields));
  invariant(matchingSite, `${definition.name} source method or field type drifted`);
  return { id: command.index, name: command.name, source_shape: { method: definition.method, fields: definition.fields } };
}

function renderZr(document) {
  const byName = new Map(document.commands.map((command) => [command.name, command]));
  const manual = required(byName, 'pet_water_jet');
  const auto = required(byName, 'pet_auto_water_jet');
  const jet = document.water_elemental.jet;
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub manualCommandId(required: bool): uint { return required ? <uint>${manual.id} : <uint>0; }\n` +
    `pub autoCommandId(required: bool): uint { return required ? <uint>${auto.id} : <uint>0; }\n` +
    `pub range(required: bool): float { return required ? ${jet.range}.0 : 0.0; }\n` +
    `pub totalDamage(required: bool): int { return required ? ${jet.total} : 0; }\n` +
    `pub durationSeconds(required: bool): float { return required ? ${jet.duration_seconds}.0 : 0.0; }\n` +
    `pub intervalSeconds(required: bool): float { return required ? ${jet.interval_seconds}.0 : 0.0; }\n` +
    `pub slowMultiplier(required: bool): float { return required ? ${jet.slow_multiplier} : 0.0; }\n` +
    `pub cooldownSeconds(required: bool): float { return required ? ${jet.cooldown_seconds}.0 : 0.0; }\n` +
    `pub perTickDamage(required: bool): int { return required ? ${Math.round(jet.total / (jet.duration_seconds / jet.interval_seconds))} : 0; }\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:pet-water-jet-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:pet-water-jet-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function required(entries, name) { const entry = entries.get(name); invariant(entry, `missing Pet Water Jet command ${name}`); return entry; }
function invariant(condition, message) { if (!condition) throw new Error(message); }
