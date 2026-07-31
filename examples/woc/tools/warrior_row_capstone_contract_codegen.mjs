import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const ROWS_PATH = 'src/sim/content/warrior_rows.ts';
const CLASSES_PATH = 'src/sim/content/classes.ts';
const DAMAGE_PATH = 'src/sim/combat/damage.ts';
const DISPATCH_PATH = 'src/sim/combat/effect_dispatch.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'warrior_row_capstone_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'warrior_row_capstone_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = readJson(join(referenceRoot, 'source_manifest.json'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before warrior capstone contracts');
  const blobs = Object.fromEntries([ROWS_PATH, CLASSES_PATH, DAMAGE_PATH, DISPATCH_PATH]
    .map((path) => [path, sourceBlob(path)]));
  const rows = blobs[ROWS_PATH].toString('utf8');
  const classes = blobs[CLASSES_PATH].toString('utf8');
  const damage = blobs[DAMAGE_PATH].toString('utf8');
  const dispatch = blobs[DISPATCH_PATH].toString('utf8');
  invariant(rows.includes("id: 'war_row_double_charge'") && rows.includes('bonusCharges: 1') &&
    rows.includes("id: 'war_row_victory_rush'") && rows.includes("ability: 'victory_rush'") &&
    rows.includes("id: 'war_row_lingering_dread'") && rows.includes('fearBreakPct: 0.2') &&
    rows.includes("id: 'war_row_bladestorm'") && rows.includes("ability: 'bladestorm'"),
  'warrior row capstone options drifted');
  invariant(classes.includes("id: 'charge'") && classes.includes('cooldown: 15') &&
    classes.includes("id: 'bladestorm'") && classes.includes('cost: 25') &&
    classes.includes('cooldown: 90') && classes.includes('channel: { duration: 4, ticks: 4 }') &&
    classes.includes("radius: 6") && classes.includes("id: 'victory_rush'") &&
    classes.includes("{ type: 'selfHealPctMax', pct: 0.2 }"),
  'warrior ability data drifted');
  invariant(damage.includes('const VICTORY_RUSH_WINDOW = 20;') &&
    dispatch.includes('breakThreshold:') &&
    dispatch.includes('Math.max(1, Math.round(hostile.maxHp * fearBreakPct))'),
  'warrior capstone runtime projection drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/warrior_row_capstone_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    selections: {
      double_charge: 'war_row_double_charge',
      victory_rush: 'war_row_victory_rush',
      lingering_dread: 'war_row_lingering_dread',
      bladestorm: 'war_row_bladestorm',
    },
    constants: {
      charge_bonus_uses: 1,
      charge_recharge_seconds: 15,
      victory_rush_window_seconds: 20,
      victory_rush_heal_pct_max: 0.2,
      lingering_dread_break_pct: 0.2,
      bladestorm_cost: 25,
      bladestorm_cooldown_seconds: 90,
      bladestorm_duration_seconds: 4,
      bladestorm_ticks: 4,
      bladestorm_radius: 6,
      bladestorm_min_damage: 16,
      bladestorm_max_damage: 22,
    },
    source_semantics: {
      double_charge: 'each spend starts an independent fifteen-second recharge while the second stored use remains available',
      victory_rush: 'a credited kill opens a twenty-second aura window; the successful target-required strike consumes it and heals twenty percent max health',
      lingering_dread: 'fear break threshold is absent without the row and otherwise max(1, round(target max health * 0.2))',
      bladestorm: 'the granted ability is an uninterruptible self-centered live-position channel; its dispatch and damage remain owned by the combat reducer',
    },
  };
  writeOrCheck(jsonOutput, render(document), 'warrior row capstone JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'warrior row capstone Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} warrior row capstone contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  const c = document.constants;
  const s = document.selections;
  return `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n` +
    `pub isDoubleCharge(optionId: string): bool { return optionId == "${s.double_charge}"; }\n` +
    `pub isVictoryRush(optionId: string): bool { return optionId == "${s.victory_rush}"; }\n` +
    `pub isLingeringDread(optionId: string): bool { return optionId == "${s.lingering_dread}"; }\n` +
    `pub isBladestorm(optionId: string): bool { return optionId == "${s.bladestorm}"; }\n` +
    `pub chargeBonusUses(required: bool): int { return required ? ${c.charge_bonus_uses} : 0; }\n` +
    `pub chargeRechargeSeconds(required: bool): float { return required ? ${c.charge_recharge_seconds} : 0.0; }\n` +
    `pub victoryRushWindowSeconds(required: bool): float { return required ? ${c.victory_rush_window_seconds} : 0.0; }\n` +
    `pub victoryRushHealPctMax(required: bool): float { return required ? ${c.victory_rush_heal_pct_max} : 0.0; }\n` +
    `pub lingeringDreadBreakPct(required: bool): float { return required ? ${c.lingering_dread_break_pct} : 0.0; }\n` +
    `pub bladestormCost(required: bool): int { return required ? ${c.bladestorm_cost} : 0; }\n` +
    `pub bladestormCooldownSeconds(required: bool): float { return required ? ${c.bladestorm_cooldown_seconds} : 0.0; }\n` +
    `pub bladestormDurationSeconds(required: bool): float { return required ? ${c.bladestorm_duration_seconds} : 0.0; }\n` +
    `pub bladestormTicks(required: bool): int { return required ? ${c.bladestorm_ticks} : 0; }\n` +
    `pub bladestormRadius(required: bool): float { return required ? ${c.bladestorm_radius} : 0.0; }\n` +
    `pub bladestormMinDamage(required: bool): int { return required ? ${c.bladestorm_min_damage} : 0; }\n` +
    `pub bladestormMaxDamage(required: bool): int { return required ? ${c.bladestorm_max_damage} : 0; }\n`;
}
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:warrior-row-capstone-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:warrior-row-capstone-contract`); return; } writeFileSync(path, output, 'utf8'); }
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function render(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
