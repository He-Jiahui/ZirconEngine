// Extracts tickDelveBadAir with the TypeScript AST from the pinned source.
import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const sourceRoot = process.env.WOC_GIT_ROOT;
const sourceCommit = process.env.WOC_GIT_COMMIT;
if (!sourceRoot || !sourceCommit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const sourcePath = 'src/sim/delves/runs.ts';
const sourceText = execFileSync('git', ['-C', sourceRoot, 'show', `${sourceCommit}:${sourcePath}`], {
  encoding: 'utf8',
  maxBuffer: 32 * 1024 * 1024,
});
const sourceFile = ts.createSourceFile(sourcePath, sourceText, ts.ScriptTarget.Latest, true,
  ts.ScriptKind.TS);

const functionBody = functionDeclaration('tickDelveBadAir').body;
const aura = findAuraObject(functionBody);
const content = {
  interval_seconds: numberConstant('DELVE_BAD_AIR_INTERVAL'),
  aura: {
    id: stringProperty(aura, 'id'),
    name: stringProperty(aura, 'name'),
    kind: stringProperty(aura, 'kind'),
    school: stringProperty(aura, 'school'),
    remaining: numberProperty(aura, 'remaining'),
    duration: numberProperty(aura, 'duration'),
    value: numberProperty(aura, 'value'),
    tick_interval: numberProperty(aura, 'tickInterval'),
    tick_timer: numberProperty(aura, 'tickTimer'),
    source_id: selfSourceProperty(aura, 'sourceId') ? 'self' : 'unsupported',
  },
};
process.stdout.write(JSON.stringify(content));

function functionDeclaration(name) {
  for (const statement of sourceFile.statements) {
    if (ts.isFunctionDeclaration(statement) && statement.name?.text === name && statement.body) {
      return statement;
    }
  }
  throw new Error(`function ${name} is missing`);
}

function declaration(name) {
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const candidate of statement.declarationList.declarations) {
      if (ts.isIdentifier(candidate.name) && candidate.name.text === name && candidate.initializer) {
        return candidate.initializer;
      }
    }
  }
  throw new Error(`constant ${name} is missing`);
}

function numberConstant(name) {
  return numeric(declaration(name), name);
}

function findAuraObject(root) {
  let found = null;
  const visit = (node) => {
    if (found) return;
    if (ts.isObjectLiteralExpression(node)) {
      try {
        if (stringProperty(node, 'id') === 'bad_air') {
          found = node;
          return;
        }
      } catch {
        // Other object literals in the function are not aura specs.
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(root);
  if (!found) throw new Error('Bad Air aura object is missing');
  return found;
}

function property(object, name) {
  for (const member of object.properties) {
    if (ts.isPropertyAssignment(member) && ts.isIdentifier(member.name) && member.name.text === name) {
      return member.initializer;
    }
  }
  throw new Error(`aura property ${name} is missing`);
}

function stringProperty(object, name) {
  const value = property(object, name);
  if (!ts.isStringLiteral(value)) throw new Error(`${name} is not a string literal`);
  return value.text;
}

function numberProperty(object, name) {
  return numeric(property(object, name), name);
}

function numeric(node, label) {
  if (ts.isNumericLiteral(node)) return Number(node.text);
  throw new Error(`${label} is not a numeric literal`);
}

function selfSourceProperty(object, name) {
  const value = property(object, name);
  return ts.isPropertyAccessExpression(value) && ts.isIdentifier(value.expression) &&
    value.expression.text === 'p' && value.name.text === 'id';
}
