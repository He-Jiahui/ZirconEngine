import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const root = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;
if (!root || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const paths = {
  lockpick: 'src/sim/lockpick.ts',
  tiers: 'src/sim/content/delves/lockpick_tiers.ts',
  controller: 'src/sim/delves/lockpick_controller.ts',
  types: 'src/sim/types.ts',
};
const readSource = (path) =>
  execFileSync('git', ['-C', root, 'show', `${commit}:${path}`], { encoding: 'utf8' });
const sources = Object.fromEntries(Object.entries(paths).map(([name, path]) => [name, readSource(path)]));

function sourceFile(path, text) {
  return ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
}

function declarationInitializer(file, name) {
  for (const statement of file.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name && declaration.initializer) {
        return declaration.initializer;
      }
    }
  }
  throw new Error(`missing declaration ${name}`);
}

function objectProperty(object, name) {
  if (!ts.isObjectLiteralExpression(object)) throw new Error(`expected object for ${name}`);
  for (const property of object.properties) {
    if (!ts.isPropertyAssignment(property)) continue;
    const key = property.name.getText().replace(/^['"]|['"]$/g, '');
    if (key === name) return property.initializer;
  }
  throw new Error(`missing property ${name}`);
}

function expressionValue(expression, resolveIdentifier = null) {
  if (ts.isParenthesizedExpression(expression)) return expressionValue(expression.expression, resolveIdentifier);
  if (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) return expression.text;
  if (ts.isNumericLiteral(expression)) return Number(expression.text);
  if (ts.isIdentifier(expression) && resolveIdentifier) return resolveIdentifier(expression.text);
  if (expression.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (expression.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (ts.isPrefixUnaryExpression(expression)) {
    const value = expressionValue(expression.operand, resolveIdentifier);
    if (typeof value !== 'number') throw new Error('invalid numeric unary operand');
    return expression.operator === ts.SyntaxKind.MinusToken ? -value : value;
  }
  if (ts.isBinaryExpression(expression)) {
    const left = expressionValue(expression.left, resolveIdentifier);
    const right = expressionValue(expression.right, resolveIdentifier);
    if (typeof left !== 'number' || typeof right !== 'number') {
      throw new Error('non-numeric binary expression');
    }
    if (expression.operatorToken.kind === ts.SyntaxKind.PlusToken) return left + right;
    if (expression.operatorToken.kind === ts.SyntaxKind.MinusToken) return left - right;
    if (expression.operatorToken.kind === ts.SyntaxKind.AsteriskToken) return left * right;
    if (expression.operatorToken.kind === ts.SyntaxKind.SlashToken) return left / right;
  }
  if (ts.isArrayLiteralExpression(expression)) return expression.elements.map((item) => expressionValue(item, resolveIdentifier));
  if (ts.isObjectLiteralExpression(expression)) {
    const result = {};
    for (const property of expression.properties) {
      if (!ts.isPropertyAssignment(property)) throw new Error('unsupported object property');
      const key = property.name.getText().replace(/^['"]|['"]$/g, '');
      result[key] = expressionValue(property.initializer, resolveIdentifier);
    }
    return result;
  }
  throw new Error(`unsupported expression ${expression.getText()}`);
}

function numberConstant(file, name, seen = new Set()) {
  if (seen.has(name)) throw new Error(`cyclic numeric constant ${name}`);
  const nextSeen = new Set(seen);
  nextSeen.add(name);
  const value = expressionValue(
    declarationInitializer(file, name),
    (identifier) => numberConstant(file, identifier, nextSeen),
  );
  if (typeof value !== 'number') throw new Error(`expected numeric ${name}`);
  return value;
}

const lockpickFile = sourceFile(paths.lockpick, sources.lockpick);
const tiersFile = sourceFile(paths.tiers, sources.tiers);
const typesFile = sourceFile(paths.types, sources.types);
const actionDelta = expressionValue(declarationInitializer(lockpickFile, 'ACTION_DELTA'));
const pickActions = expressionValue(declarationInitializer(lockpickFile, 'PICK_ACTIONS'));
const anteToTier = expressionValue(declarationInitializer(lockpickFile, 'ANTE_TO_TIER'));
const anteToPages = expressionValue(declarationInitializer(lockpickFile, 'ANTE_TO_PAGES'));
const anteToTries = expressionValue(declarationInitializer(lockpickFile, 'ANTE_TO_TRIES'));
const anteToStepTimeoutMs = expressionValue(declarationInitializer(lockpickFile, 'ANTE_TO_STEP_TIMEOUT_MS'));
const presets = expressionValue(declarationInitializer(tiersFile, 'LOCKPICK_TIER_PRESETS'));
const rewards = expressionValue(declarationInitializer(tiersFile, 'LOCKPICK_TIER_REWARD'));
const tickSeconds = numberConstant(typesFile, 'DT');
const controller = sources.controller;

for (const [name, value] of Object.entries(actionDelta)) {
  if (!pickActions.includes(name) || !Number.isInteger(value) || value < -2 || value > 2) {
    throw new Error(`invalid lockpick action ${name}`);
  }
}
for (const ante of ['1', '2', '3']) {
  if (!anteToTier[ante] || !anteToPages[ante] || !anteToTries[ante] || !anteToStepTimeoutMs[ante]) {
    throw new Error(`incomplete ante ${ante}`);
  }
}
for (const tier of ['normal', 'heroic']) {
  const preset = presets[tier];
  if (!preset || !Array.isArray(preset.allowedActions) || preset.allowedActions.join(',') !== pickActions.join(',')) {
    throw new Error(`lockpick preset ${tier} drifted`);
  }
}
for (const marker of [
  'objectId * 0x9e3779b1',
  '((i + 1) * 0x9e3779b1)',
  'triesUsed * 0x85ebca6b',
  'Math.ceil(ms / (DT * 1000))',
]) {
  if (!(sources.lockpick.includes(marker) || controller.includes(marker))) {
    throw new Error(`lockpick source marker drifted: ${marker}`);
  }
}
if (tickSeconds !== 0.05) throw new Error(`unexpected simulation tick ${tickSeconds}`);

process.stdout.write(JSON.stringify({
  actions: pickActions.map((id, index) => ({ id, index, delta: actionDelta[id] })),
  antes: ['1', '2', '3'].map((ante) => ({
    ante: Number(ante),
    loot_tier: anteToTier[ante],
    pages: anteToPages[ante],
    tries: anteToTries[ante],
    step_timeout_ms: anteToStepTimeoutMs[ante],
  })),
  presets: ['normal', 'heroic'].map((id, index) => ({ id, index, ...presets[id] })),
  rewards,
  tick_milliseconds: tickSeconds * 1000,
  base_seed_multiplier: 0x9e3779b1,
  page_seed_multiplier: 0x9e3779b1,
  retry_seed_multiplier: 0x85ebca6b,
}));
