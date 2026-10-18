import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = resolve(projectRoot, '..', '..');
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const source = execFileSync(
  'git', ['-C', sourceRoot, 'show', `${sourceCommit}:src/sim/content/classes.ts`],
  { encoding: 'utf8' },
);
const catalog = JSON.parse(readFileSync(join(projectRoot, 'contracts', 'm4_abilities.json'), 'utf8'));
const generatedCatalog = readFileSync(
  join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm4_ability_catalog.zr'),
  'utf8',
);
const generatedEffects = readFileSync(
  join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'm4_ability_effects.zr'),
  'utf8',
);
const generator = readFileSync(join(projectRoot, 'tools', 'm4_ability_zr_codegen.mjs'), 'utf8');
const plan = readFileSync(
  join(projectRoot, '..', '..', 'docs', 'plans', 'woc', '175-fire-mage-ability-content.md'),
  'utf8',
);

assert.match(source, /pyroblast:[\s\S]*?cost: 125,[\s\S]*?castTime: 6\.0,[\s\S]*?type: 'directDamage', min: 170, max: 225/);
assert.match(source, /flamestrike:[\s\S]*?cost: 80,[\s\S]*?castTime: 2,[\s\S]*?type: 'aoeDamage', min: 34, max: 44, radius: 7, canCrit: true/);
assert.match(source, /combustion:[\s\S]*?cost: 100,[\s\S]*?cooldown: 120,[\s\S]*?offGcd: true,[\s\S]*?type: 'selfBuff', kind: 'combustion', value: 0, duration: 10/);

assert.equal(catalog.entries.length, 117);
for (const [id, effectType] of [
  ['pyroblast', 'directDamage'],
  ['flamestrike', 'aoeDamage'],
  ['combustion', 'selfBuff'],
]) {
  const entry = catalog.entries.find((candidate) => candidate.id === id);
  assert.ok(entry, `${id} is absent from M4 content`);
  assert.equal(entry.definition.effects[0].type, effectType);
}
assert.equal(catalog.entries.find((entry) => entry.id === 'flamestrike').definition.effects[0].canCrit, true);
assert.match(
  generator,
  /'canCrit', 'requiresBehind', 'spell', 'exhaust', 'groupOnly'/,
);
assert.match(generatedCatalog, /if \(id == "pyroblast"\) \{ return \d+; \}/);
assert.match(generatedCatalog, /if \(id == "flamestrike"\) \{ return \d+; \}/);
assert.match(generatedCatalog, /if \(id == "combustion"\) \{ return \d+; \}/);
assert.match(generatedEffects, /if \(field == "canCrit"\) \{ return true; \}/);
assert.match(plan, /Dragon's Breath remains outside this content-only slice/);

process.stdout.write('WOS175 Fire Mage ability content static guards passed\n');
