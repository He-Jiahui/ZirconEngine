import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CLASSES_PATH = 'src/sim/content/classes.ts';
const EFFECT_DISPATCH_PATH = 'src/sim/combat/effect_dispatch.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const knownAbilityCatalogPath = join(referenceRoot, 'known_ability_catalog.json');
const jsonOutput = join(referenceRoot, 'empowered_cone_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'empowered_cone_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const classes = sourceBlob(CLASSES_PATH);
  const dispatch = sourceBlob(EFFECT_DISPATCH_PATH);
  const catalog = readJson(knownAbilityCatalogPath);
  invariant(catalog.source_commit === SOURCE_COMMIT, 'known ability catalog source commit drifted');
  const codeById = new Map(catalog.abilities.map((ability) => [ability.id, ability.code]));
  const text = classes.toString('utf8');
  const abilities = ['glacial_front', 'dragons_breath'].map((id) =>
    parseAbility(text, id, required(codeById, id)),
  );
  invariant(abilities.length === 2 && abilities.every((ability) => ability.stage_count === 4),
    'current empowered cone roster drifted');
  invariant(abilities[0].stages.map((stage) => stage.range).join(',') === '7,10,13,16',
    'Glacial Front range stages drifted');
  invariant(abilities[1].stages.map((stage) => stage.angle).join(',') === '55,65,78,90',
    "Dragon's Breath angle stages drifted");
  const effectText = dispatch.toString('utf8');
  invariant(
    effectText.includes("case 'empoweredCone':") &&
      effectText.includes('const critRoll = ctx.rng.chance(ctx.spellCrit(p));') &&
      effectText.includes('ctx.rng.range(stage.min, stage.max) + spellPower') &&
      effectText.includes('if (m.dead) continue;') &&
      effectText.includes('if (eff.hotStreakOnce && hotStreakHit)'),
    'empowered cone dispatch order drifted',
  );
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/empowered_cone_contract_codegen.mjs',
    source_blobs: {
      [CLASSES_PATH]: sha256(classes),
      [EFFECT_DISPATCH_PATH]: sha256(dispatch),
    },
    semantics: {
      stage: 'clamp level to the declared stage count before selecting range and effect values',
      hit_order: 'eligible target consumes crit then damage RNG; damage resolves before post-hit control auras and combat entry',
      hot_streak: 'Dragon\'s Breath records one aggregate hit/crit result after the target loop',
    },
    abilities,
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'empowered cone JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'empowered cone Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} empowered cone contract for ${SOURCE_COMMIT}\n`);
}

function parseAbility(source, id, code) {
  const marker = `\n  ${id}: {`;
  const start = source.indexOf(marker);
  invariant(start >= 0, `${id} definition is missing`);
  const bodyStart = start + marker.length;
  const bodyEnd = source.indexOf('\n  },', bodyStart);
  invariant(bodyEnd >= 0, `${id} definition no longer has a literal boundary`);
  const body = source.slice(bodyStart, bodyEnd);
  const effectStart = body.indexOf("type: 'empoweredCone'");
  invariant(effectStart >= 0, `${id} is no longer an empowered cone`);
  const effectEnd = body.indexOf('\n      },', effectStart);
  invariant(effectEnd >= 0, `${id} empowered cone effect has no literal boundary`);
  const effect = body.slice(effectStart, effectEnd);
  const stageBlock = capture(effect, /stages:\s*\[([\s\S]*?)\n\s*\],?$/, `${id} stages`)[1];
  const stages = [...stageBlock.matchAll(/\{\s*([^}]+)\s*\}/g)].map((match) => {
    const fields = match[1];
    return {
      range: numeric(fields, 'range', id),
      angle: optionalNumeric(fields, 'angle'),
      min: numeric(fields, 'min', id),
      max: numeric(fields, 'max', id),
      root_duration: optionalNumeric(fields, 'rootDuration'),
      incapacitate_duration: optionalNumeric(fields, 'incapacitateDuration'),
    };
  });
  invariant(stages.length === 4, `${id} stage count drifted`);
  return {
    id,
    code,
    learn_level: numeric(body, 'learnLevel', id),
    resource_cost: numeric(body, 'cost', id),
    school: capture(body, /school:\s*'([^']+)'/, `${id} school`)[1],
    cast_time: numeric(body, 'castTime', id),
    cooldown: numeric(body, 'cooldown', id),
    stage_count: numeric(body, 'empowerStages', id),
    angle: numeric(effect, 'angle', id),
    slow_mult: optionalNumeric(effect, 'slowMult'),
    slow_duration: optionalNumeric(effect, 'slowDuration'),
    fx: optionalText(effect, 'fx') ?? 'frostCone',
    guaranteed_crit_level: optionalNumeric(effect, 'guaranteedCritLevel'),
    hot_streak_once: /hotStreakOnce:\s*true/.test(effect),
    stages,
  };
}

function renderZr(document) {
  const byId = new Map(document.abilities.map((ability) => [ability.id, ability]));
  const glacial = required(byId, 'glacial_front');
  const dragon = required(byId, 'dragons_breath');
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub glacialFrontAbilityCode(required: bool): uint { return required ? <uint>${glacial.code} : <uint>0; }\n` +
    `pub dragonsBreathAbilityCode(required: bool): uint { return required ? <uint>${dragon.code} : <uint>0; }\n` +
    `pub isEmpoweredCone(ability: uint): bool { return ability == <uint>${glacial.code} || ability == <uint>${dragon.code}; }\n` +
    `pub stageCount(ability: uint): int { return isEmpoweredCone(ability) ? 4 : 0; }\n` +
    renderStageField('stageRange', 'range', 'float', document.abilities, 0) +
    renderStageField('stageAngle', 'angle', 'float', document.abilities, (ability) => ability.angle) +
    renderStageField('stageMinimumDamage', 'min', 'float', document.abilities, 0) +
    renderStageField('stageMaximumDamage', 'max', 'float', document.abilities, 0) +
    renderStageField('stageRootDuration', 'root_duration', 'float', document.abilities, 0) +
    renderStageField('stageIncapacitateDuration', 'incapacitate_duration', 'float', document.abilities, 0) +
    renderAbilityField('learnLevel', 'learn_level', document.abilities, 0) +
    renderAbilityField('resourceCost', 'resource_cost', document.abilities, 0) +
    renderAbilityField('castTime', 'cast_time', document.abilities, 0) +
    renderAbilityField('cooldown', 'cooldown', document.abilities, 0) +
    renderAbilityField('slowMultiplier', 'slow_mult', document.abilities, 0) +
    renderAbilityField('slowDuration', 'slow_duration', document.abilities, 0) +
    renderAbilityField('guaranteedCritLevel', 'guaranteed_crit_level', document.abilities, 0) +
    renderAbilityStringField('spellFx', 'fx', document.abilities, 'frostCone') +
    renderAbilityStringField('school', 'school', document.abilities, '') +
    `pub hotStreakOnce(ability: uint): bool { return ability == <uint>${dragon.code}; }\n`;
}

