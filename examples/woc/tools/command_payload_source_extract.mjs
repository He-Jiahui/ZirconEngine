import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

const HISTORICAL_SOURCE_COMMIT = '7c10f280eec380e9877e66ce16333089e171fe42';
const CURRENT_SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_COMMIT = readOption('--commit') ?? CURRENT_SOURCE_COMMIT;
const ONLINE_PATH = 'src/net/online.ts';
const SERVER_PATH = 'server/game.ts';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceDirectory = resolve(readOption('--reference') ?? join(projectRoot, 'reference', 'current-head'));
const commandCatalogPath = join(referenceDirectory, 'command_catalog.json');
const outputPath = join(referenceDirectory, 'command_payload_catalog.json');
const checkOnly = process.argv.includes('--check');
const rebaseline = process.argv.includes('--rebaseline');
const historical = process.argv.includes('--historical');

main();

function main() {
  if (SOURCE_COMMIT !== CURRENT_SOURCE_COMMIT && !historical) {
    throw new Error('a non-current source commit requires --historical');
  }
  const commandCatalog = JSON.parse(readFileSync(commandCatalogPath, 'utf8'));
  invariant(commandCatalog.source_commit === SOURCE_COMMIT, 'command catalog source commit drifted');
  invariant(Array.isArray(commandCatalog.entries), 'command catalog entries are missing');
  invariant(commandCatalog.entries.length > 0, 'command catalog is empty');
  if (!historical) invariant(commandCatalog.entries.length === 165, 'command catalog count drifted');

  const onlineBytes = sourceBlob(ONLINE_PATH);
  const serverBytes = sourceBlob(SERVER_PATH);
  const online = parse(ONLINE_PATH, onlineBytes.toString('utf8'));
  const server = parse(SERVER_PATH, serverBytes.toString('utf8'));
  const serverFieldTypes = clientMessageFieldTypes(server);
  const sites = clientCommandSites(online);
  const entries = buildEntries(commandCatalog.entries, sites, serverFieldTypes);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/command_payload_source_extract.mjs',
    source_blobs: {
      [ONLINE_PATH]: sha256(onlineBytes),
      [SERVER_PATH]: sha256(serverBytes),
    },
    command_catalog_sha256: commandCatalogSha(commandCatalog.entries),
    totals: {
      commands: entries.length,
      client_send_commands: entries.filter((entry) => entry.kind === 'client_send').length,
      dispatch_only_commands: entries.filter((entry) => entry.kind === 'dispatch_only').length,
      observed_client_commands: entries.filter((entry) => entry.client_sends.length > 0).length,
      client_send_sites: sites.length,
      declared_server_fields: Object.keys(serverFieldTypes).length,
    },
    entries,
  };
  const rendered = `${JSON.stringify(document, null, 2)}\n`;
  if (checkOnly) {
    invariant(existsSync(outputPath), 'command payload source catalog is missing; run npm run generate:command-payload-source');
    invariant(readFileSync(outputPath, 'utf8') === rendered, 'command payload source catalog is stale; run npm run generate:command-payload-source');
  } else {
    writeFileSync(outputPath, rendered, 'utf8');
  }
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} ${document.totals.client_send_sites} client command sites across ${document.totals.observed_client_commands}/${document.totals.commands} commands\n`,
  );
}

function sourceBlob(relativePath) {
  return Buffer.from(
    execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${relativePath}`], {
      encoding: 'buffer',
    }),
  );
}

function parse(path, text) {
  return ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
}

function clientMessageFieldTypes(sourceFile) {
  const declaration = sourceFile.statements.find(
    (statement) => ts.isTypeAliasDeclaration(statement) && statement.name.text === 'ClientMessage',
  );
  invariant(declaration, 'server ClientMessage type alias is missing');
  const literal = findTypeLiteral(declaration.type);
  invariant(literal, 'server ClientMessage type literal is missing');
  const fields = {};
  for (const member of literal.members) {
    if (!ts.isPropertySignature(member) || !member.name || !member.type) continue;
    const name = propertyName(member.name, sourceFile);
    fields[name] = {
      optional: Boolean(member.questionToken),
      type: normalize(member.type.getText(sourceFile)),
    };
  }
  invariant(Object.keys(fields).length > 0, 'server ClientMessage fields are empty');
  return fields;
}

function findTypeLiteral(node) {
  if (ts.isTypeLiteralNode(node)) return node;
  if (ts.isIntersectionTypeNode(node)) {
    for (const type of node.types) {
      const result = findTypeLiteral(type);
      if (result) return result;
    }
  }
  if (ts.isParenthesizedTypeNode(node)) return findTypeLiteral(node.type);
  return null;
}

function clientCommandSites(sourceFile) {
  const sites = [];
  visit(sourceFile, (node) => {
    if (!ts.isCallExpression(node) || !isTypedCommandSender(node.expression) || node.arguments.length !== 1) return;
    const payload = unwrap(node.arguments[0]);
    if (!ts.isObjectLiteralExpression(payload)) return;
    const command = objectStringProperty(payload, 'cmd');
    if (!command) return;
    const location = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
    sites.push({
      command,
      sender: node.expression.name.text,
      method: enclosingMethod(node, sourceFile),
      line: location.line + 1,
      fields: payload.properties
        .filter((property) => !isCommandProperty(property, sourceFile))
        .flatMap((property) => commandFields(property, sourceFile)),
    });
  });
  return sites.sort((left, right) => left.line - right.line);
}

