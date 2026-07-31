import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ONLINE_PATH = 'src/net/online.ts';
const SERVER_PATH = 'server/game.ts';
const EMPOWERED_PATH = 'src/sim/combat/casting_lifecycle.ts';
const RESURRECTION_PATH = 'src/sim/combat/resurrection_offer.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'combat_command_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'combat_command_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  const commandCatalog = readJson(join(referenceRoot, 'command_catalog.json'));
  const payloadCatalog = readJson(join(referenceRoot, 'command_payload_catalog.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before combat command contracts');
  invariant(commandCatalog.source_commit === SOURCE_COMMIT,
    'combat command catalog is not pinned to the current source');
  invariant(payloadCatalog.source_commit === SOURCE_COMMIT,
    'combat command payload catalog is not pinned to the current source');

  const blobs = Object.fromEntries([ONLINE_PATH, SERVER_PATH, EMPOWERED_PATH, RESURRECTION_PATH]
    .map((path) => [path, sourceBlob(path)]));
  const online = blobs[ONLINE_PATH].toString('utf8');
  const server = blobs[SERVER_PATH].toString('utf8');
  const empowered = blobs[EMPOWERED_PATH].toString('utf8');
  const resurrection = blobs[RESURRECTION_PATH].toString('utf8');
  const definitions = [
    {
      name: 'releaseEmpowered',
      method: 'releaseEmpoweredAbility',
      fields: [{ name: 'ability', type: 'string' }],
      handler: 'sim.releaseEmpoweredAbility(msg.ability, pid);',
    },
    {
      name: 'resurrect_respond',
      method: 'respondToResurrection',
      fields: [{ name: 'accept', type: 'boolean' }],
      handler: 'sim.respondToResurrection(msg.accept, pid);',
    },
  ].map((definition) => bindDefinition(definition, commandCatalog, payloadCatalog));

  invariant(online.includes("this.cmd({ cmd: 'releaseEmpowered', ability: abilityId });"),
    'releaseEmpowered client payload no longer forwards abilityId directly');
  invariant(online.includes("this.cmd({ cmd: 'resurrect_respond', accept });"),
    'resurrect_respond client payload no longer forwards accept directly');
  for (const command of definitions) {
    invariant(server.includes(`case '${command.name}':`) && server.includes(command.handler),
      `${command.name} server handler no longer validates and forwards the source field`);
  }
  invariant(empowered.includes('export function releaseEmpoweredAbility') &&
    empowered.includes('empoweredStageForProgress(') &&
    empowered.includes('empoweredCastProgress(p.castTotal, p.castRemaining)'),
  'empowered release is no longer derived from authoritative cast progress');
  invariant(resurrection.includes('export const RESURRECTION_OFFER_SECONDS = 30;') &&
    resurrection.includes('if (!accept || ctx.time >= offer.expiresAt || !r.e.dead) return;'),
  'resurrection response expiry or liveness semantics drifted');

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/combat_command_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    commands: definitions,
    source_semantics: {
      releaseEmpowered: 'ability is a string identity; authoritative stage is derived only from the active Sim cast clock',
      resurrect_respond: 'accept is a boolean; target-owned offers expire inclusively at 30 seconds and resolve at a live caster or fallback position',
    },
    zrvm_backend_status: {
      state: 'blocked',
      reason: 'Plugin08 must expose lossless transactional CommandValue transport before string and boolean envelopes may cross the ZrVM boundary',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'combat command JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'combat command Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} combat command contract for ${SOURCE_COMMIT}\n`);
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
  return {
    id: command.index,
    name: command.name,
    source_shape: {
      method: definition.method,
      fields: definition.fields,
    },
  };
}

function renderZr(document) {
  const byName = new Map(document.commands.map((command) => [command.name, command]));
  const release = required(byName, 'releaseEmpowered');
  const resurrect = required(byName, 'resurrect_respond');
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `// Kind 1 is a CommandValue string; kind 2 is a CommandValue boolean.\n` +
    `// Neither payload is byte-encoded here: Plugin08 owns lossless transport.\n` +
    `pub releaseEmpoweredCommandId(required: bool): uint { return required ? <uint>${release.id} : <uint>0; }\n` +
    `pub resurrectRespondCommandId(required: bool): uint { return required ? <uint>${resurrect.id} : <uint>0; }\n` +
    `pub payloadKind(id: uint, required: bool): int {\n` +
    `    if (!required) return 0;\n` +
    `    if (id == <uint>${release.id}) return 1;\n` +
    `    if (id == <uint>${resurrect.id}) return 2;\n` +
    `    return 0;\n` +
    `}\n` +
    `pub requiresLosslessCommandValue(required: bool): bool { return required; }\n`;
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer', maxBuffer: 64 * 1024 * 1024,
  });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:combat-command-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:combat-command-contract`);
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function render(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function required(entries, name) {
  const entry = entries.get(name);
  invariant(entry, `missing combat command ${name}`);
  return entry;
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
