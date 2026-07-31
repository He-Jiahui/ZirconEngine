// Reads the room-rule source with the TypeScript AST. This avoids executing the
// source aggregate (which currently has unresolved top-level-await imports under
// the pinned loader) while still extracting structured authored constants.
import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const sourceRoot = process.env.WOC_GIT_ROOT;
const sourceCommit = process.env.WOC_GIT_COMMIT;
if (!sourceRoot || !sourceCommit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const sourcePath = 'src/sim/delves/drowned_litany_rooms.ts';
const sourceText = execFileSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${sourcePath}`], {
  encoding: 'utf8',
  maxBuffer: 32 * 1024 * 1024,
});
const sourceFile = ts.createSourceFile(sourcePath, sourceText, ts.ScriptTarget.Latest, true,
  ts.ScriptKind.TS);

const content = {
  egg_sac_wave_radius: numberConstant('EGG_SAC_WAVE_RADIUS'),
  egg_sac_wave_percent: numberConstant('EGG_SAC_WAVE_PCT'),
  egg_sac_burst_despawn: numberConstant('EGG_SAC_BURST_DESPAWN'),
  hatchling_body_r: numberConstant('HATCHLING_BODY_R'),
  hatchling_spawn_attempts: numberConstant('HATCHLING_SPAWN_ATTEMPTS'),
  egg_sac_spots: positionArray('BAPTISTRY_EGG_SAC_SPOTS', false),
  waves: nestedSpawnArray('BAPTISTRY_WAVES'),
};

process.stdout.write(JSON.stringify(content));

function declaration(name) {
  let found = null;
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const candidate of statement.declarationList.declarations) {
      if (ts.isIdentifier(candidate.name) && candidate.name.text === name) {
        found = candidate;
        break;
      }
    }
    if (found) break;
  }
  if (!found || !found.initializer) throw new Error(`missing initializer for ${name}`);
  return found.initializer;
}

function numberConstant(name) {
  return numericExpression(declaration(name), name);
}

function numericExpression(node, label) {
  if (ts.isNumericLiteral(node)) return Number(node.text);
  if (ts.isPrefixUnaryExpression(node) && node.operator === ts.SyntaxKind.MinusToken &&
      ts.isNumericLiteral(node.operand)) {
    return -Number(node.operand.text);
  }
  throw new Error(`${label} is not a numeric literal`);
}

function arrayInitializer(name) {
  const node = declaration(name);
  if (!ts.isArrayLiteralExpression(node)) throw new Error(`${name} is not an array literal`);
  return node;
}

function property(object, name) {
  if (!ts.isObjectLiteralExpression(object)) throw new Error(`${name} parent is not an object literal`);
  for (const member of object.properties) {
    if (ts.isPropertyAssignment(member) && ts.isIdentifier(member.name) && member.name.text === name) {
      return member.initializer;
    }
  }
  throw new Error(`object property ${name} is missing`);
}

function position(object, mob) {
  const x = numericExpression(property(object, 'x'), 'x');
  const z = numericExpression(property(object, 'z'), 'z');
  if (!mob) return { x, z };
  const mobId = property(object, 'mobId');
  if (!ts.isStringLiteral(mobId)) throw new Error('mobId is not a string literal');
  return { mob_id: mobId.text, x, z };
}

function positionArray(name, mob) {
  return arrayInitializer(name).elements.map((entry) => position(entry, mob));
}

function nestedSpawnArray(name) {
  return arrayInitializer(name).elements.map((wave) => {
    if (!ts.isArrayLiteralExpression(wave)) throw new Error(`${name} contains a non-array wave`);
    return wave.elements.map((entry) => position(entry, true));
  });
}
