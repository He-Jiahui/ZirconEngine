import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const inputPath = join(projectRoot, 'contracts', 'm4_abilities.json');
const generatedRoot = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated');
const catalogPath = join(generatedRoot, 'm4_ability_catalog.zr');
const effectsPath = join(generatedRoot, 'm4_ability_effects.zr');
const checkOnly = process.argv.includes('--check');

const ABILITY_TEXT_FIELDS = [
  'id',
  'name',
  'class',
  'school',
  'scalesWith',
  'targetType',
  'targetMode',
  'description',
];
const ABILITY_METRIC_FIELDS = [
  'learnLevel',
  'cost',
  'castTime',
  'cooldown',
  'range',
  'minRange',
  'threatFlat',
  'threatMult',
  'channelDuration',
  'channelTicks',
  'awardsCombo',
];
const ABILITY_FLAG_FIELDS = [
  'requiresTarget', 'onNextSwing', 'offGcd', 'spendsCombo', 'partyOnlyTarget',
  'usableInForm', 'uninterruptible', 'projectile',
];
const EFFECT_METRIC_FIELDS = [
  'amount',
  'armor',
  'base',
  'bonus',
  'duration',
  'falloff',
  'fraction',
  'healFrac',
  'healMaxHpPct',
  'hostilePveDuration',
  'hostilePvpDuration',
  'hp',
  'interval',
  'jumps',
  'judgeMax',
  'judgeMin',
  'leechPct',
  'lockout',
  'max',
  'maxHpFraction',
  'maxTargets',
  'maxStacks',
  'mana',
  'min',
  'mult',
  'perCombo',
  'radius',
  'rageOnInterrupt',
  'selfRadius',
  'captureRadius',
  'groundDuration',
  'selfCooldownRate',
  'allyCooldownRate',
  'total',
  // Nested hunter-trap source fields are exposed as stable scalar metrics
  // without rewriting the source-shaped contract document.
  'trapArmTime',
  'trapLifetime',
  'value',
  'variance',
  'windowSec',
  'weaponMult',
];
const EFFECT_TEXT_FIELDS = ['auraKind', 'kind', 'mobId'];
const EFFECT_FLAG_FIELDS = [
  'canCrit', 'requiresBehind', 'spell', 'exhaust', 'groupOnly', 'stun',
];

main();

