import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const GATHERING_SOURCE_PATH = 'src/sim/professions/gathering.ts';
const NODES_SOURCE_PATH = 'src/sim/content/gather_nodes.ts';
const PROFESSIONS_SOURCE_PATH = 'src/sim/content/professions.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'gathering_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'gathering_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const gathering = sourceBlob(GATHERING_SOURCE_PATH);
  const nodes = sourceBlob(NODES_SOURCE_PATH);
  const professions = sourceBlob(PROFESSIONS_SOURCE_PATH);
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/gathering_contract_codegen.mjs',
    source_blobs: {
      [GATHERING_SOURCE_PATH]: sha256(gathering),
      [NODES_SOURCE_PATH]: sha256(nodes),
      [PROFESSIONS_SOURCE_PATH]: sha256(professions),
    },
    profession_ids: stringArray(professions, 'GATHERING_PROFESSION_IDS'),
    profession_max_skill: professionMaxSkill(professions),
    node_harvest_table: nodeHarvestTable(gathering),
    nodes: gatherNodes(nodes),
    material_rarity: {
      tiers: ['common', 'uncommon', 'rare', 'epic', 'legendary'],
      max_proficiency: numberConstant(gathering, 'MATERIAL_RARITY_MAX_PROFICIENCY'),
      shares: numberRecord(gathering, 'MATERIAL_RARITY_SHARE'),
    },
    harvest_components: stringRecord(professions, 'HARVEST_COMPONENT_ITEMS'),
    harvest_tiers: stringArray(gathering, 'HARVEST_TIERS'),
    focus_tier_weights: numberArray(gathering, 'BASE_TIER_WEIGHTS'),
    corpse_harvest_rarity_baseline: numberConstant(gathering, 'CORPSE_HARVEST_RARITY_BASELINE'),
    semantics: {
      node: 'a ready node first records the player-local respawn time, then consumes exactly one material-rarity RNG draw and queues one matching proficiency grant',
      proficiency: 'mining, logging and herbalism are independent additive counters; non-positive queued grants are ignored and queued grants settle on drain',
      corpse_claim: 'the first claimant succeeds and retains its player id; every later claimant observes that existing id',
      focus: 'empty or full selections spread all tags, otherwise valid selected tags retain selection order; every yielded component consumes one tier RNG draw and concentration shifts its tier upward',
      signing: 'corpse material rarity uses the fixed baseline and only rare, epic and legendary material is signable',
    },
  };
  for (const needle of [
    'meta.nodeHarvestReadyAt[node.id] = now + entry.respawnSeconds;',
    'const rarity = rollMaterialRarity(meta.gatheringProficiency[entry.professionId], rng);',
    'queueGatheringGrant(meta, entry.professionId, 1);',
    'const p = Number.isNaN(proficiency)',
    'let roll = rng.next() * total;',
    'return chosen.length === 0 || chosen.length >= taggedComponents.length',
    'return { success: true, claimedBy: pid };',
  ]) {
    invariant(gathering.includes(needle), 'gathering source drifted: ' + needle);
  }
  invariant(document.profession_ids.join(',') === 'mining,logging,herbalism', 'unexpected gathering profession order');
  invariant(document.nodes.length === 24, 'unexpected gather node count');
  invariant(document.harvest_tiers.join(',') === 'poor,common,uncommon,rare,epic,legendary', 'unexpected harvest tier order');
  invariant(document.focus_tier_weights.join(',') === '40,30,15,10,4,1', 'unexpected focus tier weights');
  invariant(document.material_rarity.max_proficiency === 100, 'unexpected material rarity max proficiency');
  invariant(Object.keys(document.harvest_components).length === 4, 'unexpected harvest component map');
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'gathering JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'gathering Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' gathering contract for ' + SOURCE_COMMIT + '\n');
}

function nodeHarvestTable(source) {
  const block = objectAfterAssignment(source, 'export const NODE_HARVEST_TABLE');
  const entries = [...block.matchAll(/(ore|wood|herb): \{ professionId: '([^']+)', itemId: '([^']+)', respawnSeconds: (\d+) \}/g)]
    .map((entry) => ({ type: entry[1], profession_id: entry[2], item_id: entry[3], respawn_seconds: Number(entry[4]) }));
  invariant(entries.length === 3, 'unexpected node harvest table');
  return entries;
}

