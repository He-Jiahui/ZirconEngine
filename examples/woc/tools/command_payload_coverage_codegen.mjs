import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const commandCatalogPath = join(projectRoot, 'reference', 'current-head', 'command_catalog.json');
const sourcePayloadCatalogPath = join(projectRoot, 'reference', 'current-head', 'command_payload_catalog.json');
const contractPath = join(projectRoot, 'contracts', 'command_payloads.json');
const outputPath = join(projectRoot, 'reference', 'current-head', 'command_payload_coverage.json');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const commandCatalog = readJson(commandCatalogPath);
  const sourcePayloadCatalog = readJson(sourcePayloadCatalogPath);
  const contracts = readJson(contractPath);
  validateInputs(commandCatalog, sourcePayloadCatalog, contracts);

  const contractById = new Map(contracts.entries.map((entry) => [entry.id, entry]));
  const entries = commandCatalog.entries.map((command) => {
    const source = sourcePayloadCatalog.entries[command.index];
    const contract = contractById.get(command.index);
    const coverage = contract ? 'typed_contract' : command.kind === 'dispatch_only'
      ? 'unmapped_dispatch'
      : 'source_shape_only';
    return {
      id: command.index,
      name: command.name,
      command_kind: command.kind,
      facet: command.facet,
      source_status: source.status,
      source_shapes: source.client_sends.map((site) => ({
        method: site.method,
        fields: site.fields.map((field) => ({
          name: field.name,
          server_type: field.server_type,
        })),
      })),
      transport_coverage: coverage,
      descriptor: contract ? descriptor(contract) : null,
    };
  });
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/command_payload_coverage_codegen.mjs',
    command_catalog_sha256: commandCatalogSha(commandCatalog.entries),
    source_payload_catalog_sha256: sha256(readFileSync(sourcePayloadCatalogPath)),
    command_payload_schema_sha256: payloadSchemaSha(contracts.entries),
    totals: {
      commands: entries.length,
      typed_contract_commands: entries.filter((entry) => entry.transport_coverage === 'typed_contract').length,
      typed_contract_client_send_commands: entries.filter(
        (entry) => entry.transport_coverage === 'typed_contract' && entry.command_kind === 'client_send',
      ).length,
      typed_contract_dispatch_only_commands: entries.filter(
        (entry) => entry.transport_coverage === 'typed_contract' && entry.command_kind === 'dispatch_only',
      ).length,
      source_shape_only_commands: entries.filter((entry) => entry.transport_coverage === 'source_shape_only').length,
      unmapped_dispatch_commands: entries.filter((entry) => entry.transport_coverage === 'unmapped_dispatch').length,
    },
    entries,
  };
  validateCoverage(document, contracts.entries);
  const rendered = `${JSON.stringify(document, null, 2)}\n`;
  if (checkOnly) {
    invariant(existsSync(outputPath), 'command payload coverage is missing; run npm run generate:command-payload-coverage');
    invariant(readFileSync(outputPath, 'utf8') === rendered, 'command payload coverage is stale; run npm run generate:command-payload-coverage');
  } else {
    writeFileSync(outputPath, rendered, 'utf8');
  }
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} ${document.totals.typed_contract_commands}/165 typed command payload contracts\n`,
  );
}

function validateInputs(commandCatalog, sourcePayloadCatalog, contracts) {
  invariant(commandCatalog.source_commit === SOURCE_COMMIT, 'command catalog source commit drifted');
  invariant(sourcePayloadCatalog.source_commit === SOURCE_COMMIT, 'source payload catalog source commit drifted');
  invariant(contracts.source_commit === SOURCE_COMMIT, 'command payload schema source commit drifted');
  invariant(commandCatalog.entries.length === 165, 'command catalog count drifted');
  invariant(sourcePayloadCatalog.entries.length === 165, 'source payload catalog count drifted');
  invariant(contracts.schema_version === 60, 'command payload schema version drifted');
  const catalogSha = commandCatalogSha(commandCatalog.entries);
  invariant(sourcePayloadCatalog.command_catalog_sha256 === catalogSha, 'source payload catalog command fingerprint drifted');
  invariant(contracts.command_catalog_sha256 === catalogSha, 'command payload schema command fingerprint drifted');
  for (const [index, command] of commandCatalog.entries.entries()) {
    const source = sourcePayloadCatalog.entries[index];
    invariant(command.index === index, `command ${index} is not contiguous`);
    invariant(source?.id === index && source.name === command.name, `source payload command ${index} drifted`);
    invariant(source.kind === command.kind, `source payload kind drifted for ${command.name}`);
  }
}

function validateCoverage(document, contracts) {
  invariant(document.totals.commands === 165, 'coverage command count drifted');
  invariant(document.totals.typed_contract_commands === contracts.length, 'typed coverage count drifted');
  invariant(document.totals.typed_contract_commands === 157, 'typed command coverage drifted');
  invariant(document.totals.typed_contract_client_send_commands === 156, 'typed client-send coverage drifted');
  invariant(document.totals.typed_contract_dispatch_only_commands === 1, 'typed dispatch coverage drifted');
  invariant(document.totals.source_shape_only_commands === 0, 'source-only coverage drifted');
  invariant(document.totals.unmapped_dispatch_commands === 8, 'unmapped dispatch coverage drifted');
  for (const entry of document.entries) {
    if (entry.transport_coverage === 'typed_contract') {
      invariant(entry.descriptor !== null, `typed command ${entry.name} has no descriptor`);
      continue;
    }
    invariant(entry.descriptor === null, `untyped command ${entry.name} has an unexpected descriptor`);
    if (entry.transport_coverage === 'source_shape_only') {
      invariant(entry.command_kind === 'client_send', `source-only command ${entry.name} is not client-sendable`);
      invariant(entry.source_status === 'observed_client_send', `source-only command ${entry.name} lacks source evidence`);
      continue;
    }
    invariant(entry.transport_coverage === 'unmapped_dispatch', `unknown coverage state for ${entry.name}`);
    invariant(entry.command_kind === 'dispatch_only', `unmapped dispatch ${entry.name} is client-sendable`);
    invariant(entry.source_status === 'dispatch_only', `unmapped dispatch ${entry.name} has source-send evidence`);
  }
}

function descriptor(entry) {
  return {
    kind: entry.kind,
    min_byte_length: entry.min_byte_length,
    max_byte_length: entry.max_byte_length,
    max_utf8_bytes: entry.max_utf8_bytes ?? null,
    max_utf16_code_units: entry.max_utf16_code_units ?? null,
    max_collection_entries: entry.max_collection_entries ?? null,
    encoding: entry.encoding,
    source_shape: entry.source_shape,
  };
}

function commandCatalogSha(entries) {
  return sha256(
    Buffer.from(
      entries.map((entry) => `${entry.index}\0${entry.name}\0${entry.kind}\0${entry.facet ?? ''}\n`).join(''),
      'utf8',
    ),
  );
}

function payloadSchemaSha(entries) {
  const source = [...entries]
    .sort((left, right) => left.id - right.id)
    .map(
      (entry) =>
        `${entry.id}\0${entry.name}\0${entry.kind}\0${entry.min_byte_length}\0` +
        `${entry.max_byte_length}\0${entry.max_utf8_bytes ?? ''}\0` +
        `${entry.max_utf16_code_units ?? ''}\0${entry.max_collection_entries ?? ''}\0` +
        `${entry.encoding}\n`,
    )
    .join('');
  return sha256(Buffer.from(source, 'utf8'));
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
