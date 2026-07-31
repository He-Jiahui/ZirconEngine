import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const ts = require('typescript');
const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ONLINE_PATH = 'src/net/online.ts';
const SERVER_PATH = 'server/game.ts';
const TALENT_ROWS_PATH = 'src/sim/content/talent_rows.ts';
const WARRIOR_ROWS_PATH = 'src/sim/content/warrior_rows.ts';
const CLASSIC_ROWS_PATH = 'src/sim/content/choice_rows_classic.ts';
const TALENT_REDUCER_PATH = 'src/sim/progression/talents.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'talent_row_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'talent_row_contract.zr');
const checkOnly = process.argv.includes('--check');

const CLASS_BY_EXPORT = {
  WARRIOR_ROWS: 'warrior',
  PALADIN_CHOICE_ROWS: 'paladin',
  HUNTER_CHOICE_ROWS: 'hunter',
  ROGUE_CHOICE_ROWS: 'rogue',
  PRIEST_CHOICE_ROWS: 'priest',
  SHAMAN_CHOICE_ROWS: 'shaman',
  MAGE_CHOICE_ROWS: 'mage',
  WARLOCK_CHOICE_ROWS: 'warlock',
  DRUID_CHOICE_ROWS: 'druid',
};
const CLASS_ORDER = Object.values(CLASS_BY_EXPORT);
const ROW_LEVELS = [5, 8, 11, 14, 17, 20];

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  const commandCatalog = readJson(join(referenceRoot, 'command_catalog.json'));
  const payloadCatalog = readJson(join(referenceRoot, 'command_payload_catalog.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before talent row contracts');
  invariant(commandCatalog.source_commit === SOURCE_COMMIT && payloadCatalog.source_commit === SOURCE_COMMIT,
    'talent row command references are not pinned to the current source');
  const blobs = Object.fromEntries([
    ONLINE_PATH, SERVER_PATH, TALENT_ROWS_PATH, WARRIOR_ROWS_PATH, CLASSIC_ROWS_PATH, TALENT_REDUCER_PATH,
  ].map((path) => [path, sourceBlob(path)]));
  const online = blobs[ONLINE_PATH].toString('utf8');
  const server = blobs[SERVER_PATH].toString('utf8');
  const reducer = blobs[TALENT_REDUCER_PATH].toString('utf8');
  const command = bindCommand(commandCatalog, payloadCatalog);
  invariant(online.includes("this.cmd({ cmd: 'selectTalentRow', level, optionId });"),
    'selectTalentRow client payload no longer forwards level and optionId directly');
  invariant(server.includes("case 'selectTalentRow':") &&
    server.includes('parseTalentRowLevel(msg.level)') &&
    server.includes('parseTalentOptionId(msg.optionId)') &&
    server.includes('sim.selectTalentRow(level, optionId, pid);'),
  'selectTalentRow server parser or Sim handoff drifted');
  invariant(reducer.includes('export function selectTalentRow(') &&
    reducer.includes('const row = rowForLevel(r.meta.cls, level);') &&
    reducer.includes('if (optionId === null) delete cand.rows[level];') &&
    reducer.includes('return applyTalentAllocation(ctx, cand, pid);'),
  'selectTalentRow reducer transaction semantics drifted');
  const rows = parseAllRows(blobs[WARRIOR_ROWS_PATH].toString('utf8'), blobs[CLASSIC_ROWS_PATH].toString('utf8'));
  validateRows(rows);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/talent_row_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    command,
    row_levels: ROW_LEVELS,
    options_per_row: 3,
    rows,
    source_semantics: {
      transaction: 'select validates class row, player level and option id before cloning allocation and entering the shared applyTalentAllocation choke point',
      clearing: 'null option removes only the selected row and preserves specialization and other rows',
      locking: 'combat and arena locks are enforced by the shared allocation commit after row validation',
    },
    zrvm_backend_status: {
      state: 'blocked',
      reason: 'The number plus string-or-null CommandValue envelope needs Plugin08 lossless transactional transport for dynamic ZrVM dispatch',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'talent row JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'talent row Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} talent row contract for ${SOURCE_COMMIT}\n`);
}

function bindCommand(commandCatalog, payloadCatalog) {
  const command = commandCatalog.entries.find((entry) => entry.name === 'selectTalentRow');
  invariant(command?.kind === 'client_send', 'selectTalentRow is not a client-send command');
  const payload = payloadCatalog.entries.find((entry) => entry.id === command.index);
  invariant(payload?.name === 'selectTalentRow', 'selectTalentRow source payload row is missing');
  const matchingSite = payload.client_sends.find((site) => site.method === 'selectTalentRow' &&
    JSON.stringify(site.fields.map((field) => field.name)) === JSON.stringify(['level', 'optionId']));
  invariant(matchingSite, 'selectTalentRow source method or field shape drifted');
  return {
    id: command.index,
    name: command.name,
    source_shape: { method: 'selectTalentRow', fields: ['level:number', 'optionId:string|null'] },
  };
}

function parseAllRows(warriorSource, classicSource) {
  const rows = {};
  Object.assign(rows, parseExports(WARRIOR_ROWS_PATH, warriorSource));
  Object.assign(rows, parseExports(CLASSIC_ROWS_PATH, classicSource));
  return Object.fromEntries(CLASS_ORDER.map((cls) => [cls, rows[cls]]));
}

function parseExports(path, source) {
  const file = ts.createSourceFile(path, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const rows = {};
  for (const statement of file.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (!ts.isIdentifier(declaration.name)) continue;
      const cls = CLASS_BY_EXPORT[declaration.name.text];
      if (!cls || !declaration.initializer) continue;
      const initializer = unwrap(declaration.initializer);
      const array = declaration.name.text === 'WARRIOR_ROWS'
        ? initializer
        : propertyValue(initializer, 'rows');
      invariant(array && ts.isArrayLiteralExpression(unwrap(array)), `${declaration.name.text} rows are not a literal array`);
      rows[cls] = parseRowArray(unwrap(array), declaration.name.text);
    }
  }
  return rows;
}

function parseRowArray(array, label) {
  return array.elements.map((element, index) => {
    const row = unwrap(element);
    invariant(ts.isObjectLiteralExpression(row), `${label} row ${index} is not an object`);
    const level = numericProperty(row, 'level');
    const options = unwrap(propertyValue(row, 'options'));
    invariant(ts.isArrayLiteralExpression(options), `${label} level ${level} options are not an array`);
    return {
      level,
      options: options.elements.map((option, optionIndex) => {
        const object = unwrap(option);
        invariant(ts.isObjectLiteralExpression(object), `${label} level ${level} option ${optionIndex} is not an object`);
        return stringProperty(object, 'id');
      }),
    };
  });
}

function unwrap(expression) {
  let value = expression;
  while (ts.isAsExpression(value) || ts.isTypeAssertionExpression(value) || ts.isSatisfiesExpression(value) || ts.isParenthesizedExpression(value)) {
    value = value.expression;
  }
  return value;
}

function propertyValue(object, name) {
  invariant(ts.isObjectLiteralExpression(object), `expected object while reading ${name}`);
  for (const property of object.properties) {
    if (!ts.isPropertyAssignment(property) || !property.name) continue;
    const propertyName = ts.isIdentifier(property.name) || ts.isStringLiteral(property.name) ? property.name.text : '';
    if (propertyName === name) return property.initializer;
  }
  throw new Error(`missing literal property ${name}`);
}

function numericProperty(object, name) {
  const value = unwrap(propertyValue(object, name));
  invariant(ts.isNumericLiteral(value), `${name} is not numeric`);
  return Number(value.text);
}

function stringProperty(object, name) {
  const value = unwrap(propertyValue(object, name));
  invariant(ts.isStringLiteral(value), `${name} is not a string literal`);
  return value.text;
}

function validateRows(rows) {
  invariant(Object.keys(rows).length === CLASS_ORDER.length, 'not every current player class has rows');
  for (const cls of CLASS_ORDER) {
    const tree = rows[cls];
    invariant(Array.isArray(tree) && tree.length === ROW_LEVELS.length, `${cls} does not have six rows`);
    const seen = new Set();
    tree.forEach((row, index) => {
      invariant(row.level === ROW_LEVELS[index], `${cls} row ${index} level drifted`);
      invariant(row.options.length === 3, `${cls} level ${row.level} option count drifted`);
      for (const option of row.options) {
        invariant(option.length > 0 && !seen.has(option), `${cls} option identity is missing or duplicated`);
        seen.add(option);
      }
    });
  }
}

function renderZr(document) {
  const lines = [
    `// Generated from ${SOURCE_COMMIT}; do not edit by hand.`,
    `pub selectTalentRowCommandId(required: bool): uint { return required ? <uint>${document.command.id} : <uint>0; }`,
    'pub isRowLevel(level: int): bool { return level == 5 || level == 8 || level == 11 || level == 14 || level == 17 || level == 20; }',
    'pub knownClass(classId: string): bool {',
    ...CLASS_ORDER.map((cls) => `    if (classId == "${cls}") return true;`),
    '    return false;',
    '}',
    'pub hasOption(classId: string, level: int, optionId: string): bool {',
  ];
  for (const cls of CLASS_ORDER) {
    for (const row of document.rows[cls]) {
      for (const option of row.options) {
        lines.push(`    if (classId == "${cls}" && level == ${row.level} && optionId == "${option}") return true;`);
      }
    }
  }
  lines.push('    return false;', '}');
  return `${lines.join('\n')}\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:talent-row-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:talent-row-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
