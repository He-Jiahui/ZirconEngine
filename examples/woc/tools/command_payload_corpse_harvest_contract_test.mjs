import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const read = (...parts) => readFileSync(join(projectRoot, ...parts), 'utf8');
const readSource = (...parts) => readFileSync(join(sourceRoot, ...parts), 'utf8');
const payloads = JSON.parse(read('contracts', 'command_payloads.json'));
const harvest = JSON.parse(read('reference', 'current-head', 'corpse_harvest_contract.json'));
const coverage = JSON.parse(read('reference', 'current-head', 'command_payload_coverage.json'));
const protocol = read('native', 'crates', 'woc_protocol', 'src', 'corpse_harvest_payload.rs');
const generated = read('native', 'crates', 'woc_protocol', 'src', 'generated_command_payloads.rs');
const zr = read('scripts', 'woc_game', 'src', 'generated', 'corpse_harvest_contract.zr');
const input = read('native', 'apps', 'woc_client', 'src', 'input', 'intent.rs');

assert.equal(payloads.schema_version, 60);
assert.deepEqual(payloads.entries.find((entry) => entry.id === 13), {
  id: 13,
  name: 'harvestCorpse',
  kind: 'corpse_harvest',
  min_byte_length: 9,
  max_byte_length: 12,
  max_collection_entries: 3,
  encoding: 'u64_le_target+u8_component_count_0_to_3+component_codes',
  source_shape: { kind: 'client_send', method: 'harvestCorpse', fields: ['id', 'components'] },
});
assert.deepEqual(harvest.tags, ['hide', 'fang', 'claw', 'tusk', 'venomSac', 'silk', 'gills', 'horn']);
assert.equal(harvest.max_components_per_mob, 3);
assert.deepEqual(harvest.component_items, {
  hide: 'boar_hide', fang: 'wolf_fang', silk: 'webwood_silk', venomSac: 'widow_venom_sac',
});
assert.match(readSource('src', 'net', 'online.ts'), /harvestCorpse\(id: number, components\?: string\[\]\)/);
assert.match(readSource('server', 'game.ts'), /case 'harvestCorpse':/);
assert.match(readSource('src', 'sim', 'interaction.ts'), /mob\.harvestClaimedBy = claim\.claimedBy;/);
assert.match(readSource('src', 'sim', 'interaction.ts'), /ctx\.addItemInstance\(itemId, \{ signer: meta\.name \}/);
assert.match(protocol, /pub struct HarvestCorpseCommandPayload/);
assert.match(protocol, /InvalidCorpseHarvestComponentCount/);
assert.match(generated, /pub const HARVEST_CORPSE_COMMAND_ID: u16 = 13;/);
assert.match(zr, /pub componentCode\(tag: string\): uint/);
assert.match(input, /ClientGameplayIntent::HarvestCorpse/);
assert.match(input, /components\.len\(\) >= 3/);
assert.equal(coverage.totals.typed_contract_commands, 157);
assert.equal(coverage.totals.typed_contract_client_send_commands, 156);
assert.equal(coverage.totals.source_shape_only_commands, 0);
assert.equal(coverage.totals.unmapped_dispatch_commands, 8);

const codeByTag = new Map(harvest.tags.map((tag, index) => [tag, index + 1]));
const canonicalize = (components) => components.length >= 3
  ? [0, 0, 0]
  : components.map((component) => codeByTag.get(component) ?? 0);
assert.deepEqual(canonicalize([]), []);
assert.deepEqual(canonicalize(['fang', 'unknown']), [2, 0]);
assert.deepEqual(canonicalize(['hide', 'hide']), [1, 1]);
assert.deepEqual(canonicalize(['hide', 'fang', 'silk']), [0, 0, 0]);

process.stdout.write('Corpse-harvest command payload contract is complete\n');
