import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const workspaceRoot = resolve(fileURLToPath(new URL('../../..', import.meta.url)));
const sourceRoot = resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const wocRoot = resolve(workspaceRoot, 'examples', 'woc');
const casting = gitShow('src/sim/combat/casting_lifecycle.ts');
const forms = gitShow('src/sim/combat/forms.ts');
const projection = readFileSync(
  resolve(wocRoot, 'scripts', 'woc_game', 'src', 'combat', 'ability_admission.zr'),
  'utf8',
);

for (const needle of [
  'isResourceShiftFormAuraKind',
  "const shift = formShiftKind(p, res.def);",
  "if (shift === 'cross') {",
  'const parked = p.auras.some((a) => isResourceShiftFormAuraKind(a.kind));',
  'p.savedMana = Math.max(0, p.savedMana - res.cost);',
  'spendResource(p, res.cost);',
]) {
  invariant(casting.includes(needle), `missing current-head cross-form billing behavior: ${needle}`);
}
for (const needle of [
  'export { isFormAuraKind } from \'../types\';',
  'export function isResourceShiftFormAuraKind(kind: AuraKind): boolean',
  "return kind === 'form_bear' || kind === 'form_cat' || kind === 'form_travel';",
]) {
  invariant(forms.includes(needle), `missing current-head form ownership behavior: ${needle}`);
}

for (const needle of [
  'var forms = %import("combat/forms_state");',
  'if (forms.isFormAuraKind(kind)) { return kind; }',
  'if (forms.isFormAuraKind(kind)) {',
  'if (forms.isResourceShiftFormAuraKind(actor.activeForm)) {',
  'actor.savedMana = actor.savedMana > cost ? actor.savedMana - cost : 0;',
  'actor.resource = actor.resource > cost ? actor.resource - cost : 0;',
  'moonkin.activeForm = "form_moonkin";',
  'if (moonkin.resource != 70 || moonkin.savedMana != 37) { return -27; }',
  'cat.activeForm = "form_cat";',
  'if (cat.resource != 100 || cat.savedMana != 70) { return -28; }',
]) {
  invariant(projection.includes(needle), `cross-form mana projection omitted: ${needle}`);
}

process.stdout.write(`checked M4 cross-form mana source projection: ${SOURCE_COMMIT.slice(0, 15)}\n`);

function gitShow(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'utf8',
  });
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
