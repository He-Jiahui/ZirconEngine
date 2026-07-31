import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const FORMS_PATH = 'src/sim/combat/forms.ts';
const TYPES_PATH = 'src/sim/types.ts';
const CLASSES_PATH = 'src/sim/content/classes.ts';
const ENTITY_PATH = 'src/sim/entity.ts';
const PLAYER_MOTION_PATH = 'src/sim/player_motion.ts';
const THREAT_PATH = 'src/sim/threat.ts';
const SPELL_COMBAT_PATH = 'src/sim/combat/spell_combat.ts';
const DAMAGE_PATH = 'src/sim/combat/damage.ts';
const FORM_SWING_PATH = 'src/sim/combat/form_swing.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'forms_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'forms_contract.zr');
const knownAbilityCatalogOutput = join(projectRoot, 'reference', 'current-head', 'known_ability_catalog.json');
const checkOnly = process.argv.includes('--check');

const blobs = Object.fromEntries([
  FORMS_PATH,
  TYPES_PATH,
  CLASSES_PATH,
  ENTITY_PATH,
  PLAYER_MOTION_PATH,
  THREAT_PATH,
  SPELL_COMBAT_PATH,
  DAMAGE_PATH,
  FORM_SWING_PATH,
].map((path) => [path, execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer' })]));
const forms = blobs[FORMS_PATH].toString('utf8');
const types = blobs[TYPES_PATH].toString('utf8');
const classes = blobs[CLASSES_PATH].toString('utf8');
const entity = blobs[ENTITY_PATH].toString('utf8');
const playerMotion = blobs[PLAYER_MOTION_PATH].toString('utf8');
const threat = blobs[THREAT_PATH].toString('utf8');
const spellCombat = blobs[SPELL_COMBAT_PATH].toString('utf8');
const damage = blobs[DAMAGE_PATH].toString('utf8');
const formSwing = blobs[FORM_SWING_PATH].toString('utf8');
const match = types.match(/FORM_AURA_KINDS: ReadonlySet<AuraKind> = new Set<AuraKind>\(\[([\s\S]*?)\]\);/);
invariant(match, 'form aura kind set drifted');
const all = [...match[1].matchAll(/'([^']+)'/g)].map((entry) => entry[1]);
invariant(all.length === 6, 'unexpected form aura kind count');
const resourceShift = ['form_bear', 'form_cat', 'form_travel'];
const resourceBarShift = ['form_bear', 'form_cat'];
const actionLocking = [...resourceShift, 'form_fireball'];
const travel = ['form_travel', 'form_fireball'];
for (const kind of resourceShift) invariant(forms.includes(`kind === '${kind}'`), `resource shift kind ${kind} drifted`);
invariant(forms.includes("return isResourceShiftFormAuraKind(kind) || kind === 'form_fireball';"), 'action locking form rule drifted');
invariant(forms.includes("return kind === 'form_travel' || kind === 'form_fireball';"), 'travel form rule drifted');
invariant(classes.includes('export const ABILITIES:'), 'ability content source drifted');
invariant(entity.includes("bearForm ? 'rage' : catForm ? 'energy' : null;"),
  'resource-bar form transition drifted');
const bearFormBlock = captureBlock(entity, 'bearForm');
const catFormBlock = captureBlock(entity, 'catForm');
const derivedFormRules = {
  bear_armor_multiplier: extractNumber(bearFormBlock, /s\.armor = Math\.round\(s\.armor \* ([0-9.]+)\);/, 'bear armor multiplier'),
  bear_bonus_attack_power_flat: extractNumber(bearFormBlock, /bonusAp \+= ([0-9.]+) \+ Math\.round\(s\.agi \* [0-9.]+\);/, 'bear flat attack power'),
  bear_bonus_attack_power_per_agility: extractNumber(bearFormBlock, /bonusAp \+= [0-9.]+ \+ Math\.round\(s\.agi \* ([0-9.]+)\);/, 'bear agility attack power'),
  bear_max_hp_multiplier: extractNumber(entity, /if \(bearForm\) e\.maxHp = Math\.round\(e\.maxHp \* ([0-9.]+)\);/, 'bear max HP multiplier'),
  cat_bonus_attack_power_flat: extractNumber(catFormBlock, /bonusAp \+= ([0-9.]+) \+ lvl \* [0-9.]+;/, 'cat flat attack power'),
  cat_bonus_attack_power_per_level: extractNumber(catFormBlock, /bonusAp \+= [0-9.]+ \+ lvl \* ([0-9.]+);/, 'cat level attack power'),
  cat_agility_minimum: extractNumber(catFormBlock, /s\.agi \+= Math\.max\(([0-9.]+), Math\.floor\(lvl \/ [0-9.]+\)\);/, 'cat minimum agility'),
  cat_agility_level_divisor: extractNumber(catFormBlock, /s\.agi \+= Math\.max\([0-9.]+, Math\.floor\(lvl \/ ([0-9.]+)\)\);/, 'cat agility level divisor'),
  moonkin_armor_multiplier: extractNumber(entity, /if \(moonkinForm\) s\.armor = Math\.round\(s\.armor \* ([0-9.]+)\);/, 'moonkin armor multiplier'),
  bear_threat_multiplier: extractNumber(threat, /export const BEAR_FORM_THREAT_MULT = ([0-9.]+);/, 'bear threat multiplier'),
  cat_threat_multiplier: extractNumber(threat, /export const CAT_FORM_THREAT_MULT = ([0-9.]+);/, 'cat threat multiplier'),
  moonkin_spell_damage_bonus: extractNumber(spellCombat, /else if \(aura\.kind === 'form_moonkin'\) bonus \+= ([0-9.]+);/, 'moonkin spell damage bonus'),
};
invariant(playerMotion.includes("a.kind === 'buff_speed' || a.kind === 'form_travel' || a.kind === 'form_fireball'"),
  'form movement-speed rule drifted');
invariant(damage.includes("a.kind === 'form_shadow'") && damage.includes('form.value / 100'),
  'shadow form damage rule drifted');
invariant(formSwing.includes("a.kind === 'form_cat') return ROGUE_BASE_SWING_SPEED") &&
  formSwing.includes("new Set(['form_bear', 'form_cat', 'form_travel'])"),
  'form weapon rules drifted');
invariant(existsSync(knownAbilityCatalogOutput), 'known ability catalog is missing; run the known ability catalog generator first');
const knownCatalog = JSON.parse(readFileSync(knownAbilityCatalogOutput, 'utf8'));
invariant(knownCatalog.schema_version >= 3 && knownCatalog.source_commit === SOURCE_COMMIT,
  'known ability catalog lacks current form metadata; regenerate it first');
invariant(knownCatalog.source_blobs?.[CLASSES_PATH] === createHash('sha256').update(blobs[CLASSES_PATH]).digest('hex'),
  'known ability catalog classes source drifted');
invariant(Array.isArray(knownCatalog.abilities), 'known ability catalog abilities are invalid');
const formAbilities = all.map((kind) => {
  const candidates = knownCatalog.abilities.filter((ability) => ability.primary_self_buff_kind === kind);
  invariant(candidates.length === 1, `expected one ability for form kind ${kind}`);
  const ability = candidates[0];
  invariant(Number.isInteger(ability.code) && ability.code > 0 &&
    Number.isInteger(ability.base_cost) && ability.base_cost >= 0 &&
    Number.isFinite(ability.base_cast_time) && ability.base_cast_time >= 0 &&
    Number.isFinite(ability.base_cooldown) && ability.base_cooldown >= 0 &&
    Number.isFinite(ability.primary_self_buff_value) &&
    typeof ability.id === 'string' && /^[a-z0-9_]+$/.test(ability.id),
  `form ability metadata is invalid for ${kind}`);
  return {
    kind,
    id: ability.id,
    code: ability.code,
    base_cost: ability.base_cost,
    base_cast_time: ability.base_cast_time,
    base_cooldown: ability.base_cooldown,
    aura_value: ability.primary_self_buff_value,
  };
});
const document = {
  schema_version: 4,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/forms_contract_codegen.mjs',
  source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, createHash('sha256').update(value).digest('hex')])),
  all,
  resource_shift: resourceShift,
  resource_bar_shift: resourceBarShift,
  action_locking: actionLocking,
  travel,
  form_abilities: formAbilities,
  derived_form_rules: derivedFormRules,
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const predicate = (name, ids) => `pub ${name}(kind: string): bool { return ${ids.map((id) => `kind == \"${id}\"`).join(' || ')}; }\n`;
const formAbilityCodeRows = formAbilities
  .map((ability) => `    if (kind == "${ability.kind}") { return <uint>${ability.code}; }`).join('\n');
const formKindRows = formAbilities
  .map((ability) => `    if (code == <uint>${ability.code}) { return "${ability.kind}"; }`).join('\n');
const formCostRows = formAbilities
  .map((ability) => `    if (code == <uint>${ability.code}) { return ${ability.base_cost}; }`).join('\n');
const formCastTimeRows = formAbilities
  .map((ability) => `    if (code == <uint>${ability.code}) { return ${floatLiteral(ability.base_cast_time)}; }`).join('\n');
const formCooldownRows = formAbilities
  .map((ability) => `    if (code == <uint>${ability.code}) { return ${floatLiteral(ability.base_cooldown)}; }`).join('\n');
const formAuraValueRows = formAbilities
  .map((ability) => `    if (code == <uint>${ability.code}) { return ${floatLiteral(ability.aura_value)}; }`).join('\n');
const derivedFormRuleZr = `pub bearArmorMultiplier(): float { return ${floatLiteral(derivedFormRules.bear_armor_multiplier)}; }\n` +
  `pub bearBonusAttackPowerFlat(): int { return ${derivedFormRules.bear_bonus_attack_power_flat}; }\n` +
  `pub bearBonusAttackPowerPerAgility(): float { return ${floatLiteral(derivedFormRules.bear_bonus_attack_power_per_agility)}; }\n` +
  `pub bearMaxHpMultiplier(): float { return ${floatLiteral(derivedFormRules.bear_max_hp_multiplier)}; }\n` +
  `pub catBonusAttackPowerFlat(): int { return ${derivedFormRules.cat_bonus_attack_power_flat}; }\n` +
  `pub catBonusAttackPowerPerLevel(): int { return ${derivedFormRules.cat_bonus_attack_power_per_level}; }\n` +
  `pub catAgilityMinimum(): int { return ${derivedFormRules.cat_agility_minimum}; }\n` +
  `pub catAgilityLevelDivisor(): int { return ${derivedFormRules.cat_agility_level_divisor}; }\n` +
  `pub moonkinArmorMultiplier(): float { return ${floatLiteral(derivedFormRules.moonkin_armor_multiplier)}; }\n` +
  `pub bearThreatMultiplier(): float { return ${floatLiteral(derivedFormRules.bear_threat_multiplier)}; }\n` +
  `pub catThreatMultiplier(): float { return ${floatLiteral(derivedFormRules.cat_threat_multiplier)}; }\n` +
  `pub moonkinSpellDamageBonus(): float { return ${floatLiteral(derivedFormRules.moonkin_spell_damage_bonus)}; }\n`;
const formResourceKindRows = formAbilities
  .filter((ability) => ability.kind === 'form_bear' || ability.kind === 'form_cat')
  .map((ability) => `    if (code == <uint>${ability.code}) { return <uint>${ability.kind === 'form_bear' ? 2 : 3}; }`).join('\n');
const formPayloadRows = formAbilities.map((ability) => {
  const checks = [...ability.id].map((character, index) =>
    `<uint>payloadBytes[<int>(offset + <uint>${index})] == <uint>${character.charCodeAt(0)}`).join(' &&\n        ');
  return `    if (length == <uint>${ability.id.length} &&\n        ${checks}) { return <uint>${ability.code}; }`;
}).join('\n');
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `var container = %import("zr.container");\n` +
  predicate('isForm', all) + predicate('isResourceShift', resourceShift) +
  predicate('isResourceBarShift', resourceBarShift) + predicate('isActionLocking', actionLocking) + predicate('isTravel', travel) +
  `\npub formAbilityCode(kind: string): uint {\n${formAbilityCodeRows}\n    return <uint>0;\n}\n` +
  `\npub formKindForAbilityCode(code: uint): string {\n${formKindRows}\n    return "";\n}\n` +
  `\npub isFormAbilityCode(code: uint): bool { return formKindForAbilityCode(code) != ""; }\n` +
  `pub formAbilityCodeForExactPayload(\n    payloadBytes: container.Array<uint>, offset: uint, length: uint\n): uint {\n${formPayloadRows}\n    return <uint>0;\n}\n` +
  `pub formCost(code: uint): int {\n${formCostRows}\n    return 0;\n}\n` +
  `\npub formCastTime(code: uint): float {\n${formCastTimeRows}\n    return 0.0;\n}\n` +
  `\npub formCooldown(code: uint): float {\n${formCooldownRows}\n    return 0.0;\n}\n` +
  `\npub formAuraValue(code: uint): float {\n${formAuraValueRows}\n    return 0.0;\n}\n` +
  `\n${derivedFormRuleZr}` +
  `\npub isResourceShiftAbilityCode(code: uint): bool { return isResourceShift(formKindForAbilityCode(code)); }\n` +
  `pub isResourceBarShiftAbilityCode(code: uint): bool { return isResourceBarShift(formKindForAbilityCode(code)); }\n` +
  `pub resourceKindForFormAbilityCode(code: uint): uint {\n${formResourceKindRows}\n    return <uint>0;\n}\n` +
  `pub isActionLockingAbilityCode(code: uint): bool { return isActionLocking(formKindForAbilityCode(code)); }\n` +
  `pub isTravelAbilityCode(code: uint): bool { return isTravel(formKindForAbilityCode(code)); }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'forms JSON contract'], [zrOutput, zr, 'forms Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:forms-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:forms-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} forms contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }

function floatLiteral(value) { return Number.isInteger(value) ? `${value}.0` : String(value); }

function extractNumber(text, pattern, label) {
  const match = text.match(pattern);
  invariant(match && Number.isFinite(Number(match[1])), `${label} source rule drifted`);
  return Number(match[1]);
}

function captureBlock(text, condition) {
  const match = text.match(new RegExp(`if \\(${condition}\\) \\{([\\s\\S]*?)\\n\\s*\\}`, 'm'));
  invariant(match, `${condition} form block drifted`);
  return match[1];
}