function gatherNodes(source) {
  const block = arrayAfterAssignment(source, 'export const GATHER_NODES');
  const entries = topLevelObjects(block).map((body) => ({
    id: requiredString(body, 'id'),
    zone_id: requiredString(body, 'zoneId'),
    type: requiredString(body, 'type'),
    x: requiredNumber(body, 'x'),
    z: requiredNumber(body, 'z'),
    level: requiredNumber(body, 'level'),
  }));
  invariant(entries.every((entry) => ['ore', 'wood', 'herb'].includes(entry.type)), 'unexpected gather node type');
  return entries;
}

function professionMaxSkill(source) {
  const result = {};
  for (const id of stringArray(source, 'GATHERING_PROFESSION_IDS')) {
    const entry = source.match(new RegExp('  ' + id + ': \\{([\\s\\S]*?)\\n  \\},'));
    invariant(entry, 'gathering profession missing: ' + id);
    const maxSkill = entry[1].match(/maxSkill: (\d+),/);
    invariant(maxSkill, 'gathering profession max skill missing: ' + id);
    result[id] = Number(maxSkill[1]);
  }
  return result;
}

function stringArray(source, name) {
  const values = [...arrayAfterAssignment(source, 'const ' + name).matchAll(/'([^']+)'/g)].map((entry) => entry[1]);
  invariant(values.length > 0, 'source no longer exposes nonempty ' + name);
  return values;
}

function numberArray(source, name) {
  const values = [...arrayAfterAssignment(source, 'const ' + name).matchAll(/-?\d+(?:\.\d+)?/g)].map((entry) => Number(entry[0]));
  invariant(values.length > 0, 'source no longer exposes nonempty ' + name);
  return values;
}

function stringRecord(source, name) {
  const result = Object.fromEntries([...objectAfterAssignment(source, 'const ' + name).matchAll(/(\w+): '([^']+)'/g)]
    .map((entry) => [entry[1], entry[2]]));
  invariant(Object.keys(result).length > 0, 'source no longer exposes ' + name);
  return result;
}

function numberRecord(source, name) {
  const result = Object.fromEntries([...objectAfterAssignment(source, 'const ' + name).matchAll(/(\w+): (\d+(?:\.\d+)?)/g)]
    .map((entry) => [entry[1], Number(entry[2])]));
  invariant(Object.keys(result).length > 0, 'source no longer exposes ' + name);
  return result;
}

function numberConstant(source, name) {
  const match = source.match(new RegExp('(?:export )?const ' + name + ' = (\\d+(?:\\.\\d+)?);'));
  invariant(match, 'source no longer exposes ' + name);
  return Number(match[1]);
}

function requiredString(body, field) {
  const match = body.match(new RegExp(field + ": '([^']+)'"));
  invariant(match, 'node missing ' + field);
  return match[1];
}

function requiredNumber(body, field) {
  const match = body.match(new RegExp(field + ': (-?\\d+(?:\\.\\d+)?)'));
  invariant(match, 'node missing ' + field);
  return Number(match[1]);
}

function arrayAfterAssignment(source, anchor) {
  const assignment = source.indexOf('=', source.indexOf(anchor));
  invariant(assignment >= 0, 'source no longer exposes ' + anchor);
  return balancedBlock(source, source.indexOf('[', assignment), '[', ']');
}

function objectAfterAssignment(source, anchor) {
  const assignment = source.indexOf('=', source.indexOf(anchor));
  invariant(assignment >= 0, 'source no longer exposes ' + anchor);
  return balancedBlock(source, source.indexOf('{', assignment), '{', '}');
}

function balancedBlock(source, start, open, close) {
  invariant(start >= 0, 'source block start missing');
  let depth = 0;
  let quote = '';
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (quote !== '') {
      if (character === '\\') index += 1;
      else if (character === quote) quote = '';
      continue;
    }
    if (character === "'" || character === '"' || character === '`') quote = character;
    else if (character === open) depth += 1;
    else if (character === close) {
      depth -= 1;
      if (depth === 0) return source.slice(start + 1, index);
    }
  }
  throw new Error('unterminated source block');
}

