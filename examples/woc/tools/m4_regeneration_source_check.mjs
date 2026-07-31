import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const wocRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(wocRoot, '..', '..', 'dev', 'world-of-claudecraft');
const source = gitShow('src/sim/combat/auras.ts');
const regeneration = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'combat', 'regeneration_state.zr'),
  'utf8',
);

for (const needle of [
  'if (!isStunned(p)) {',
  "if (aura.kind === 'resource_sap') {",
  'ctx.playerMods(meta).global.manaRegenPct',
  'const secondWindPct = ctx.playerMods(meta).global.secondWindPctPerSec;',
  'for (const slot of [\'eating\', \'drinking\'] as const) {',
]) {
  invariant(source.includes(needle), `source regeneration rule drifted: ${needle}`);
}

assertOrder(source, [
  'if (!isStunned(p)) {',
  "if (p.resourceType === 'mana') {",
  'const secondWindPct = ctx.playerMods(meta).global.secondWindPctPerSec;',
  "for (const slot of ['eating', 'drinking'] as const) {",
]);

for (const needle of [
  'pub var stunned: bool;',
  'pub var resourceSapValues: container.Array<float>;',
  'pub var manaRegenPct: float;',
  'pub var secondWindPctPerSec: float;',
  'if (!state.stunned) {',
  'var sapIndex = 0;',
  '(1.0 + state.manaRegenPct);',
  'if (state.secondWindPctPerSec > 0.0 && state.hp > 0',
  'enhancedMana.resource != 49',
  'stunnedSap.resource != 10',
  'secondWind.hp != 33 || secondWind.healEvents != 1',
]) {
  invariant(regeneration.includes(needle), `WOC regeneration projection is missing: ${needle}`);
}

assertOrder(regeneration, [
  'if (!state.stunned) {',
  'if (state.resourceType == "mana") {',
  'if (state.secondWindPctPerSec > 0.0 && state.hp > 0',
  'if (state.eating) {',
]);

process.stdout.write(`checked M4 regeneration source: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function assertOrder(text, needles) {
  let prior = -1;
  for (const needle of needles) {
    const position = text.indexOf(needle);
    invariant(position >= 0, `missing ordered rule: ${needle}`);
    invariant(position > prior, `source order drifted at: ${needle}`);
    prior = position;
  }
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
