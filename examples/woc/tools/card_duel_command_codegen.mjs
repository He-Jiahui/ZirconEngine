import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ONLINE_PATH = 'src/net/online.ts';
const SERVER_PATH = 'server/game.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'card_duel_command_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'card_duel_command_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  const commandCatalog = readJson(join(referenceRoot, 'command_catalog.json'));
  const payloadCatalog = readJson(join(referenceRoot, 'command_payload_catalog.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before Card Duel command contracts');
  invariant(commandCatalog.source_commit === SOURCE_COMMIT,
    'Card Duel command catalog is not pinned to the current source');
  invariant(payloadCatalog.source_commit === SOURCE_COMMIT,
    'Card Duel payload catalog is not pinned to the current source');

  const onlineBytes = sourceBlob(ONLINE_PATH);
  const serverBytes = sourceBlob(SERVER_PATH);
  const online = onlineBytes.toString('utf8');
  const server = serverBytes.toString('utf8');
  const definitions = [
    { name: 'card_queue_join', method: 'joinCardDuelQueue', fields: [], kind: 'empty' },
    { name: 'card_queue_leave', method: 'leaveCardDuelQueue', fields: [], kind: 'empty' },
    { name: 'play_card', method: 'playCardInDuel', fields: ['value'], kind: 'i32_le' },
    { name: 'card_forfeit', method: 'forfeitCardDuel', fields: [], kind: 'empty' },
  ].map((definition) => bindDefinition(definition, commandCatalog, payloadCatalog));

  invariant(online.includes("this.cmd({ cmd: 'play_card', value: cardValue });"),
    'Card Duel client play payload no longer sends cardValue directly');
  invariant(server.includes("case 'play_card':") &&
    server.includes('typeof msg.value === \'number\' && Number.isInteger(msg.value)') &&
    server.includes('sim.playCardInDuel(msg.value, pid);'),
  'Card Duel server play validation no longer accepts an integer for Sim validation');
  for (const command of definitions.filter((definition) => definition.kind === 'empty')) {
    invariant(online.includes(`this.cmd({ cmd: '${command.name}' });`),
      `${command.name} client payload is no longer empty`);
    invariant(server.includes(`case '${command.name}':`),
      `${command.name} server handler is missing`);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/card_duel_command_codegen.mjs',
    source_blobs: {
      [ONLINE_PATH]: sha256(onlineBytes),
      [SERVER_PATH]: sha256(serverBytes),
    },
    commands: definitions,
    payload_encoding: {
      empty: 'empty',
      i32_le: 'i32_le',
    },
    source_semantics: {
      play_card: 'server accepts any JavaScript integer and lets the simulation decide whether the player holds that card',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'Card Duel command JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Card Duel command Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Card Duel command contract for ${SOURCE_COMMIT}\n`);
}

function bindDefinition(definition, commandCatalog, payloadCatalog) {
  const command = commandCatalog.entries.find((entry) => entry.name === definition.name);
  invariant(command?.kind === 'client_send', `${definition.name} is not a client-send command`);
  const payload = payloadCatalog.entries.find((entry) => entry.id === command.index);
  invariant(payload?.name === definition.name, `${definition.name} source payload row is missing`);
  const matchingSite = payload.client_sends.find((site) =>
    site.method === definition.method &&
    JSON.stringify(site.fields.map((field) => field.name)) === JSON.stringify(definition.fields));
  invariant(matchingSite, `${definition.name} source method or fields drifted`);
  return {
    id: command.index,
    name: command.name,
    source_shape: {
      method: definition.method,
      fields: definition.fields,
    },
    kind: definition.kind,
    min_byte_length: definition.kind === 'empty' ? 0 : 4,
    max_byte_length: definition.kind === 'empty' ? 0 : 4,
  };
}

function renderZr(document) {
  const byName = new Map(document.commands.map((command) => [command.name, command]));
  const queueJoin = required(byName, 'card_queue_join');
  const queueLeave = required(byName, 'card_queue_leave');
  const playCard = required(byName, 'play_card');
  const forfeit = required(byName, 'card_forfeit');
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub queueJoinCommandId(required: bool): uint { return required ? <uint>${queueJoin.id} : <uint>0; }\n` +
    `pub queueLeaveCommandId(required: bool): uint { return required ? <uint>${queueLeave.id} : <uint>0; }\n` +
    `pub playCardCommandId(required: bool): uint { return required ? <uint>${playCard.id} : <uint>0; }\n` +
    `pub forfeitCommandId(required: bool): uint { return required ? <uint>${forfeit.id} : <uint>0; }\n` +
    `pub playCardPayloadBytes(required: bool): int { return required ? 4 : 0; }\n` +
    `// kind: 1 empty, 2 i32_le.\n` +
    `pub payloadKind(id: uint, required: bool): int {\n` +
    `    if (!required) return 0;\n` +
    `    if (id == <uint>${queueJoin.id} || id == <uint>${queueLeave.id} || id == <uint>${forfeit.id}) return 1;\n` +
    `    if (id == <uint>${playCard.id}) return 2;\n` +
    `    return 0;\n` +
    `}\n`;
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer', maxBuffer: 64 * 1024 * 1024,
  });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:card-duel-command-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:card-duel-command-contract`);
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
  invariant(entry, `missing Card Duel command ${name}`);
  return entry;
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
