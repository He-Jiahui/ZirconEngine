import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const RULES_PATH = 'src/sim/equipment_rules.ts';
const ITEMS_PATH = 'src/sim/items.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'equipment_routing_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'equipment_routing_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const rules = sourceBlob(RULES_PATH);
  const items = sourceBlob(ITEMS_PATH);
  for (const needle of [
    'export function resolveEquipSlot(',
    "if (item.slot !== 'ring') return item.slot;",
    "if (!equipment.ring1) return 'ring1';",
    "if (!equipment.ring2) return 'ring2';",
    "return 'ring1';",
    'export function slotAcceptsItem(item: ItemDef, slot: EquipSlot): boolean {',
    "if (item.slot === 'ring') return slot === 'ring1' || slot === 'ring2';",
    'export function canDualWield(cls: PlayerClass, spec?: string | null): boolean {',
    "return cls === 'rogue' || (cls === 'warrior' && spec === 'fury');",
    'export function canDualWieldTwoHand(cls: PlayerClass, spec?: string | null): boolean {',
    "return cls === 'warrior' && spec === 'fury';",
    "return item.hand ?? 'onehand';",
    'export function canEquipItemInSlot(',
    "if (slot === 'mainhand') return true;",
    "if (slot !== 'offhand' || !canDualWield(cls, spec)) return false;",
    "return hand === 'onehand' || (hand === 'twohand' && canDualWieldTwoHand(cls, spec));",
  ]) {
    invariant(rules.includes(needle), 'equipment routing rule drifted: ' + needle);
  }
  for (const needle of [
    'function desiredEquipSlot(meta: PlayerMeta, itemId: string): EquipSlot | null {',
    "if (!def?.slot) return null;",
    "if (def.kind !== 'weapon') return resolveEquipSlot(def, meta.equipment);",
    "if (hand === 'mainhand') return 'mainhand';",
    "if (hand === 'twohand') {",
    "if (!canDualWieldTwoHand(meta.cls, spec)) return 'mainhand';",
    "return 'offhand';",
    "if (!meta.equipment.mainhand) return 'mainhand';",
    "if (!canDualWield(meta.cls, spec)) return 'mainhand';",
    "if (!canEquipItemInSlot(meta.cls, def, 'offhand', spec)) return 'mainhand';",
  ]) {
    invariant(items.includes(needle), 'equipment desired-slot source drifted: ' + needle);
  }

  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/equipment_routing_contract_codegen.mjs',
    source_blobs: {
      [RULES_PATH]: sha256(rules),
      [ITEMS_PATH]: sha256(items),
    },
    values: {
      ring_kind: 'ring',
      ring_one: 'ring1',
      ring_two: 'ring2',
      mainhand: 'mainhand',
      offhand: 'offhand',
      weapon_kind: 'weapon',
      onehand: 'onehand',
      twohand: 'twohand',
      warrior: 'warrior',
      rogue: 'rogue',
      fury: 'fury',
    },
    semantics: {
      ring_resolution: 'ring1 then ring2, then replace ring1',
      default_weapon_hand: 'onehand',
      dual_wield: 'rogue or warrior fury',
      twohand_offhand: 'warrior fury only',
      eligibility_boundary: 'itemEligible represents preceding canEquipItem content/proficiency admission; requiredClass arrays are not scalarized here',
    },
  };
  writeOrCheck(jsonOutput, JSON.stringify(document, null, 2) + '\n', 'equipment routing JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'equipment routing Zr contract');
  process.stdout.write((checkOnly ? 'checked' : 'generated') + ' equipment routing contract for ' + SOURCE_COMMIT + '\n');
}

function renderZr(document) {
  const lines = ['// Generated from ' + SOURCE_COMMIT + '; do not edit by hand.\n'];
  for (const [name, value] of Object.entries(document.values)) {
    lines.push('pub ' + camelCase(name) + '(): string { return "' + value + '"; }\n');
  }
  return lines.join('');
}

function camelCase(value) {
  return value.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', SOURCE_COMMIT + ':' + path], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
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