function renderStageField(name, field, type, abilities, fallback) {
  let output = `pub ${name}(ability: uint, level: int): ${type} {\n`;
  for (const ability of abilities) {
    output += `    if (ability == <uint>${ability.code}) {\n`;
    for (const [index, stage] of ability.stages.entries()) {
      const value = stage[field] ?? (typeof fallback === 'function' ? fallback(ability) : fallback);
      output += `        if (level == ${index + 1}) return ${zrNumber(value)};\n`;
    }
    output += '        return 0.0;\n    }\n';
  }
  return `${output}    return 0.0;\n}\n`;
}

function renderAbilityField(name, field, abilities, fallback) {
  let output = `pub ${name}(ability: uint): float {\n`;
  for (const ability of abilities) {
    output += `    if (ability == <uint>${ability.code}) return ${zrNumber(ability[field] ?? fallback)};\n`;
  }
  return `${output}    return 0.0;\n}\n`;
}

function renderAbilityStringField(name, field, abilities, fallback) {
  let output = `pub ${name}(ability: uint): string {\n`;
  for (const ability of abilities) {
    output += `    if (ability == <uint>${ability.code}) return ${JSON.stringify(ability[field] ?? fallback)};\n`;
  }
  return `${output}    return ${JSON.stringify(fallback)};\n}\n`;
}

function numeric(source, field, label) {
  return Number(capture(source, new RegExp(`${field}:\\s*([0-9.]+)`), `${label} ${field}`)[1]);
}

function optionalNumeric(source, field) {
  const match = source.match(new RegExp(`${field}:\\s*([0-9.]+)`));
  return match ? Number(match[1]) : null;
}

function optionalText(source, field) {
  const match = source.match(new RegExp(`${field}:\\s*'([^']+)'`));
  return match ? match[1] : null;
}

function zrNumber(value) {
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer', maxBuffer: 64 * 1024 * 1024,
  });
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:empowered-cone-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:empowered-cone-contract`);
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function capture(source, expression, label) {
  const match = source.match(expression);
  invariant(match, `${label} is no longer a literal contract`);
  return match;
}

function required(values, key) {
  const value = values.get(key);
  invariant(value !== undefined, `missing ${key}`);
  return value;
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
