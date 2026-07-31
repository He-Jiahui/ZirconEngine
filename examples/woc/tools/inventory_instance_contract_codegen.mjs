import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const BAGS_PATH = 'src/sim/bags.ts';
const TYPES_PATH = 'src/sim/types.ts';
const SIM_PATH = 'src/sim/sim.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const referenceRoot = resolve(projectRoot, 'reference', 'current-head');
const jsonOutput = join(referenceRoot, 'inventory_instance_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'inventory_instance_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const manifest = JSON.parse(readFileSync(join(referenceRoot, 'source_manifest.json'), 'utf8'));
  invariant(manifest.source_commit === SOURCE_COMMIT,
    'current-head reference inventory must be regenerated before inventory instance contracts');
  const blobs = Object.fromEntries([BAGS_PATH, TYPES_PATH, SIM_PATH].map((path) => [path, sourceBlob(path)]));
  const bags = blobs[BAGS_PATH].toString('utf8');
  const types = blobs[TYPES_PATH].toString('utf8');
  const sim = blobs[SIM_PATH].toString('utf8');
  invariant(bags.includes('const DEFAULT_STACK = 20;') &&
    bags.includes("const UNSTACKED_KINDS = new Set(['weapon', 'armor', 'held_offhand', 'bag', 'tool']);"),
  'bags stack-cap constants drifted');
  invariant(bags.includes('if (s.itemId === itemId && !s.instance && s.count < stack)') &&
    bags.includes('if (s.itemId !== itemId || s.instance || s.count >= stack) continue;') &&
    bags.includes('while (remaining > 0) {\n    const take = Math.min(stack, remaining);\n    inventory.push({ itemId, count: take });'),
  'bags instanced-slot non-merge or stack append semantics drifted');
  invariant(bags.includes('for (let i = inventory.length - 1; i >= 0 && remaining > 0; i--)') &&
    bags.includes('if (s.itemId !== itemId) continue;'),
  'bags reverse removal semantics drifted');
  for (const field of ['signer?: string;', 'charges?: Record<string, number>;', 'rolled?: { quality?: string; stats?: Record<string, number>; masterwork?: boolean };', 'enchant?: string;', 'boundTo?: number;', 'slot?: number;']) {
    invariant(types.includes(field), `ItemInstancePayload/InvSlot field drifted: ${field}`);
  }
  invariant(types.includes('if (src.charges) instance.charges = { ...src.charges };') &&
    types.includes('...(src.rolled.stats && { stats: { ...src.rolled.stats } }),'),
  'item instance deep clone semantics drifted');
  invariant(sim.includes('meta.inventory.push({ itemId, count: 1, instance });') &&
    sim.includes('addStacked(meta.inventory, itemId, count);') &&
    sim.includes('if (s.instance) consumedInstances.push(s.instance);') &&
    sim.includes('if (s.itemId !== itemId || s.instance) continue;'),
  'Sim item hub instance, force-grant, removal, or fungible-only semantics drifted');
  const document = {
    schema_version: 1,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/inventory_instance_contract_codegen.mjs',
    source_blobs: Object.fromEntries(Object.entries(blobs).map(([path, value]) => [path, sha256(value)])),
    default_stack_size: 20,
    unstacked_kinds: ['weapon', 'armor', 'held_offhand', 'bag', 'tool'],
    item_instance_fields: ['signer', 'charges', 'rolled.quality', 'rolled.stats', 'rolled.masterwork', 'enchant', 'boundTo', 'slot'],
    source_semantics: {
      instanced_slots_never_merge: true,
      force_grants_ignore_capacity: true,
      removal_is_newest_first: true,
      fungible_removal_skips_instances: true,
      instance_payload_is_deep_cloned_at_save_load_boundaries: true,
    },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'inventory instance JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'inventory instance Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} inventory instance contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  return [
    `// Generated from ${SOURCE_COMMIT}; do not edit by hand.`,
    `pub defaultStackSize(required: bool): int { return required ? ${document.default_stack_size} : 0; }`,
    'pub instancedSlotsNeverMerge(required: bool): bool { return required; }',
    'pub forceGrantsIgnoreCapacity(required: bool): bool { return required; }',
    'pub removalIsNewestFirst(required: bool): bool { return required; }',
    'pub fungibleRemovalSkipsInstances(required: bool): bool { return required; }',
  ].join('\n') + '\n';
}
function sourceBlob(path) { return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 }); }
function writeOrCheck(path, output, label) { if (checkOnly) { invariant(existsSync(path), `${label} is missing; run npm run generate:inventory-instance-contract`); invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:inventory-instance-contract`); return; } writeFileSync(path, output, 'utf8'); }
function sha256(value) { return createHash('sha256').update(value).digest('hex'); }
function invariant(condition, message) { if (!condition) throw new Error(message); }
