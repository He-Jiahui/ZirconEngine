import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CLASSES_PATH = 'src/sim/content/classes.ts';
const CASTING_LIFECYCLE_PATH = 'src/sim/combat/casting_lifecycle.ts';
const EFFECT_DISPATCH_PATH = 'src/sim/combat/effect_dispatch.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'temporal_reversal_offer_contract.json');
const zrOutput = join(
  projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'temporal_reversal_offer_contract.zr',
);
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blobs = Object.fromEntries(
    [CLASSES_PATH, CASTING_LIFECYCLE_PATH, EFFECT_DISPATCH_PATH, TYPES_PATH].map(
      (path) => [path, sourceBlob(path)],
    ),
  );
  const classes = blobs[CLASSES_PATH].toString('utf8');
  const dispatch = blobs[EFFECT_DISPATCH_PATH].toString('utf8');
  const ability = abilityContract(classes, blobs[TYPES_PATH].toString('utf8'));
  const targetResolution = targetResolutionContract(blobs[CASTING_LIFECYCLE_PATH].toString('utf8'));
  assertDispatchContract(dispatch);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/temporal_reversal_offer_contract_codegen.mjs',
    source_blobs: Object.fromEntries(
      Object.entries(blobs).map(([path, value]) => [path, sha256(value)]),
    ),
    ability,
    target_resolution: targetResolution,
    dispatch: {
      effect_type: 'resurrectAlly',
      offer_owner: 'src/sim/combat/resurrection_offer.ts#offerResurrection',
      school: 'arcane',
      fx: 'temporalGlyph',
      requires_dead_target: true,
    },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Temporal Reversal JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Temporal Reversal Zr contract');
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} Temporal Reversal offer contract for ${SOURCE_COMMIT}\n`,
  );
}

function targetResolutionContract(source) {
  const start = source.indexOf('function resolveDeadAllyTarget(');
  const end = source.indexOf('export function castAbility(', start);
  if (start < 0 || end < 0) throw new Error('Temporal Reversal target resolver is absent');
  const resolver = source.slice(start, end);
  const required = [
    'const id = overrideId ?? p.targetId;',
    'if (id === null) return null;',
    'if (!t || !t.dead || t.kind !== \'player\') return null;',
    'const party = ctx.partyOf(p.id);',
    'party && party.members.includes(t.id) ? t : null',
  ];
  for (const fragment of required) {
    if (!resolver.includes(fragment)) throw new Error(`Temporal Reversal target resolver drifted: ${fragment}`);
  }
  return {
    override_precedes_current_target: true,
    requires_dead_player: true,
    requires_party_membership: true,
    no_self_fallback: true,
  };
}

function abilityContract(source, types) {
  const block = capture(
    source,
    /\btemporal_reversal:\s*\{([\s\S]*?)\n\s*\},\n\s*\/\/ ---- Chronomancy out-of-combat mass resurrection/,
    'Temporal Reversal ability block',
  )[1];
  const id = capture(block, /\bid:\s*'([^']+)'/, 'Temporal Reversal id')[1];
  const learnLevel = number(block, /\blearnLevel:\s*(\d+)/, 'Temporal Reversal learn level');
  const cost = number(block, /\bcost:\s*(\d+)/, 'Temporal Reversal cost');
  const castTime = number(block, /\bcastTime:\s*(\d+)/, 'Temporal Reversal cast time');
  const cooldown = number(block, /\bcooldown:\s*(\d+)/, 'Temporal Reversal cooldown');
  const range = number(block, /\brange:\s*(\d+)/, 'Temporal Reversal range');
  const school = capture(block, /\bschool:\s*'([^']+)'/, 'Temporal Reversal school')[1];
  const targetType = capture(block, /\btargetType:\s*'([^']+)'/, 'Temporal Reversal target type')[1];
  const hpFraction = number(
    block, /effects:\s*\[\{\s*type:\s*'resurrectAlly',\s*hpFrac:\s*([\d.]+)\s*}\s*]/,
    'Temporal Reversal resurrect effect',
  );
  const globalCooldown = number(types, /export const GCD = ([\d.]+)/, 'base global cooldown');
  const minimumGlobalCooldown = number(
    types, /export const MIN_GCD = ([\d.]+)/, 'minimum global cooldown',
  );
  if (id !== 'temporal_reversal' || learnLevel !== 16 || cost !== 60 || castTime !== 2 ||
      cooldown !== 600 || range !== 30 || school !== 'arcane' || targetType !== 'friendly' ||
      !block.includes('targetsDead: true') || hpFraction !== 0.35 ||
      globalCooldown !== 1.5 || minimumGlobalCooldown !== 0.75) {
    throw new Error('Temporal Reversal ability contract drifted');
  }
  return {
    id,
    learn_level: learnLevel,
    cost,
    cast_time_seconds: castTime,
    global_cooldown_seconds: globalCooldown,
    minimum_global_cooldown_seconds: minimumGlobalCooldown,
    cooldown_seconds: cooldown,
    range,
    school,
    target_type: targetType,
    targets_dead: true,
    hp_fraction: hpFraction,
  };
}

function assertDispatchContract(source) {
  const start = source.indexOf("case 'resurrectAlly':");
  const end = source.indexOf("case 'massResurrectGroup':", start);
  if (start < 0 || end < 0) throw new Error('Temporal Reversal effect-dispatch branch is absent');
  const branch = source.slice(start, end);
  const required = [
    'const ally = target;',
    'if (!ally?.dead) break;',
    'offerResurrection(ctx, p, ally, eff.hpFrac);',
    "type: 'spellfx'",
    'sourceId: p.id',
    'targetId: ally.id',
    "school: 'arcane'",
    "fx: 'temporalGlyph'",
  ];
  for (const fragment of required) {
    if (!branch.includes(fragment)) throw new Error(`Temporal Reversal dispatch drifted: ${fragment}`);
  }
}

function renderZr(document) {
  const ability = document.ability;
  return [
    `// Generated from ${SOURCE_COMMIT}; do not edit by hand.`,
    '',
    `pub abilityId(required: bool): string { return required ? "${ability.id}" : ""; }`,
    `pub resourceCost(required: bool): int { return required ? ${ability.cost} : 0; }`,
    `pub hpFraction(required: bool): float { return required ? ${ability.hp_fraction} : 0.0; }`,
    `pub castTimeSeconds(required: bool): float { return required ? ${ability.cast_time_seconds}.0 : 0.0; }`,
    `pub globalCooldownSeconds(required: bool): float { return required ? ${ability.global_cooldown_seconds} : 0.0; }`,
    `pub minimumGlobalCooldownSeconds(required: bool): float { return required ? ${ability.minimum_global_cooldown_seconds} : 0.0; }`,
    `pub cooldownSeconds(required: bool): float { return required ? ${ability.cooldown_seconds}.0 : 0.0; }`,
    `pub range(required: bool): float { return required ? ${ability.range}.0 : 0.0; }`,
    'pub targetsDead(required: bool): bool { return required; }',
    'pub overridePrecedesCurrentTarget(required: bool): bool { return required; }',
    'pub requiresPartyMembership(required: bool): bool { return required; }',
    'pub noSelfFallback(required: bool): bool { return required; }',
    'pub school(required: bool): string { return required ? "arcane" : ""; }',
    'pub fx(required: bool): string { return required ? "temporalGlyph" : ""; }',
    '',
    'pub contractTest(): int {',
    '    return abilityId(true) == "temporal_reversal" && resourceCost(true) == 60 && hpFraction(true) == 0.35 &&',
    '        castTimeSeconds(true) == 2.0 && globalCooldownSeconds(true) == 1.5 &&',
    '        minimumGlobalCooldownSeconds(true) == 0.75 && cooldownSeconds(true) == 600.0 &&',
    '        range(true) == 30.0 && targetsDead(true) && overridePrecedesCurrentTarget(true) &&',
    '        requiresPartyMembership(true) && noSelfFallback(true) && school(true) == "arcane" &&',
    '        fx(true) == "temporalGlyph" ? 1 : -1;',
    '}',
    '',
  ].join('\n');
}

function sourceBlob(path) {
  return execFileSync(
    'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`],
    { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 },
  );
}

function number(source, expression, label) { return Number(capture(source, expression, label)[1]); }
function capture(source, expression, label) {
  const match = source.match(expression);
  if (!match) throw new Error(`${label} is no longer a literal contract`);
  return match;
}
function writeOrCheck(path, content, label) {
  if (checkOnly) {
    if (!existsSync(path)) throw new Error(`${label} is missing; run its generator`);
    if (readFileSync(path, 'utf8') !== content) throw new Error(`${label} is stale; run its generator`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, 'utf8');
}
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
