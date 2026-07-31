import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const CLASSES_PATH = 'src/sim/content/classes.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'known_ability_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'known_ability_contract.zr');
const checkOnly = process.argv.includes('--check');

const sourceBlob = execFileSync(
  'git',
  ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${CLASSES_PATH}`],
  { encoding: 'buffer' },
);
const source = sourceBlob.toString('utf8');
const statements = [
  'const baseIds = CLASSES[cls].abilities;',
  'const ids = [...baseIds];',
  'for (const g of mods?.grants ?? []) grantIds.add(g.ability);',
  'for (const g of mods?.grants ?? []) if (!ids.includes(g.ability)) ids.push(g.ability);',
  'const granted = grantIds.has(id) || !baseIds.includes(id);',
  'if (!granted && def.learnLevel > level) continue;',
  'if (!granted && def.specs && (!mods?.spec || !def.specs.includes(mods.spec))) continue;',
  'def.excludeSpecs.includes(mods.spec) &&',
  'level >= (def.excludeSpecsAtLevel ?? 0)',
  'if (mods) applyTalentMods(entry, mods);',
  'out.push(entry);',
];
for (const statement of statements) {
  invariant(source.includes(statement), `abilitiesKnownAt rule drifted: ${statement}`);
}
invariant(
  source.indexOf(statements[1]) < source.indexOf(statements[3]) &&
    source.indexOf(statements[3]) < source.indexOf(statements[4]) &&
    source.indexOf(statements[4]) < source.indexOf(statements[5]) &&
    source.indexOf(statements[5]) < source.indexOf(statements[9]) &&
    source.indexOf(statements[9]) < source.indexOf(statements[10]),
  'abilitiesKnownAt selection order drifted',
);

const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/known_ability_contract_codegen.mjs',
  source_blobs: {
    [CLASSES_PATH]: createHash('sha256').update(sourceBlob).digest('hex'),
  },
  id: 'known_ability_selection',
  rules: [
    'base_class_order_is_preserved',
    'grant_order_is_appended_only_when_absent',
    'missing_definitions_are_skipped_before_all_other_gates',
    'grants_bypass_level_and_spec_gates',
    'exclude_specs_apply_at_the_optional_minimum_level',
    'talent_modifiers_apply_after_selection',
  ],
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub sourceCommit(): string { return ${JSON.stringify(SOURCE_COMMIT)}; }\n` +
  `pub sourceSha256(): string { return ${JSON.stringify(document.source_blobs[CLASSES_PATH])}; }\n` +
  'pub ruleCount(): int { return 6; }\n' +
  'pub grantsBypassLevelAndSpecs(): bool { return true; }\n' +
  'pub appliesTalentModifiersAfterSelection(): bool { return true; }\n';

for (const [path, output, label] of [
  [jsonOutput, json, 'known-ability JSON contract'],
  [zrOutput, zr, 'known-ability Zr contract'],
]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:known-ability-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:known-ability-contract`);
  } else {
    writeFileSync(path, output, 'utf8');
  }
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} known-ability contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
