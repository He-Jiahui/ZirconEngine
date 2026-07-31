import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOURCE_PATH = 'src/sim/combat/talent_procs.ts';
const RNG_SOURCE_PATH = 'src/sim/rng.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'talent_proc_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'talent_proc_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blob = sourceBlob(SOURCE_PATH);
  const source = blob.toString('utf8');
  const rngBlob = sourceBlob(RNG_SOURCE_PATH);
  const rngSource = rngBlob.toString('utf8');
  const triggers = ['castNth', 'spellCrit', 'shieldConsumed', 'hotExpired', 'bigHitTaken', 'meleeSwingWhile', 'thornsReflect'];
  const responses = ['empowerNext', 'cooldownRefund', 'resource', 'heal', 'absorb', 'aura', 'echo'];
  for (const trigger of triggers) invariant(source.includes(`'${trigger}'`), `missing ${trigger} trigger`);
  for (const response of responses) invariant(source.includes(`case '${response}'`), `missing ${response} response`);
  invariant(source.includes('if (wasEmpowered) return;'), 'empowered-cast counter guard drifted');
  invariant(source.includes('procState.icds[key] -= dt;') && source.includes('if (procState.icds[key] <= 0) delete procState.icds[key];'), 'proc ICD expiry rule drifted');
  invariant(source.includes('if (trigger.icd !== undefined && procState.icds[def.id] !== undefined) continue;'), 'cast/crit ICD gate drifted');
  invariant(source.includes('const count = (procState.counters[def.id] ?? 0) + 1;') && source.includes('if (count >= trigger.n) {') && source.includes('procState.counters[def.id] = 0;'), 'castNth counter/reset rule drifted');
  invariant(source.includes('if (trigger.chance !== undefined && !ctx.rng.chance(trigger.chance)) continue;') && source.includes('if (trigger.icd !== undefined) procState.icds[def.id] = trigger.icd;'), 'optional chance/ICD order drifted');
  invariant(source.includes("if (player.maxHp <= 0) return;") && source.includes('amount < player.maxHp * trigger.hpFrac') && source.includes('procState.icds[def.id] = trigger.icd;'), 'big-hit admission or ICD order drifted');
  invariant(source.includes("trigger.ability === 'personal_barrier'") && source.includes('PERSONAL_BARRIER_IDS.includes(shieldAbilityId)'), 'personal barrier sentinel drifted');
  invariant(source.includes("school: def.school ?? 'holy'") && source.includes("fx: 'procSurge'") && source.includes("fx: 'wardBloom'"), 'response default school or FX drifted');
  invariant(rngSource.includes('return this.next() < p;'), 'shared Rng chance boundary drifted');

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/talent_proc_contract_codegen.mjs',
    source_blobs: { [SOURCE_PATH]: sha256(blob), [RNG_SOURCE_PATH]: sha256(rngBlob) },
    triggers,
    responses,
    defaults: { school: 'holy', personal_barrier_sentinel: 'personal_barrier', proc_fx: 'procSurge', ward_fx: 'wardBloom' },
    semantics: {
      cast_nth: 'empowered_casts_skip_counter; active_icd_skips_without_banking; counter_resets_before_optional_chance; failed_chance_does_not_arm_icd',
      crit_and_melee: 'active_icd_skips; optional_chance_draw_occurs_only_after_all_non_rng_admission; shared_rng_chance_is_strictly_random_less_than_chance; icd_arms_only_on_success',
      big_hit: 'max_hp_must_be_positive; amount_must_reach_hp_fraction; icd_arms_before_response',
      tick: 'each_active_icd_subtracts_dt_and_is_removed_at_or_below_zero',
    },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'talent proc JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'talent proc Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} talent proc contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    'pub castNthTriggerCode(): int { return 1; }\n' +
    'pub spellCritTriggerCode(): int { return 2; }\n' +
    'pub shieldConsumedTriggerCode(): int { return 3; }\n' +
    'pub hotExpiredTriggerCode(): int { return 4; }\n' +
    'pub bigHitTakenTriggerCode(): int { return 5; }\n' +
    'pub meleeSwingWhileTriggerCode(): int { return 6; }\n' +
    'pub thornsReflectTriggerCode(): int { return 7; }\n' +
    'pub empowerNextResponseCode(): int { return 1; }\n' +
    'pub cooldownRefundResponseCode(): int { return 2; }\n' +
    'pub resourceResponseCode(): int { return 3; }\n' +
    'pub healResponseCode(): int { return 4; }\n' +
    'pub absorbResponseCode(): int { return 5; }\n' +
    'pub auraResponseCode(): int { return 6; }\n' +
    'pub echoResponseCode(): int { return 7; }\n' +
    `pub defaultSchool(): string { return "${document.defaults.school}"; }\n` +
    `pub personalBarrierSentinel(): string { return "${document.defaults.personal_barrier_sentinel}"; }\n` +
    `pub procFx(): string { return "${document.defaults.proc_fx}"; }\n` +
    `pub wardFx(): string { return "${document.defaults.ward_fx}"; }\n`;
}

function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:talent-proc-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:talent-proc-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
