import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sourceCommit = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = resolve(projectRoot, '..', '..');
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const source = (path) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${sourceCommit}:${path}`], { encoding: 'utf8' },
);

const classes = source('src/sim/content/classes.ts');
assert.match(
  classes,
  /overload:[\s\S]*?class: 'mage',[\s\S]*?learnLevel: 14,[\s\S]*?cost: 0,[\s\S]*?castTime: 0,[\s\S]*?cooldown: 30,[\s\S]*?school: 'arcane',[\s\S]*?offGcd: true,[\s\S]*?kind: 'overload', value: 0\.4, duration: 10/,
);
const choices = source('src/sim/content/choice_rows_classic.ts');
assert.match(choices, /id: 'mag_r14_overload'[\s\S]*?ability: 'overload'/);

const codegen = read('tools', 'm4_ability_codegen.mjs');
assert.match(codegen, /'overload'/);
assert.match(codegen, /EXPECTED_ABILITY_COUNT = 117/);
const zrCodegen = read('tools', 'm4_ability_zr_codegen.mjs');
assert.match(zrCodegen, /entries\.length === 117/);
const contract = JSON.parse(read('contracts', 'm4_abilities.json'));
assert.equal(contract.entries.length, 117);
const overload = contract.entries.find((entry) => entry.id === 'overload');
assert.deepEqual(overload?.definition.effects, [{ type: 'selfBuff', kind: 'overload', value: 0.4, duration: 10 }]);
assert.equal(overload?.definition.offGcd, true);
assert.equal(overload?.definition.cooldown, 30);

const catalog = read('scripts', 'woc_game', 'src', 'generated', 'm4_ability_catalog.zr');
assert.match(catalog, /pub count\(\): int \{\s*return 117;/);
assert.match(catalog, /id == "overload"/);
const effects = read('scripts', 'woc_game', 'src', 'generated', 'm4_ability_effects.zr');
assert.match(effects, /"overload"/);

const plan = read('..', '..', 'docs', 'plans', 'woc', '188-overload-ability-content.md');
assert.match(plan, /mag_r14_overload/);
assert.match(plan, /without hand-authored content drift/);

process.stdout.write('WOS188 Overload ability content static guards passed\n');