function main() {
  const document = JSON.parse(readFileSync(inputPath, 'utf8'));
  invariant(document.schema_version === 1, 'unsupported M4 ability catalog schema');
invariant(document.entries.length === 117, 'M4 ability catalog must contain 117 entries');
  invariant(
    document.catalog_sha256 === hashText(JSON.stringify(document.entries)),
    'M4 ability catalog entry fingerprint drifted',
  );
  validateEntries(document.entries);

  writeOrCheck(catalogPath, renderCatalog(document));
  writeOrCheck(effectsPath, renderEffects(document));
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} WOC M4 Zr ability projection ` +
      `(${document.catalog_sha256.slice(0, 15)})\n`,
  );
}

function validateEntries(entries) {
  const ids = new Set();
  for (const [index, entry] of entries.entries()) {
    invariant(entry.index === index, `M4 ability index drifted at ${entry.id}`);
    invariant(!ids.has(entry.id), `duplicate M4 ability id ${entry.id}`);
    ids.add(entry.id);
    const definition = entry.definition;
    invariant(definition.id === entry.id, `definition id drifted at ${entry.id}`);
    invariant(Array.isArray(definition.effects) && definition.effects.length > 0,
      `${entry.id} has no base effects`);
    let previousRank = 1;
    let previousLevel = definition.learnLevel;
    for (const rank of definition.ranks ?? []) {
      invariant(rank.rank > previousRank, `${entry.id} ranks are not ascending`);
      invariant(rank.level > previousLevel, `${entry.id} rank levels are not ascending`);
      invariant(Array.isArray(rank.effects) && rank.effects.length > 0,
        `${entry.id} rank ${rank.rank} has no effects`);
      previousRank = rank.rank;
      previousLevel = rank.level;
    }
    for (const rank of resolvedRanks(definition)) {
      for (const effect of rank.effects) validateEffect(entry.id, rank.rank, effect);
    }
  }
}

function validateEffect(abilityId, rank, effect) {
  invariant(typeof effect.type === 'string', `${abilityId} rank ${rank} effect has no type`);
  if (effect.type === 'consumeAura') {
    const hasAuraIds = Array.isArray(effect.auraIds) && effect.auraIds.length > 0 &&
      effect.auraIds.every((value) => typeof value === 'string');
    const hasAuraKind = effect.auraKind === 'dot' || effect.auraKind === 'hot';
    invariant(hasAuraIds || hasAuraKind,
      `${abilityId} rank ${rank} consumeAura must declare auraIds or auraKind`);
    invariant((effect.deal && typeof effect.deal === 'object') ||
      (effect.heal && typeof effect.heal === 'object'),
    `${abilityId} rank ${rank} consumeAura must declare deal or heal`);
    for (const [payload, values] of [['deal', effect.deal], ['heal', effect.heal]]) {
      if (!values) continue;
      for (const [field, value] of Object.entries(values)) {
        invariant(typeof value === 'number' && EFFECT_METRIC_FIELDS.includes(field),
          `${abilityId} rank ${rank} consumeAura has unsupported ${payload} field ${field}`);
      }
    }
  }
  if (effect.type === 'massTemporalEcho') {
    invariant(effect.heal && typeof effect.heal === 'object',
      `${abilityId} rank ${rank} massTemporalEcho must declare heal`);
    for (const [field, value] of Object.entries(effect.heal)) {
      invariant(typeof value === 'number' && EFFECT_METRIC_FIELDS.includes(field),
        `${abilityId} rank ${rank} massTemporalEcho has unsupported heal field ${field}`);
    }
  }
  for (const [field, value] of Object.entries(effect)) {
    if (field === 'type' || field === 'auraIds' || field === 'auraKind' ||
        field === 'deal' || field === 'heal') continue;
    const supportedTrap = field === 'trap' && effect.type === 'aoeRoot' &&
      value && typeof value === 'object' && !Array.isArray(value) &&
      Object.keys(value).length === 2 &&
      typeof value.armTime === 'number' && typeof value.lifetime === 'number';
    const supported =
      (typeof value === 'number' && EFFECT_METRIC_FIELDS.includes(field)) ||
      (typeof value === 'string' && EFFECT_TEXT_FIELDS.includes(field)) ||
      (typeof value === 'boolean' && EFFECT_FLAG_FIELDS.includes(field)) ||
      supportedTrap;
    invariant(supported, `${abilityId} rank ${rank} has unsupported effect field ${field}`);
  }
}

function renderCatalog(document) {
  const lines = header(document, 'ability headers and resolved ranks');
  lines.push(
    'pub catalogSha(): string {',
    `    return ${zrString(document.catalog_sha256)};`,
    '}',
    '',
    'pub count(): int {',
    `    return ${document.entries.length};`,
    '}',
    '',
    'pub indexOf(id: string): int {',
  );
  for (const entry of document.entries) {
    lines.push(`    if (id == ${zrString(entry.id)}) { return ${entry.index}; }`);
  }
  lines.push(
    '    return -1;',
    '}',
    '',
    'pub idUtf8Length(index: int): int {',
  );
  for (const entry of document.entries) {
    lines.push(`    if (index == ${entry.index}) { return ${Buffer.byteLength(entry.id, 'utf8')}; }`);
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub idUtf8Byte(index: int, byteIndex: int): uint {',
  );
  for (const entry of document.entries) {
    lines.push(`    if (index == ${entry.index}) {`);
    for (const [byteIndex, byte] of Buffer.from(entry.id, 'utf8').entries()) {
      lines.push(`        if (byteIndex == ${byteIndex}) { return <uint>${byte}; }`);
    }
    lines.push('        throw "WOC M4 ability UTF-8 byte index is invalid";', '    }');
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub text(index: int, field: string): string {',
  );
  for (const entry of document.entries) {
    const definition = entry.definition;
    lines.push(`    if (index == ${entry.index}) {`);
    const values = {
      id: definition.id,
      name: definition.name,
      class: definition.class,
      school: definition.school,
      scalesWith: definition.scalesWith ?? '',
      targetType: definition.targetType ?? 'enemy',
      targetMode: definition.targetMode ?? 'entity',
      description: definition.description,
    };
    for (const field of ABILITY_TEXT_FIELDS) {
      lines.push(`        if (field == ${zrString(field)}) { return ${zrString(values[field])}; }`);
    }
    lines.push('        throw "unknown WOC M4 ability text field";', '    }');
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub rankAtLevel(index: int, level: int): int {',
  );
  for (const entry of document.entries) {
    const definition = entry.definition;
    lines.push(`    if (index == ${entry.index}) {`);
    lines.push(`        if (level < ${definition.learnLevel}) { return 0; }`);
    for (const rank of [...(definition.ranks ?? [])].reverse()) {
      lines.push(`        if (level >= ${rank.level}) { return ${rank.rank}; }`);
    }
    lines.push('        return 1;', '    }');
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub rankCount(index: int): int {',
  );
  for (const entry of document.entries) {
    lines.push(`    if (index == ${entry.index}) { return ${(entry.definition.ranks ?? []).length + 1}; }`);
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub metric(index: int, level: int, field: string): float {',
    '    var rank = rankAtLevel(index, level);',
    '    if (rank == 0) { throw "WOC M4 ability is not learned at this level"; }',
  );
  for (const entry of document.entries) {
    const definition = entry.definition;
    lines.push(`    if (index == ${entry.index}) {`);
    for (const field of ABILITY_METRIC_FIELDS) {
      lines.push(`        if (field == ${zrString(field)}) {`);
      for (const rank of [...(definition.ranks ?? [])].reverse()) {
        const override = rankMetricOverride(rank, field);
        if (override !== undefined) {
          lines.push(`            if (rank >= ${rank.rank}) { return ${zrNumber(override)}; }`);
        }
      }
      lines.push(`            return ${zrNumber(baseAbilityMetric(definition, field))};`, '        }');
    }
    lines.push('        throw "unknown WOC M4 ability metric field";', '    }');
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub flag(index: int, field: string): bool {',
  );
  for (const entry of document.entries) {
    const definition = entry.definition;
    lines.push(`    if (index == ${entry.index}) {`);
    for (const field of ABILITY_FLAG_FIELDS) {
      lines.push(
        `        if (field == ${zrString(field)}) { return ${definition[field] === true}; }`,
      );
    }
    lines.push('        throw "unknown WOC M4 ability flag field";', '    }');
  }
  lines.push('    throw "unknown WOC M4 ability index";', '}');
  return `${lines.join('\n')}\n`;
}

function renderEffects(document) {
  const lines = header(document, 'rank-resolved ability effects');
  lines.push('pub count(index: int, rank: int): int {');
  for (const entry of document.entries) {
    lines.push(`    if (index == ${entry.index}) {`);
    for (const resolved of resolvedRanks(entry.definition)) {
      lines.push(`        if (rank == ${resolved.rank}) { return ${resolved.effects.length}; }`);
    }
    lines.push('        throw "unknown WOC M4 ability rank";', '    }');
  }
  lines.push(
    '    throw "unknown WOC M4 ability index";',
    '}',
    '',
    'pub typeAt(index: int, rank: int, effect: int): string {',
  );
  renderEffectBranches(lines, document.entries, (value) => zrString(value.type));
  lines.push(
    '}',
    '',
    'pub metric(index: int, rank: int, effect: int, field: string): float {',
  );
  renderFieldGuard(lines, EFFECT_METRIC_FIELDS, 'effect metric');
  renderEffectBranches(lines, document.entries, (value) => renderEffectMetric(value));
  lines.push(
    '}',
    '',
    'pub flag(index: int, rank: int, effect: int, field: string): bool {',
  );
  renderFieldGuard(lines, EFFECT_FLAG_FIELDS, 'effect flag');
  renderEffectBranches(lines, document.entries, (value) => renderEffectFlag(value));
  lines.push(
    '}',
    '',
    'pub consumeAuraIdCount(index: int, rank: int, effect: int): int {',
  );
  renderEffectBranches(lines, document.entries, (value) => String((value.auraIds ?? []).length));
  lines.push(
    '}',
    '',
    'pub consumeAuraIdAt(index: int, rank: int, effect: int, slot: int): string {',
  );
  renderEffectBranches(lines, document.entries, (value) => renderConsumeAuraId(value));
  lines.push(
    '}',
    '',
    'pub text(index: int, rank: int, effect: int, field: string): string {',
  );
  renderFieldGuard(lines, EFFECT_TEXT_FIELDS, 'effect text');
  renderEffectBranches(lines, document.entries, (value) => renderEffectText(value));
  lines.push('}');
  return `${lines.join('\n')}\n`;
}

function renderFieldGuard(lines, fields, label) {
  lines.push('    if (');
  fields.forEach((field, index) => {
    lines.push(`        field != ${zrString(field)}${index + 1 === fields.length ? '' : ' &&'}`);
  });
  lines.push(`    ) { throw "unknown WOC M4 ${label} field"; }`);
}

function renderEffectBranches(lines, entries, renderValue) {
  for (const entry of entries) {
    lines.push(`    if (index == ${entry.index}) {`);
    for (const resolved of resolvedRanks(entry.definition)) {
      lines.push(`        if (rank == ${resolved.rank}) {`);
      resolved.effects.forEach((effect, effectIndex) => {
        lines.push(`            if (effect == ${effectIndex}) {`);
        const rendered = renderValue(effect);
        if (Array.isArray(rendered)) lines.push(...rendered.map((line) => `                ${line}`));
        else lines.push(`                return ${rendered};`);
        lines.push('            }');
      });
      lines.push('            throw "unknown WOC M4 effect index";', '        }');
    }
    lines.push('        throw "unknown WOC M4 ability rank";', '    }');
  }
  lines.push('    throw "unknown WOC M4 ability index";');
}

function renderEffectMetric(effect) {
  const lines = [];
  for (const field of EFFECT_METRIC_FIELDS) {
    const value = typeof effect[field] === 'number' ? effect[field] :
      effect.deal?.[field] ?? effect.heal?.[field] ??
      (field === 'trapArmTime' ? effect.trap?.armTime : undefined) ??
      (field === 'trapLifetime' ? effect.trap?.lifetime : undefined);
    if (typeof value === 'number') {
      lines.push(`if (field == ${zrString(field)}) { return ${zrNumber(value)}; }`);
    }
  }
  lines.push('return 0.0;');
  return lines;
}

function renderConsumeAuraId(effect) {
  const lines = [];
  for (const [slot, auraId] of (effect.auraIds ?? []).entries()) {
    lines.push(`if (slot == ${slot}) { return ${zrString(auraId)}; }`);
  }
  lines.push('return "";');
  return lines;
}

function renderEffectText(effect) {
  const lines = [];
  for (const field of EFFECT_TEXT_FIELDS) {
    if (typeof effect[field] === 'string') {
      lines.push(`if (field == ${zrString(field)}) { return ${zrString(effect[field])}; }`);
    }
  }
  lines.push('return "";');
  return lines;
}

function renderEffectFlag(effect) {
  const lines = [];
  for (const field of EFFECT_FLAG_FIELDS) {
    lines.push(`if (field == ${zrString(field)}) { return ${effect[field] === true}; }`);
  }
  lines.push('return false;');
  return lines;
}

function resolvedRanks(definition) {
  return [
    { rank: 1, effects: definition.effects },
    ...(definition.ranks ?? []).map((rank) => ({ rank: rank.rank, effects: rank.effects })),
  ];
}

function rankMetricOverride(rank, field) {
  if (field === 'cost') return rank.cost;
  if (field === 'castTime') return rank.castTime;
  if (field === 'threatFlat') return rank.threatFlat;
  return undefined;
}

function baseAbilityMetric(definition, field) {
  const values = {
    learnLevel: definition.learnLevel,
    cost: definition.cost,
    castTime: definition.castTime,
    cooldown: definition.cooldown,
    range: definition.range,
    minRange: definition.minRange ?? 0,
    threatFlat: definition.threat?.flat ?? 0,
    threatMult: definition.threat?.mult ?? 1,
    channelDuration: definition.channel?.duration ?? 0,
    channelTicks: definition.channel?.ticks ?? 0,
    awardsCombo: definition.awardsCombo ?? 0,
  };
  return values[field];
}

function header(document, purpose) {
  return [
    '// Generated by examples/woc/tools/m4_ability_zr_codegen.mjs.',
    `// Source ${document.source_commit}; ${purpose}; do not edit by hand.`,
    '',
  ];
}

function writeOrCheck(path, content) {
  if (checkOnly) {
    invariant(existsSync(path), `${path} is missing; run npm run generate`);
    invariant(readFileSync(path, 'utf8') === content, `${path} is stale; run npm run generate`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, 'utf8');
}

function zrString(value) {
  return JSON.stringify(value);
}

function zrNumber(value) {
  invariant(Number.isFinite(value), `non-finite Zr number ${value}`);
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function hashText(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
