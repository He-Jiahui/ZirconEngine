import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/spell_combat.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'spell_combat_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'spell_combat_contract.zr');
const checkOnly = process.argv.includes('--check');

const blob = execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${SOURCE_PATH}`], { encoding: 'buffer' });
const source = blob.toString('utf8');
for (const kind of ['buff_spellcrit', 'buff_spelldmg', 'form_moonkin', 'buff_spellhaste', 'cast_shield']) invariant(source.includes(`aura.kind === '${kind}'`), `spell combat kind ${kind} drifted`);
invariant(source.includes("else if (aura.kind === 'form_moonkin') bonus += 0.2;") && source.includes('return 1 + bonus;'), 'spell damage multiplier drifted');
invariant(source.includes('let bonus = p.spellHaste;') && source.includes('return 1 + Math.max(0, bonus);'), 'spell haste clamp drifted');
invariant(source.includes('fireMageOnSpellHit(ctx, p, abilityId, crit);'), 'spell hit proc handoff drifted');
const document = {
  schema_version: 1,
  source_commit: SOURCE_COMMIT,
  generated_by: 'examples/woc/tools/spell_combat_contract_codegen.mjs',
  source_blobs: { [SOURCE_PATH]: createHash('sha256').update(blob).digest('hex') },
  kinds: { spell_crit: 'buff_spellcrit', spell_damage: 'buff_spelldmg', moonkin: 'form_moonkin', spell_haste: 'buff_spellhaste', cast_shield: 'cast_shield' },
  moonkin_spell_damage_bonus: 0.2,
  spell_haste: 'one_plus_max_zero_of_resolved_stat_plus_live_aura_bonuses',
};
const json = `${JSON.stringify(document, null, 2)}\n`;
const zr = `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
  `pub isSpellCrit(kind: string): bool { return kind == \"${document.kinds.spell_crit}\"; }\n` +
  `pub isSpellDamage(kind: string): bool { return kind == \"${document.kinds.spell_damage}\"; }\n` +
  `pub isMoonkin(kind: string): bool { return kind == \"${document.kinds.moonkin}\"; }\n` +
  `pub isSpellHaste(kind: string): bool { return kind == \"${document.kinds.spell_haste}\"; }\n` +
  `pub isCastShield(kind: string): bool { return kind == \"${document.kinds.cast_shield}\"; }\n` +
  `pub moonkinSpellDamageBonus(): float { return ${document.moonkin_spell_damage_bonus}; }\n`;
for (const [path, output, label] of [[jsonOutput, json, 'spell combat JSON contract'], [zrOutput, zr, 'spell combat Zr contract']]) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:spell-combat-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:spell-combat-contract`);
  } else writeFileSync(path, output, 'utf8');
}
process.stdout.write(`${checkOnly ? 'checked' : 'generated'} spell combat contract for ${SOURCE_COMMIT}\n`);

function invariant(condition, message) { if (!condition) throw new Error(message); }