function topLevelObjects(block) {
  const result = [];
  for (let index = 0; index < block.length; index += 1) {
    if (block[index] !== '{') continue;
    const end = index + balancedBlock(block, index, '{', '}').length + 1;
    result.push(block.slice(index, end));
    index = end - 1;
  }
  return result;
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.'];
  renderIndexedStrings(lines, 'profession', document.profession_ids);
  lines.push('pub professionMaxSkill(id: string): float {');
  Object.entries(document.profession_max_skill).forEach(([id, skill]) => lines.push('    if (id == "' + id + '") return ' + zrFloat(skill) + ';'));
  lines.push('    return 0.0;');
  lines.push('}');
  renderIndexedStrings(lines, 'node', document.nodes.map((node) => node.id));
  for (const [name, key] of [['Zone', 'zone_id'], ['Type', 'type']]) {
    lines.push('pub node' + name + 'At(index: int): string {');
    document.nodes.forEach((node, index) => lines.push('    if (index == ' + index + ') return "' + node[key] + '";'));
    lines.push('    return "";');
    lines.push('}');
  }
  for (const [name, key] of [['X', 'x'], ['Z', 'z'], ['Level', 'level']]) {
    lines.push('pub node' + name + 'At(index: int): float {');
    document.nodes.forEach((node, index) => lines.push('    if (index == ' + index + ') return ' + zrFloat(node[key]) + ';'));
    lines.push('    return 0.0;');
    lines.push('}');
  }
  for (const [name, key] of [['Profession', 'profession_id'], ['Item', 'item_id']]) {
    lines.push('pub nodeHarvest' + name + 'For(nodeType: string): string {');
    document.node_harvest_table.forEach((entry) => lines.push('    if (nodeType == "' + entry.type + '") return "' + entry[key] + '";'));
    lines.push('    return "";');
    lines.push('}');
  }
  lines.push('pub nodeRespawnSecondsFor(nodeType: string): float {');
  document.node_harvest_table.forEach((entry) => lines.push('    if (nodeType == "' + entry.type + '") return ' + zrFloat(entry.respawn_seconds) + ';'));
  lines.push('    return 0.0;');
  lines.push('}');
  renderIndexedStrings(lines, 'materialRarity', document.material_rarity.tiers);
  lines.push('pub materialRarityMaxProficiency(): float { return ' + zrFloat(document.material_rarity.max_proficiency) + '; }');
  lines.push('pub materialRarityShare(tier: string): float {');
  Object.entries(document.material_rarity.shares).forEach(([tier, value]) => lines.push('    if (tier == "' + tier + '") return ' + zrFloat(value) + ';'));
  lines.push('    return 0.0;');
  lines.push('}');
  const components = Object.entries(document.harvest_components);
  lines.push('pub harvestComponentCount(): int { return ' + components.length + '; }');
  for (const [name, valueIndex] of [['Tag', 0], ['Item', 1]]) {
    lines.push('pub harvestComponent' + name + 'At(index: int): string {');
    components.forEach((entry, index) => lines.push('    if (index == ' + index + ') return "' + entry[valueIndex] + '";'));
    lines.push('    return "";');
    lines.push('}');
  }
  renderIndexedStrings(lines, 'harvestTier', document.harvest_tiers);
  lines.push('pub focusTierWeightAt(index: int): float {');
  document.focus_tier_weights.forEach((weight, index) => lines.push('    if (index == ' + index + ') return ' + zrFloat(weight) + ';'));
  lines.push('    return 0.0;');
  lines.push('}');
  lines.push('pub corpseHarvestRarityBaseline(): float { return ' + zrFloat(document.corpse_harvest_rarity_baseline) + '; }');
  return lines.join('\n') + '\n';
}

function renderIndexedStrings(lines, name, values) {
  lines.push('pub ' + name + 'Count(): int { return ' + values.length + '; }');
  lines.push('pub ' + name + 'At(index: int): string {');
  values.forEach((value, index) => lines.push('    if (index == ' + index + ') return "' + value + '";'));
  lines.push('    return "";');
  lines.push('}');
}

function zrFloat(value) {
  return Number.isInteger(value) ? String(value) + '.0' : String(value);
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), label + ' is missing; run its generate script');
    invariant(readFileSync(path, 'utf8') === output, label + ' is stale; run its generate script');
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
