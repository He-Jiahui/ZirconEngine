import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (path) => readFileSync(resolve(root, path), 'utf8');
const requireText = (source, text, label) => {
  if (!source.includes(text)) throw new Error(`WOS39 source contract missing ${label}: ${text}`);
};

const state = read('scripts/woc_game/src/world/state.zr');
for (const text of [
  'writer.u16(<uint>67, 1, 1);',
  'schemaVersion != <uint>39',
  'pub var entityMotionAuraOffsets',
  'pub var entityMotionAuraAbilityCodes',
  'pub var entityMotionAuraSourceIds',
  'pub var entityMotionAuraKindCodes',
  'pub var entityMotionAuraRemaining',
  'if (schemaVersion >= <uint>39)',
  'appendDefaultMotionAuraColumns(state);',
  'stepRetainedPlayerTicks(state);',
  'ageMotionAurasForEntity(state, index);',
]) requireText(state, text, 'world state');
const tick = state.indexOf('stepRetainedPlayerTicks(state);');
const clearDeaths = state.indexOf('clearDeadCasting(state);');
if (tick < 0 || clearDeaths < tick) throw new Error('WOS39 player tick must run before death cleanup');

const motion = read('scripts/woc_game/src/world/motion_aura_state.zr');
for (const text of [
  'pub isStunned(',
  'pub isRooted(',
  'pub hasIceFloes(',
  'pub canCastWhileMoving(',
  'pub remainsActiveAfterTick(',
  'return 0.05;',
]) requireText(motion, text, 'motion helper');

const catalogExtract = read('tools/known_ability_catalog_source_extract.mjs');
const catalogGenerator = read('tools/known_ability_catalog_codegen.mjs');
requireText(catalogExtract, 'cast_while_moving: Boolean(definition.castWhileMoving)', 'catalog source extract');
requireText(catalogGenerator, 'pub baseCastWhileMoving', 'catalog generated resolver');

const ccGenerator = read('tools/cc_contract_codegen.mjs');
for (const text of [
  "ice_floes: 6",
  "id: 'ice_floes'",
  'pub isMotionStunnedKindCode',
  'pub isMotionRootedKindCode',
]) requireText(ccGenerator, text, 'CC motion contract');

const protocol = read('native/crates/woc_protocol/src/lib.rs');
requireText(protocol, 'pub const WORLD_STATE_FORMAT: &str = "WOS67";', 'native state format');
requireText(protocol, 'pub const WORLD_STATE_SCHEMA_VERSION: u16 = 67;', 'native schema version');

process.stdout.write('WOS39 motion aura source contract is current\n');