function isTypedCommandSender(expression) {
  return (
    ts.isPropertyAccessExpression(expression) &&
    ts.isThis(expression.expression) &&
    (expression.name.text === 'cmd' || expression.name.text === 'cmdWithOutcome')
  );
}

function isCommandProperty(property, sourceFile) {
  return ts.isPropertyAssignment(property) && propertyName(property.name, sourceFile) === 'cmd';
}

function commandFields(property, sourceFile) {
  if (ts.isPropertyAssignment(property)) {
    return [{
      name: propertyName(property.name, sourceFile),
      expression: normalize(property.initializer.getText(sourceFile)),
    }];
  }
  if (ts.isShorthandPropertyAssignment(property)) {
    return [{ name: property.name.text, expression: property.name.text }];
  }
  if (ts.isSpreadAssignment(property)) {
    return conditionalObjectSpreadFields(property.expression, sourceFile) ?? [{
      name: null,
      expression: `...${normalize(property.expression.getText(sourceFile))}`,
    }];
  }
  throw new Error(`unsupported client command property: ${property.getText(sourceFile)}`);
}

function conditionalObjectSpreadFields(expression, sourceFile) {
  expression = unwrap(expression);
  if (!ts.isConditionalExpression(expression)) return null;
  const branches = [unwrap(expression.whenTrue), unwrap(expression.whenFalse)];
  if (!branches.every(ts.isObjectLiteralExpression)) return null;
  const fields = [];
  for (const branch of branches) {
    for (const property of branch.properties) {
      if (ts.isSpreadAssignment(property)) return null;
      fields.push(...commandFields(property, sourceFile));
    }
  }
  return [...new Map(fields.map((field) => [field.name, field])).values()];
}

function buildEntries(commands, sites, serverFieldTypes) {
  const known = new Map(commands.map((entry) => [entry.name, entry]));
  for (const site of sites) {
    const command = known.get(site.command);
    invariant(command, `online client sends unknown command ${site.command}`);
    invariant(command.kind === 'client_send', `online client sends dispatch-only command ${site.command}`);
  }
  return commands.map((command) => {
    const clientSends = sites
      .filter((site) => site.command === command.name)
      .map((site) => ({
        sender: site.sender,
        method: site.method,
        source_owner: `${ONLINE_PATH}:${site.line}`,
        fields: site.fields.map((field) => ({
          ...field,
          server_type: field.name ? serverFieldTypes[field.name] ?? null : null,
        })),
      }));
    const fieldNames = [...new Set(clientSends.flatMap((site) => site.fields.flatMap((field) => field.name ?? [])))].sort();
    return {
      id: command.index,
      name: command.name,
      kind: command.kind,
      facet: command.facet,
      source_owner: command.source_owner,
      client_sends: clientSends,
      client_field_names: fieldNames,
      status:
        command.kind === 'dispatch_only'
          ? 'dispatch_only'
          : clientSends.length > 0
            ? 'observed_client_send'
            : 'no_static_client_send',
    };
  });
}

function objectStringProperty(object, name) {
  const property = object.properties.find(
    (candidate) => ts.isPropertyAssignment(candidate) && propertyName(candidate.name, object.getSourceFile()) === name,
  );
  if (!property || !ts.isStringLiteralLike(unwrap(property.initializer))) return null;
  return unwrap(property.initializer).text;
}

function enclosingMethod(node, sourceFile) {
  let current = node.parent;
  while (current) {
    if (ts.isMethodDeclaration(current) && current.name) return propertyName(current.name, sourceFile);
    current = current.parent;
  }
  return null;
}

function propertyName(name, sourceFile) {
  if (ts.isIdentifier(name) || ts.isStringLiteralLike(name) || ts.isNumericLiteral(name)) return name.text;
  throw new Error(`unsupported property name: ${name.getText(sourceFile)}`);
}

function unwrap(node) {
  while (ts.isParenthesizedExpression(node) || ts.isAsExpression(node) || ts.isTypeAssertionExpression(node)) {
    node = node.expression;
  }
  return node;
}

function commandCatalogSha(entries) {
  return sha256(
    Buffer.from(
      entries.map((entry) => `${entry.index}\0${entry.name}\0${entry.kind}\0${entry.facet ?? ''}\n`).join(''),
      'utf8',
    ),
  );
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function normalize(value) {
  return value.replace(/\s+/gu, ' ').trim();
}

function visit(node, visitor) {
  visitor(node);
  ts.forEachChild(node, (child) => visit(child, visitor));
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function readOption(name) {
  const index = process.argv.indexOf(name);
  if (index < 0) return null;
  invariant(process.argv[index + 1], `${name} requires a value`);
  return process.argv[index + 1];
}
